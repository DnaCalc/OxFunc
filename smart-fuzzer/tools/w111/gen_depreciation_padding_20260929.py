"""Financial array padding: earlier explicit errors versus later absent cells."""
import copy,hashlib,json
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/cache/w111-depreciation-20260929';g.TRANCHE='w111-depreciation-padding-20260929'
specs={'SLN':[1000.,100.,10.],'SYD':[1000.,100.,10.,2.], 'DB':[1000.,100.,10.,2.,6.], 'DDB':[1000.,100.,10.,2.,2.], 'VDB':[1000.,100.,10.,1.,2.,2.]}
for fn,values in specs.items():
 base=list(map(g.n,values))
 if fn=='VDB':base.append(g.b(True))
 for earlier,later in [(0,1),(0,2),(1,2)]:
  for tag,value in [('ref',g.e('Ref')),('div0',g.e('Div0')),('num',g.e('Num')),('badtext',g.t('bad')),('domain',g.n(-1.))]:
   args=copy.deepcopy(base)
   args[earlier]=g.a([[base[earlier],base[earlier],value],[base[earlier],base[earlier],value]])
   args[later]=g.a([[base[later],base[later]],[base[later],base[later]]])
   g.emit(fn,f'earlier-{earlier}-later-padding-{later}-{tag}',args,axis='array_lift')
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111dep-padding-')
path=out/'typed-depreciation-padding.json';doc=dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',tranche_id=g.TRANCHE,cases=g.cases,tranches=[dict(tranche_id=g.TRANCHE,case_ids=[c['case_id'] for c in g.cases])])
path.write_text(json.dumps(doc,indent=2)+'\n',encoding='utf-8');path.with_suffix('.jsonl').write_text(''.join(json.dumps(c,ensure_ascii=False)+'\n' for c in g.cases),encoding='utf-8')
(out/'typed-depreciation-padding-manifest.json').write_text(json.dumps(dict(cases=len(g.cases),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),scope='Discovery only: five financial functions, absent cells versus earlier explicit coercion errors and numerical admission controls.'),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),cases=len(g.cases))))
