"""Retain ATAN capture provenance, frozen candidates and typed exact replay banks."""
import hashlib,json,shutil
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=root/'docs/function-lane/evidence/w111-broad-20260929/atan'
def retain(source,dest):
 dest.parent.mkdir(parents=True,exist_ok=True)
 if dest.exists():assert dest.read_bytes()==source.read_bytes(),f'Frozen artifact differs: {dest}'
 else:shutil.copyfile(source,dest)
for run,tag in [('w111-atan-heldout-20260929','first-heldout'),('w111-atan-heldout2-20260929','second-heldout'),('w111-atan-refinement-20260929','refinement-inputs')]:
 p=root/'smart-fuzzer/runs'/run
 for name in ['manifest.json','candidate-freeze.json','batch.json','typed.json']:
  if (p/name).exists():retain(p/name,out/tag/name)
 if (p/'candidate').exists():
  for source in (p/'candidate').rglob('*'):
   if source.is_file():retain(source,out/tag/'candidate'/source.relative_to(p/'candidate'))
for run,tag in [('w111-atan-heldout-typed-20260929','typed-heldout'),('w111-atan-repeat-controls-typed-20260929','typed-repeat-controls')]:
 p=root/'smart-fuzzer/runs'/run
 read=lambda path:[json.loads(s) for s in path.read_text(encoding='utf-8-sig').split('\n') if s.strip()]
 cases=read(p/'cases/cases.jsonl');answers={r['case_id']:r for r in read(p/'outcomes/excel.jsonl')}
 bank={'source_run':run,'cases_sha256':hashlib.sha256((p/'cases/cases.jsonl').read_bytes()).hexdigest(),'excel_sha256':hashlib.sha256((p/'outcomes/excel.jsonl').read_bytes()).hexdigest(),'witnesses':[{'case':c,'excel':answers[c['case_id']]} for c in cases]}
 dest=out/(tag+'-bank.json');blob=json.dumps(bank,ensure_ascii=False,separators=(',',':')).encode()
 if dest.exists():assert dest.read_bytes()==blob
 else:dest.write_bytes(blob)
 for name in ['manifest.json','rollup.json','comparisons/comparisons.jsonl','outcomes/local.jsonl']:
  retain(p/name,out/tag/name)
for name in ['w111-atan-graph-results.json','w111-atan-refinement-graph-results.json','w111-fpatan-control-results.json','w111-fpatan-affinity-results.json']:
 retain(root/'.tmp'/name,out/'graph-research'/name)
manifest={'schema_version':'w111.atan_artifacts.v1','files':[{'path':p.relative_to(out).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(out.rglob('*')) if p.is_file() and p.name!='artifact-manifest.json']}
(out/'artifact-manifest.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
print('Retained',len(manifest['files']),'ATAN artifacts')
