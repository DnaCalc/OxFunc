"""SYD typed discovery; explicit missing, reference shape, and error order."""
import copy,hashlib,json
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/cache/w111-depreciation-20260929';g.TRANCHE='w111-syd-typed-20260929'
base=[g.n(1234.125),g.n(12.875),g.n(8.5),g.n(2.125)]
for pos in range(4):
 for i,value in enumerate([g.n(-2.5),g.n(0.),g.n(1e-300),g.n(1e155),g.t('3.75'),g.t(''),g.t('bad'),g.b(True),g.b(False),g.missing(),g.blank(),g.e('Div0'),g.e('NA')]):
  args=copy.deepcopy(base);args[pos]=value;g.emit('SYD',f'p{pos}-direct-{i}',args)
 for i,value in enumerate([g.n(-2.5),g.t('3.75'),g.t(''),g.blank(),g.b(True),g.e('Ref')]):
  args=copy.deepcopy(base);args[pos]=g.r('B3');g.emit('SYD',f'p{pos}-ref-{i}',args,[g.fix('B3',value)],axis='reference_coercion')
 args=copy.deepcopy(base);args[pos]=g.a([[base[pos],g.n(0.),g.n(-.25)],[g.t(''),g.b(True),g.e('Num')]]);g.emit('SYD',f'p{pos}-array',args,axis='array_lift')
 args=copy.deepcopy(base);args[pos]=g.r('B3:C4');g.emit('SYD',f'p{pos}-area',args,[g.fix('B3',base[pos]),g.fix('C3',g.blank()),g.fix('B4',g.t('3.75')),g.fix('C4',g.e('Num'))],axis='reference_array_lift')
for left in range(4):
 for right in range(left+1,4):
  for x,y in [('NA','Div0'),('Ref','Value')]:
   args=copy.deepcopy(base);args[left]=g.e(x);args[right]=g.e(y);g.emit('SYD',f'error-{left}-{right}-{x}',args)
args=copy.deepcopy(base);args[0]=g.a([[g.n(-700.),g.n(900.),g.n(1300.)]]);args[2]=g.a([[g.n(5.)],[g.n(10.)]]);g.emit('SYD','row-column-broadcast',args,axis='array_lift')
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111syd-typed-')
path=out/'typed-syd.json';doc=dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',tranche_id=g.TRANCHE,cases=g.cases,tranches=[dict(tranche_id=g.TRANCHE,case_ids=[c['case_id'] for c in g.cases])])
path.write_text(json.dumps(doc,indent=2)+'\n',encoding='utf-8');path.with_suffix('.jsonl').write_text(''.join(json.dumps(c,ensure_ascii=False)+'\n' for c in g.cases),encoding='utf-8')
(out/'typed-syd-manifest.json').write_text(json.dumps(dict(cases=len(g.cases),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),scope='Typed discovery; simple ASCII decimal numeric text only.'),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),cases=len(g.cases))))
