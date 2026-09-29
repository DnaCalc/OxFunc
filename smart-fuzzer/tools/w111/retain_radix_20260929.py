"""Bank exact radix source cases and oracle outputs; no inferred expectations."""
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parents[3]
out=root/'docs/function-lane/evidence/w111-broad-20260929/radix'
out.mkdir(parents=True,exist_ok=True)
manifest={'schema_version':'w111.radix_artifacts.v1','files':[]}
def retain(src,dest):
    dst=out/dest;dst.parent.mkdir(parents=True,exist_ok=True)
    dst.write_bytes(src.read_bytes());manifest['files'].append({'source':str(src.relative_to(root)),'retained':dest,'sha256':hashlib.sha256(dst.read_bytes()).hexdigest()})
for path in (root/'smart-fuzzer/runs/w111-radix-heldout-20260929/answers').glob('*.json'):
    retain(path,'heldout/'+path.name)
retain(root/'smart-fuzzer/runs/w111-radix-heldout-20260929/batches/manifest.json','heldout-input-manifest.json')
for run,name in [('w111-radix-typed-20260929','typed-discovery'),
                 ('w111-radix-typed-heldout-20260929-002','typed-heldout')]:
    p=root/'smart-fuzzer/runs'/run
    load=lambda path:[json.loads(s) for s in path.read_text(encoding='utf-8-sig').split('\n') if s.strip()]
    cases=load(p/'cases/cases.jsonl');excel={x['case_id']:x for x in load(p/'outcomes/excel.jsonl')}
    doc={'source_run':run,'manifest':json.loads((p/'manifest.json').read_text(encoding='utf-8-sig')),
         'source_cases_sha256':hashlib.sha256((p/'cases/cases.jsonl').read_bytes()).hexdigest(),
         'source_excel_sha256':hashlib.sha256((p/'outcomes/excel.jsonl').read_bytes()).hexdigest(),
         'witnesses':[{'case':c,'excel':excel[c['case_id']]} for c in cases]}
    assert len(cases)==len(excel)
    (out/(name+'.json')).write_text(json.dumps(doc,ensure_ascii=False,separators=(',',':')),encoding='utf-8')
(out/'artifact-manifest.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
print('Retained numeric heldout and typed discovery/heldout')
