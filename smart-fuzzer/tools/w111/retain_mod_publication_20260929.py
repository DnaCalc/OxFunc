"""Retain lossless public MOD black-box evidence and failed frozen candidates."""
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parents[3];dest=root/'docs/function-lane/evidence/w111-broad-20260929/mod-publication';dest.mkdir(exist_ok=True);manifest=[]
def keep(p,name):
 raw=p.read_bytes()
 if p.suffix=='.json':data=json.loads(raw.decode('utf-8-sig'))
 elif p.suffix=='.jsonl':data=[json.loads(line)for line in raw.decode('utf-8-sig').split('\n')if line.strip()]
 else:data={'source':str(p.relative_to(root)).replace('\\','/'),'sha256':hashlib.sha256(raw).hexdigest(),'utf8_source':raw.decode('utf-8-sig')}
 out=dest/name;out.write_text(json.dumps(data,ensure_ascii=False,separators=(',',':'))+'\n',encoding='utf-8')
 manifest.append(dict(source=str(p.relative_to(root)).replace('\\','/'),source_sha256=hashlib.sha256(raw).hexdigest(),retained=name,retained_sha256=hashlib.sha256(out.read_bytes()).hexdigest()))
for run in sorted((root/'smart-fuzzer/runs').glob('w111-mod-*-20260929')):
 for p in sorted(run.iterdir()):
  if p.suffix in ['.json','.rs','.snapshot']:keep(p,run.name.removeprefix('w111-mod-').removesuffix('-20260929')+'-'+p.name+('.json'if p.suffix!='.json'else''))
for run in sorted((root/'smart-fuzzer/runs').glob('w111-mod-*-20260929')):
 for sub in ['cases/cases.jsonl','outcomes/excel.jsonl','outcomes/local.jsonl']+[str(p.relative_to(run)).replace('\\','/')for p in sorted((run/'outcomes').glob('local-mod-*.jsonl'))]:
  p=run/sub
  if p.exists():keep(p,run.name.removeprefix('w111-mod-').removesuffix('-20260929')+'-'+p.name.replace('.jsonl','.json'))
for p in sorted((root/'smart-fuzzer/cache').glob('w111-typed-mod-*-replay.json')):keep(p,p.name)
for p in sorted((root/'.tmp').glob('w111-mod-*.json')):keep(p,p.name)
for rel in ['crates/oxfunc_core/src/functions/mod_fn.rs','formal/lean/OxFunc/RemainderPublication.lean','formal/lean/OxFunc/Functions/ModFn.lean','crates/oxfunc_core/tests/w111_mod_publication.rs','crates/oxfunc_core/tests/fixtures/w111_mod_publication.json','smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/mod_publication_explorer.rs']:
 p=root/rel
 if p.exists():keep(p,'current-'+p.name+'.json')
for p in sorted((root/'smart-fuzzer/tools/w111').glob('gen_mod*.py')):keep(p,'generator-'+p.name+'.json')
(dest/'artifact-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8');print(len(manifest))
