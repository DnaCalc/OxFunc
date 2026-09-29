"""Independent typed aggregate preparation after a frozen local collector repair."""
import copy,itertools,json,random,hashlib
from pathlib import Path
import gen_broad_typed_20260929 as g
rng=random.Random(0x20260929ac0149);cache=g.ROOT/'smart-fuzzer/cache/w111-aggregate-prepared-20260929';g.TRANCHE='w111-aggregate-prepared-independent-20260929';g.cases.clear()
for fn in ['HARMEAN','DEVSQ']:
 for i in range(6):
  base=[g.n(rng.uniform(.125,32.))for _ in range(3)]
  values=[g.blank(),g.missing(),g.e('Num'),g.e('Div0'),g.e('Ref'),g.b(False),g.t('x'),g.t('3.75')]
  for pos,v in itertools.product(range(3),values):
   args=copy.deepcopy(base);args[pos]=v;g.emit(fn,f'{i}-position{pos}-{v["kind"]}',args,axis='independent_scalar_collection')
 for i,(v,w) in enumerate(itertools.product([g.n(0),g.n(-3.25),g.missing(),g.blank(),g.e('Ref'),g.e('Div0'),g.t('x')],repeat=2)):
  anchor=g.n(rng.uniform(.25,99.));row=[v,anchor,w]
  g.emit(fn,f'order-{i}',row,axis='independent_coercion_before_domain')
  if all(x['kind']not in ['missing_arg','empty_cell']for x in row):
   g.emit(fn,f'row-{i}',[g.a([row])],axis='independent_array_order')
   g.emit(fn,f'column-{i}',[g.a([[x]for x in row])],axis='independent_array_order')
  if all(x['kind']!='missing_arg'for x in row):
   g.emit(fn,f'ref-{i}',[g.r('B200:D200')],[g.fix(f'{chr(66+j)}200',x)for j,x in enumerate(row)],axis='independent_reference_order')
 for i,row in enumerate([[g.blank()],[g.blank(),g.blank()],[g.b(True),g.t('3.75'),g.t('x')],[g.blank(),g.e('Div0')]]):
  end=chr(66+len(row)-1);target='B210'if len(row)==1 else f'B210:{end}210'
  g.emit(fn,f'no-count-reference-{i}',[g.r(target)],[g.fix(f'{chr(66+j)}210',x)for j,x in enumerate(row)],axis='independent_empty_reference')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111aggregateindependent-')
p=cache/'typed-independent.json';p.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',tranche_id=g.TRANCHE,cases=g.cases,tranches=[dict(tranche_id=g.TRANCHE,case_ids=[c['case_id']for c in g.cases])]),indent=2)+'\n',encoding='utf-8')
(cache/'typed-independent-manifest.json').write_text(json.dumps(dict(cases=len(g.cases),source_freeze='prepared-v1-freeze.json',batch_sha256=hashlib.sha256(p.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),seed='0x20260929ac0149',qualification='Independent random positive anchors and3-position/error-order controls; no arbitrary locale grammar; no output normalization.'),indent=2)+'\n')
print(json.dumps(dict(path=str(p),cases=len(g.cases),tranche=g.TRANCHE)))
