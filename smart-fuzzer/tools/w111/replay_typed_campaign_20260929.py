"""Replay retained completed W111 typed captures without changing old outcomes."""
import hashlib
import argparse
import json
import subprocess
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parents[3]
parser=argparse.ArgumentParser()
parser.add_argument('--tag',default='roundtrip-v1')
parser.add_argument('--run-pattern',default='w111-*')
parser.add_argument('--run',action='append',default=[],help='Restrict to named runs; repeat for each run.')
parser.add_argument('--binary',help='Use an already built, frozen local replay executable instead of rebuilding.')
parser.add_argument('--comparison-tag',default='existinglocal',help='Compare against local-TAG.jsonl; existinglocal selects the original local.jsonl, whose input parser is not requalified.')
options=parser.parse_args()
manifest = root / "smart-fuzzer/tools/pmt_ppmt_local_eval/Cargo.toml"
if options.binary:
    binary = (root / options.binary).resolve()
    if not binary.is_file(): raise SystemExit(f'Replay executable not found: {binary}')
else:
    subprocess.run(["cargo","build","-q","--manifest-path",str(manifest),"--bin","array_tranche_local_eval"],cwd=root,check=True)
    binary = manifest.parent / "target/debug/array_tranche_local_eval.exe"
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def rows(path):
    result = {}
    for row in map(json.loads,(line for line in path.read_text(encoding="utf-8-sig").split("\n") if line.strip())):
        if row['case_id'] in result: raise ValueError(f'Duplicate case ID in {path}: {row["case_id"]}')
        result[row['case_id']] = row
    return result
report = {"schema_version":"w111.typed_final_roundtrip_replay.v1","generated_utc":datetime.now(timezone.utc).isoformat(),
          "binary_sha256":sha(binary),"parser_policy":"serde_json/float_roundtrip",
          "comparison_tag":options.comparison_tag,
          "comparison_baseline_policy":"historical_capture_input_parser_not_requalified" if options.comparison_tag == 'existinglocal' else 'tagged_local_replay',
          "comparison_policy":"strict_typed_digest_no_normalization","runs":[]}
for run in sorted((root/"smart-fuzzer/runs").glob(options.run_pattern)):
    if options.run and run.name not in options.run: continue
    cases = run/"cases/cases.jsonl"
    if not cases.exists() or not (run/"rollup.json").exists(): continue
    oracle_path = run/"outcomes/excel.jsonl"
    comparison_path = run/('outcomes/local.jsonl' if options.comparison_tag == 'existinglocal' else f'outcomes/local-{options.comparison_tag}.jsonl')
    input_hashes = {str(path.relative_to(root)).replace('\\','/'):sha(path) for path in [cases,oracle_path,comparison_path]}
    captured_manifest = json.loads((run/'manifest.json').read_text(encoding='utf-8-sig'))
    expected_cases_sha256 = captured_manifest.get('materialized_cases_sha256')
    if expected_cases_sha256 is not None and expected_cases_sha256 != sha(cases):
        raise ValueError(f'Materialized cases changed since capture: {cases}')
    case_rows = rows(cases)
    excel = rows(oracle_path)
    before = rows(comparison_path)
    out = run/f"outcomes/local-{options.tag}.jsonl"
    if out.exists(): raise SystemExit(f'Refusing to overwrite a prior replay: {out}')
    subprocess.run([str(binary),"--cases",str(cases),"--out",str(out)],cwd=root,check=True)
    assert all(sha(root/path) == value for path,value in input_hashes.items()), 'Replay inputs changed during execution'
    local = rows(out)
    assert local.keys() == excel.keys() == before.keys() == case_rows.keys(), run
    by_function = defaultdict(lambda:dict(rows=0,exact=0,semantic_rows=0,semantic_exact=0,harness_nonpasses=0,previous_exact=0,changed_local_outcomes=0,new_misses=0,newly_exact=0))
    misses = []
    transitions = []
    for id,row in local.items():
        assert row['function_id'] == excel[id]['function_id'] == before[id]['function_id'] == case_rows[id]['function_id'], (run,id)
        counts = by_function[row["function_id"]]; counts["rows"] += 1
        actual = row["outcome"]["digest_payload"]; expected = excel[id]["outcome"]["digest_payload"]
        previous = before[id]["outcome"]["digest_payload"]
        counts["exact"] += actual == expected
        counts["previous_exact"] += previous == expected
        counts["changed_local_outcomes"] += actual != previous
        newly_exact = actual == expected and previous != expected
        new_miss = actual != expected and previous == expected
        counts['newly_exact'] += newly_exact
        counts['new_misses'] += new_miss
        admitted = row["execution_status"] == "ok" and excel[id]["execution_status"] == "ok"
        counts["semantic_rows"] += admitted
        counts["semantic_exact"] += admitted and actual == expected
        counts["harness_nonpasses"] += not admitted
        if actual != previous:
            transitions.append(dict(case_id=id,function_id=row['function_id'],
                change='newly_exact' if newly_exact else 'new_miss' if new_miss else 'changed_miss',
                previous=previous,actual=actual,expected=expected,
                previous_execution_status=before[id]['execution_status'],local_execution_status=row['execution_status'],
                excel_execution_status=excel[id]['execution_status']))
        if actual != expected:
            misses.append(dict(case_id=id,function_id=row["function_id"],actual=actual,expected=expected,
                               local_execution_status=row["execution_status"],excel_execution_status=excel[id]["execution_status"],
                               same_input_semantic_comparison_admitted=admitted,
                               previous=previous,previous_execution_status=before[id]['execution_status'],new_miss=new_miss))
    report["runs"].append(dict(run_id=run.name,rows=len(local),exact=sum(v["exact"] for v in by_function.values()),
        semantic_rows=sum(v["semantic_rows"] for v in by_function.values()),semantic_exact=sum(v["semantic_exact"] for v in by_function.values()),
        harness_nonpasses=sum(v["harness_nonpasses"] for v in by_function.values()),
        local_outcomes=str(out.relative_to(root)).replace("\\","/"),
        comparison_outcomes=comparison_path.relative_to(root).as_posix(),comparison_sha256=sha(comparison_path),
        input_sha256=input_hashes,captured_case_hash_verified=expected_cases_sha256 is not None,
        newly_exact_ids=[r['case_id'] for r in transitions if r['change']=='newly_exact'],
        new_miss_ids=[r['case_id'] for r in transitions if r['change']=='new_miss'],
        raw_transitions=transitions,by_function=dict(by_function),misses=misses))
assert sha(binary) == report['binary_sha256'], 'Replay executable changed during execution'
if options.run:
    assert {r['run_id'] for r in report['runs']} == set(options.run), 'A requested run was missing or unfinished'
out = root/f"smart-fuzzer/cache/w111-typed-{options.tag}-replay.json"
out.write_text(json.dumps(report,indent=2)+"\n",encoding="utf-8")
print(json.dumps({"report":str(out),"runs":len(report["runs"]),"rows":sum(r["rows"] for r in report["runs"]),"exact":sum(r["exact"] for r in report["runs"])}))
