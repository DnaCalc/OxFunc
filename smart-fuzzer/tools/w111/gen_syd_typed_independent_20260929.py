"""Independent SYD typed values and array-shape/error-precedence controls."""
import copy,hashlib,json,random
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/cache/w111-depreciation-20260929';g.TRANCHE='w111-syd-typed-independent-20260929'
rng=random.Random(202609292030)
base=[g.n(-2876.4375),g.n(17.8125),g.n(13.375),g.n(4.625)]
for pos in range(4):
 values=[g.n(rng.uniform(-10.,15.)),g.n(0.),g.n(1e-200),g.t('2.875'),g.t(''),g.t('bad value'),g.b(True),g.b(False),g.missing(),g.blank(),g.e('Div0'),g.e('NA')]
 for i,value in enumerate(values):
  args=copy.deepcopy(base);args[pos]=value;g.emit('SYD',f'independent-p{pos}-direct-{i}',args)
 for i,value in enumerate([g.n(3.875),g.t('2.875'),g.t(''),g.blank(),g.b(True),g.e('Ref')]):
  args=copy.deepcopy(base);args[pos]=g.r('D7');g.emit('SYD',f'independent-p{pos}-ref-{i}',args,[g.fix('D7',value)],axis='reference_coercion')
 args=copy.deepcopy(base);args[pos]=g.a([[base[pos],g.n(0.),g.n(-.375)],[g.t(''),g.b(False),g.e('NA')]]);g.emit('SYD',f'independent-p{pos}-array',args,axis='array_lift')
 args=copy.deepcopy(base);args[pos]=g.r('D7:E8');g.emit('SYD',f'independent-p{pos}-area',args,[g.fix('D7',base[pos]),g.fix('E7',g.blank()),g.fix('D8',g.t('2.875')),g.fix('E8',g.e('Div0'))],axis='reference_array_lift')
for left in range(4):
 for right in range(left+1,4):
  for earlier,later in [('Ref','Num'),('Num','Div0')]:
   args=copy.deepcopy(base);args[left]=g.e(earlier);args[right]=g.e(later);g.emit('SYD',f'independent-errors-{left}-{right}-{earlier}',args)
  for reversed_shapes in [False,True]:
   args=copy.deepcopy(base)
   a=g.a([[base[left],g.e('Ref')],[g.e('Div0'),base[left]]])
   b=g.a([[base[right],base[right],base[right]]])
   args[left],args[right]=(b,a) if reversed_shapes else (a,b)
   g.emit('SYD',f'independent-padding-{left}-{right}-{reversed_shapes}',args,axis='array_lift')
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111syd-independent-')
path=out/'typed-independent-syd.json';doc=dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',tranche_id=g.TRANCHE,cases=g.cases,tranches=[dict(tranche_id=g.TRANCHE,case_ids=[c['case_id'] for c in g.cases])])
path.write_text(json.dumps(doc,indent=2)+'\n',encoding='utf-8');path.with_suffix('.jsonl').write_text(''.join(json.dumps(c,ensure_ascii=False)+'\n' for c in g.cases),encoding='utf-8')
(out/'typed-independent-syd-manifest.json').write_text(json.dumps(dict(cases=len(g.cases),seed=202609292030,batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),scope='Independent typed scalar/reference/array controls plus12 incompatible-shape discovery controls; simple ASCII numeric text only.'),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),cases=len(g.cases))))
