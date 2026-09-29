"""Retain aggregate prepared observations, source freezes, and strict judgments."""
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parents[3];cache=root/'smart-fuzzer/cache/w111-aggregate-prepared-20260929';dest=root/'docs/function-lane/evidence/w111-broad-20260929/aggregate-prepared';dest.mkdir(exist_ok=True);manifest=[]
def keep(path,name):
 if not path.exists():return
 raw=path.read_bytes();text=raw.decode('utf-8-sig');data=[json.loads(line)for line in text.split('\n')if line.strip()]if path.suffix=='.jsonl'else json.loads(text)
 out=dest/name;out.write_text(json.dumps(data,ensure_ascii=False,separators=(',',':'))+'\n',encoding='utf-8');manifest.append(dict(source=str(path.relative_to(root)).replace('\\','/'),source_sha256=hashlib.sha256(raw).hexdigest(),retained=name,retained_sha256=hashlib.sha256(out.read_bytes()).hexdigest()))
for run,tag in [('w111-aggregate-prepared-audit-typed-20260929','discovery'),('w111-aggregate-prepared-independent-20260929','independent'),('w111-aggregate-error-origin-typed-20260929','cross-origin-discovery'),('w111-aggregate-origin-heldout-typed-20260929','cross-origin-provisional'),('w111-aggregate-origin-serial-typed-20260929','cross-origin-serial-independent')]:
 p=root/'smart-fuzzer/runs'/run
 for sub in ['cases/cases.jsonl','outcomes/excel.jsonl','outcomes/local.jsonl','outcomes/local-aggregate-prepared-v1.jsonl','outcomes/local-aggregate-prepared-independent-v1.jsonl','outcomes/local-aggregate-direct-prepass-v2.jsonl','manifest.json','rollup.json']:
  keep(p/sub,tag+'-'+Path(sub).name.replace('.jsonl','.json'))
for p in cache.glob('*.json'):keep(p,p.name)
for tag in ['aggregate-prepared-v1','aggregate-prepared-independent-v1','aggregate-direct-prepass-v2']:keep(root/f'smart-fuzzer/cache/w111-typed-{tag}-replay.json','typed-'+tag+'-replay.json')
for p in ['formal/lean/OxFunc/AggregatePublication.lean','formal/lean/OxFunc/Functions/HarMeanFn.lean','formal/lean/OxFunc/Functions/DevSqFn.lean','crates/oxfunc_core/tests/aggregate_prepared_excel_parity_20260929.rs']:
 f=root/p;data={'source':p,'sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'utf8_source':f.read_text(encoding='utf-8')};out=dest/(f.name+'.json');out.write_text(json.dumps(data,ensure_ascii=False,separators=(',',':'))+'\n',encoding='utf-8')
(dest/'artifact-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8');print(json.dumps({'retained':len(manifest),'source_bindings':4}))
