"""Retain the explicit production-provider comparison, including context exclusions."""
import hashlib
import json
from collections import defaultdict
from pathlib import Path

root = Path(__file__).resolve().parents[3]
run = root / "smart-fuzzer/runs/w111-locale-profile-20260929"
dest = root / "docs/function-lane/evidence/w111-broad-20260929/locale-context"
def rows(path):
    return {r["case_id"]:r for line in path.read_text(encoding="utf-8-sig").split("\n") if line.strip() for r in [json.loads(line)]}
local_path = root / "smart-fuzzer/cache/w111-locale-profile-local-20260929.jsonl"
local, excel = rows(local_path), rows(run/"outcomes/excel.jsonl")
assert local.keys() == excel.keys()
groups = defaultdict(lambda:dict(rows=0,raw_exact=0,admitted=0,admitted_exact=0,harness_nonpasses=0,context_excluded=0))
observations = []
for id, actual in local.items():
    expected = excel[id]
    fn = actual["function_id"]
    width = fn in {"FUNC.ASC","FUNC.DBCS","FUNC.JIS"}
    execution_ok = actual["execution_status"] == expected["execution_status"] == "ok"
    admitted = execution_ok and not width
    match = actual["outcome"]["digest_payload"] == expected["outcome"]["digest_payload"]
    counts = groups[fn]
    for key, value in dict(rows=1,raw_exact=match,admitted=admitted,admitted_exact=admitted and match,harness_nonpasses=not execution_ok,context_excluded=width).items(): counts[key] += value
    observations.append(dict(case_id=id,function_id=fn,exact=match,semantic_comparison_admitted=admitted,
        qualification="missing_width_conversion_host_info; JIS host name availability also differs" if width else "explicit_recorded_locale_provider",
        local=actual,excel=expected))
report = dict(schema_version="w111.locale_profile_comparison.v1",comparison="ordinal text, exact numeric bits, exact error code; no normalization",
    rows=len(local),by_function=dict(groups),observations=observations)
(dest/"comparison.json").write_text(json.dumps(report,ensure_ascii=False,separators=(",",":"))+"\n",encoding="utf-8")
sources = [run/"cases/cases.jsonl",run/"manifest.json",run/"rollup.json",local_path,
           local_path.with_suffix(".provenance.json"),run/"outcomes/excel.jsonl"]
manifest=[]
for src in sources:
    if not src.exists(): continue
    data = src.read_bytes()
    name = "explicit-local.json" if src == local_path else src.name.replace(".jsonl",".json")
    if src.suffix == ".jsonl": payload=list(rows(src).values())
    else: payload=json.loads(data.decode("utf-8-sig"))
    (dest/name).write_text(json.dumps(payload,ensure_ascii=False,separators=(",",":"))+"\n",encoding="utf-8")
    manifest.append(dict(source=str(src.relative_to(root)).replace("\\","/"),source_sha256=hashlib.sha256(data).hexdigest(),retained=name))
(dest/"retention-manifest.json").write_text(json.dumps(manifest,indent=2)+"\n",encoding="utf-8")
print(json.dumps(dict(by_function=groups,retained=len(manifest))))
