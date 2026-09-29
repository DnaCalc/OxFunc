"""Fresh structural depreciation cohort after the numeric candidate freeze."""
import copy,hashlib,json,random
from pathlib import Path
from datetime import datetime,timezone
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/cache/w111-depreciation-20260929'
g.TRANCHE='w111-depreciation-typed-independent-20260929'
rng=random.Random(202609292013)
source=g.ROOT/'crates/oxfunc_core/src/functions/depreciation_family.rs'
specs={'DB':[g.n(1234.5678),g.n(12.345),g.n(7.75),g.n(3.25),g.n(9.5)],
       'DDB':[g.n(1234.5678),g.n(12.345),g.n(7.75),g.n(3.25),g.n(1.875)],
       'VDB':[g.n(1234.5678),g.n(12.345),g.n(7.75),g.n(1.125),g.n(3.375),g.n(1.875),g.b(True)]}
for fn,base in specs.items():
 positions=range(6 if fn=='VDB' else 5)
 for pos in positions:
  replacements=[g.n(rng.uniform(.1,11.9)),g.n(0.),g.n(-.125),g.t('3.75'),g.t(''),g.t('not numeric'),g.b(False),g.b(True),g.missing(),g.blank(),g.e('NA'),g.e('Div0')]
  for index,value in enumerate(replacements):
   args=copy.deepcopy(base);args[pos]=value
   g.emit(fn,f'fresh-p{pos}-direct-{index}',args)
  for index,value in enumerate([g.n(2.875),g.t('3.75'),g.blank(),g.e('Ref'),g.b(True)]):
   args=copy.deepcopy(base);args[pos]=g.r('C3')
   g.emit(fn,f'fresh-p{pos}-reference-{index}',args,[g.fix('C3',value)],axis='reference_coercion')
  for index,rows in enumerate([[[base[pos],g.n(0.),g.n(-.125)]],[[base[pos]],[g.e('Value')],[g.t('3.75')]],[[base[pos],g.b(True)],[g.t(''),g.e('NA')]]]):
   args=copy.deepcopy(base);args[pos]=g.a(rows)
   g.emit(fn,f'fresh-p{pos}-array-{index}',args,axis='array_lift')
  args=copy.deepcopy(base);args[pos]=g.r('C3:D4')
  g.emit(fn,f'fresh-p{pos}-mixed-area',args,[g.fix('C3',base[pos]),g.fix('D3',g.blank()),g.fix('C4',g.t('3.75')),g.fix('D4',g.e('Num'))],axis='reference_array_lift')
 for left in range(4):
  for right in range(left+1,5):
   args=copy.deepcopy(base);args[left]=g.e('NA');args[right]=g.e('Div0')
   g.emit(fn,f'fresh-error-order-{left}-{right}',args)
 args=copy.deepcopy(base);args[0]=g.a([[g.n(750.),g.n(1000.),g.n(1500.)]]);args[1]=g.a([[g.n(0.)],[g.n(75.)]])
 g.emit(fn,'fresh-row-column-broadcast',args,axis='array_lift')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111dep-independent-')
path=out/'typed-independent.json'
doc=dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',tranche_id=g.TRANCHE,cases=g.cases,tranches=[dict(tranche_id=g.TRANCHE,case_ids=[c['case_id'] for c in g.cases])])
path.write_text(json.dumps(doc,indent=2)+'\n',encoding='utf-8')
path.with_suffix('.jsonl').write_text(''.join(json.dumps(c,ensure_ascii=False)+'\n' for c in g.cases),encoding='utf-8')
(out/'typed-independent-manifest.json').write_text(json.dumps(dict(generated_utc=datetime.now(timezone.utc).isoformat(),cases=len(g.cases),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),scope='DB/DDB full prepared arguments; VDB no_switch=true only. ASCII decimal numeric text only; locale grammar and switched VDB remain separate open lanes.'),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),cases=len(g.cases))))
