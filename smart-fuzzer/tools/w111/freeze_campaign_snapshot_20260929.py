"""Freeze executables and replay the original broad banks without oracle access."""
import argparse, hashlib, json, shutil, subprocess
from datetime import datetime, timezone
from pathlib import Path
root = Path(__file__).resolve().parents[3]
p = argparse.ArgumentParser(); p.add_argument('--tag', required=True); a = p.parse_args()
dest = root / f'docs/function-lane/evidence/w111-broad-20260929/snapshot-{a.tag}'
assert not dest.exists(), dest
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def run(args): return subprocess.run(args, cwd=root, check=True, capture_output=True, text=True, encoding='utf-8')
def sources():
    paths = run(['git', 'ls-files', '--cached', '--others', '--exclude-standard']).stdout.splitlines()
    return {v: sha(root/v) for v in sorted(set(paths)) if v.startswith(('crates/', 'formal/lean/', 'smart-fuzzer/engine/', 'smart-fuzzer/tools/')) and Path(v).suffix in {'.rs','.lean','.toml','.lock','.py','.ps1'} and (root/v).is_file()}
before = sources()
source_lf_hashes = {v: hashlib.sha256((root/v).read_bytes().replace(b'\r\n', b'\n')).hexdigest() for v in before}
for manifest, binary in [('smart-fuzzer/engine/Cargo.toml','sf'),('smart-fuzzer/tools/pmt_ppmt_local_eval/Cargo.toml','array_tranche_local_eval')]:
    result = run(['cargo','build','--manifest-path',manifest,'--bin',binary])
    (root/f'.tmp/w111-{binary}-build-{a.tag}.log').write_text(result.stdout+result.stderr, encoding='utf-8')
assert sources() == before, 'Source changed during builds; retry under a new tag'
frozen = {}
for name, source in [('numeric','smart-fuzzer/engine/target/debug/sf.exe'),('typed','smart-fuzzer/tools/pmt_ppmt_local_eval/target/debug/array_tranche_local_eval.exe')]:
    target = root/f'.tmp/w111-{name}-snapshot-{a.tag}.exe'; assert not target.exists()
    shutil.copyfile(root/source,target); frozen[name] = {'path':target.relative_to(root).as_posix(),'sha256':sha(target)}
result=run(['python','-X','utf8','smart-fuzzer/tools/w111/snapshot_numeric_20260929.py','--tag',a.tag,'--binary',frozen['numeric']['path']]); print(result.stdout)
dest.mkdir()
shutil.copytree(root/f'smart-fuzzer/cache/w111-snapshot-{a.tag}/inputs',dest/'inputs')
input_hashes={v.relative_to(dest).as_posix():sha(v) for v in sorted((dest/'inputs').rglob('answers-*.json'))}
result=run(['python','-X','utf8','smart-fuzzer/tools/w111/replay_typed_campaign_20260929.py','--tag',f'snapshot-{a.tag}','--comparison-tag','snapshot-1735','--binary',frozen['typed']['path'],'--run','w111-broad-typed-20260929-002','--run','w111-numeric-text-20260929']); print(result.stdout)
assert sources() == before, 'Source changed during replay; retained candidate remains historical only'
result=run(['python','-X','utf8','smart-fuzzer/tools/w111/reconcile_campaign_snapshot_20260929.py','--tag',a.tag]); print(result.stdout)
(dest/'source-freeze.json').write_text(json.dumps({'base_commit':run(['git','rev-parse','HEAD']).stdout.strip(),'tag':a.tag,'captured_utc':datetime.now(timezone.utc).isoformat(),'files':before,'files_lf_sha256':source_lf_hashes,'source_hash_policy':'files hashes the captured working-tree bytes; files_lf_sha256 normalizes CRLF to LF for Git checkout comparison; retained oracle artifacts have -text attributes','binaries':frozen},indent=2)+'\n',encoding='utf-8')
# The initial executable is judged on exactly the final snapshot's qualified inputs.
baseline=root/'.tmp/sf-baseline-11b2350.exe'
baseline_summary={'binary_sha256':sha(baseline),'comparison':'same qualified input banks as current snapshot','groups':{}}
regressions={'policy':'exact same admitted inputs; a new miss is retained even when net agreement improves','numeric_baseline':'initial immutable sf executable','typed_baseline':'tagged float_roundtrip replay snapshot-1735','groups':{}}
for group in ['numeric','extended']:
    banks=sorted((dest/'inputs'/group).glob('answers-*.json'))
    reports=json.loads(run([str(baseline),'judge-witnesses',*[str(v) for v in banks],'--max-misses','100000','--json']).stdout)
    (dest/f'baseline-{group}-judgement.json').write_text(json.dumps(reports,separators=(',',':')),encoding='utf-8')
    baseline_summary['groups'][group]={'rows':sum(v['rows_judged'] for v in reports),'exact':sum(v['rows_agree'] for v in reports),'functions':len(reports)}
    current={v['function_id']:v for v in json.loads((dest/f'{group}-qualified.json').read_text())['functions']}
    assert set(current) == {v['function_id'] for v in reports}, 'Baseline/current function sets differ'
    transitions=[]
    for previous in reports:
        now=current[previous['function_id']]
        assert previous['rows_judged'] == now['rows_judged'], 'Baseline/current row counts differ'
        assert all(len(v['misses']) == v['rows_judged']-v['rows_agree'] for v in [previous,now]), 'Truncated misses cannot support transition accounting'
        old_misses={v['id']:v for v in previous['misses']}; new_misses={v['id']:v for v in now['misses']}
        fixed=sorted(old_misses.keys()-new_misses.keys()); added=sorted(new_misses.keys()-old_misses.keys())
        if fixed or added: transitions.append({'function_id':previous['function_id'],'newly_exact_ids':fixed,'new_misses':[new_misses[v] for v in added]})
    regressions['groups'][group]={'newly_exact':sum(len(v['newly_exact_ids']) for v in transitions),'new_misses':sum(len(v['new_misses']) for v in transitions),'functions':transitions}
assert all(sha(dest/path) == value for path,value in input_hashes.items()), 'Retained numeric inputs changed during baseline replay'
(dest/'numeric-input-hashes.json').write_text(json.dumps(input_hashes,indent=2)+'\n',encoding='utf-8')
typed=json.loads((dest/'typed-summary.json').read_text())
qualification={(r['run'],r['case_id']):r for r in json.loads((dest/'report.json').read_text())['typed_qualification']}
raw_typed=[]; admitted_typed=[]; withheld_typed=[]
for replay in typed['runs']:
    source=root/replay['comparison_outcomes']
    assert sha(source) == replay['comparison_sha256'], 'Typed baseline changed after comparison'
    target=dest/replay['run_id']/'outcomes'/source.name
    target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source,target)
    for transition in replay['raw_transitions']:
        item={'run':replay['run_id'],**transition}; raw_typed.append(item)
        q=qualification[(replay['run_id'],transition['case_id'])]
        assert (q['actual'],q['expected']) == (transition['actual'],transition['expected'])
        reason=q['withheld_reason'] if not q['admitted'] else ('baseline_execution_nonpass' if transition['previous_execution_status']!='ok' else None)
        if reason is None: admitted_typed.append(item)
        else: withheld_typed.append({**item,'withheld_reason':reason})
regressions['groups']['typed']={
    'comparison_tag':typed['comparison_tag'],
    'qualification':'same per-row admitted/withheld classification as report.json; baseline execution must also be ok',
    'newly_exact':sum(v['change']=='newly_exact' for v in admitted_typed),
    'new_misses':sum(v['change']=='new_miss' for v in admitted_typed),
    'newly_exact_ids':[{'run':v['run'],'case_id':v['case_id'],'function_id':v['function_id']} for v in admitted_typed if v['change']=='newly_exact'],
    'new_miss_rows':[v for v in admitted_typed if v['change']=='new_miss'],
    'changed_miss_rows':[v for v in admitted_typed if v['change']=='changed_miss'],
    'withheld_transitions':withheld_typed}
(dest/'typed-raw-transitions.json').write_text(json.dumps({'comparison_tag':typed['comparison_tag'],'qualification':'raw digest transitions before semantic exclusions','rows':raw_typed},indent=2)+'\n',encoding='utf-8')
(dest/'baseline-summary.json').write_text(json.dumps(baseline_summary,indent=2)+'\n',encoding='utf-8')
(dest/'regression-summary.json').write_text(json.dumps(regressions,indent=2)+'\n',encoding='utf-8')
manifest={'artifacts':[{'path':v.relative_to(dest).as_posix(),'sha256':sha(v)} for v in sorted(dest.rglob('*')) if v.is_file() and v.name!='artifact-manifest.json']}
(dest/'artifact-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'tag':a.tag,'source_files':len(before),'baseline':baseline_summary,'frozen':frozen}))
