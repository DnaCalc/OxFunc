"""Bank every date-discovery observation, including contradictory WEEKNUM rows."""
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parents[3]
out=root/'docs/function-lane/evidence/w111-broad-20260929/dates';out.mkdir(exist_ok=True)
manifest={'schema_version':'w111.date_artifacts.v1','files':[]}
for run,name in [('w111-date-boundaries-20260929','boundaries'),('w111-date-conversion-20260929','conversion'),('w111-date-integer-20260929','integer'),('w111-date-heldout-20260929','heldout'),('w111-date-refinement-20260929','refinement'),('w111-date-heldout2-20260929','heldout2')]:
 p=root/'smart-fuzzer/runs'/run
 for source in [p/'capture_manifest.json',*sorted((p/'answers').glob('*.json')),*p.glob('candidate-freeze.json')]:
  dest=out/name/source.name;dest.parent.mkdir(exist_ok=True)
  doc=json.loads(source.read_text(encoding='utf-8-sig'))
  dest.write_text(json.dumps(doc,separators=(',',':')),encoding='utf-8')
  manifest['files'].append({'source':str(source.relative_to(root)).replace('\\','/'),'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'retained':str(dest.relative_to(out)).replace('\\','/'),'sha256':hashlib.sha256(dest.read_bytes()).hexdigest()})
typed=root/'smart-fuzzer/runs/w111-date-typed-20260929'
for relative in ['cases/cases.jsonl','outcomes/excel.jsonl','outcomes/local.jsonl','manifest.json','rollup.json','comparisons/comparisons.jsonl']:
 source=typed/relative;dest=out/'typed-first'/relative;dest.parent.mkdir(parents=True,exist_ok=True)
 if dest.exists():
  assert dest.read_bytes()==source.read_bytes(), f'Frozen evidence changed: {dest}'
 else:
  dest.write_bytes(source.read_bytes())
 manifest['files'].append({'source':str(source.relative_to(root)).replace('\\','/'),'retained':str(dest.relative_to(out)).replace('\\','/'),'sha256':hashlib.sha256(dest.read_bytes()).hexdigest()})
(out/'artifact-manifest.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
print('Retained',len(manifest['files']),'date artifacts')
