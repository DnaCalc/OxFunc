"""Independent MOD prepared validation: omissions, origins, shape and error competition."""
import copy,json,random,itertools,hashlib
from pathlib import Path
from datetime import datetime,timezone
import gen_broad_typed_20260929 as g
root=g.ROOT;out=root/'smart-fuzzer/runs/w111-mod-prepared-heldout-20260929';out.mkdir(exist_ok=True);assert not(out/'candidate-freeze.json').exists()
rng=random.Random(202609293309);g.TRANCHE='w111-mod-prepared-heldout-20260929';g.cases.clear()
errors=[g.e(c)for c in ['Ref','Div0','Value','Num','NA','Name','Null']]
scalars=[g.n(rng.uniform(-20,20))for _ in range(8)]+[g.n(0),g.b(True),g.b(False),g.t('4.25'),g.t('x'),g.t(''),g.blank(),g.missing()]+errors
for i,v in enumerate(scalars):
 for reverse in [False,True]:
  args=[g.missing(),v]
  if reverse:args.reverse()
  g.emit('MOD',f'omission-{i}-{reverse}',args,axis='independent_omission')
  if v['kind']!='missing_arg':
   args=[g.missing(),g.r('B200')]
   if reverse:args.reverse()
   g.emit('MOD',f'omission-ref-{i}-{reverse}',args,[g.fix('B200',v)],axis='independent_omission_reference')
shapes=[(1,1),(1,2),(2,1),(2,2),(2,3),(3,1),(3,2)]
for i,(sa,sb)in enumerate(itertools.product(shapes,repeat=2)):
 for mode in range(5):
  a=[[g.n(rng.uniform(-9,9))for _ in range(sa[1])]for _ in range(sa[0])];b=[[g.n(rng.uniform(.01,9))for _ in range(sb[1])]for _ in range(sb[0])]
  if mode in [1,3]:a[-1][0]=copy.deepcopy(errors[i%7])
  if mode in [2,3]:b[0][-1]=copy.deepcopy(errors[(i+2)%7])
  if mode==4:a[0][0]=g.t('x');b[-1][-1]=g.t('4.25')
  g.emit('MOD',f'shapes-{i}-{mode}',[g.a(a),g.a(b)],axis='independent_binary_padding_errors')
  if mode==3:
   fixtures=[]
   for col0,values in [(2,a),(6,b)]:
    for ri,row in enumerate(values):
     for ci,v in enumerate(row):fixtures.append(g.fix(f'{chr(64+col0+ci)}{220+ri}',v))
   ta='B220'if sa==(1,1)else f'B220:{chr(65+sa[1])}{219+sa[0]}';tb='F220'if sb==(1,1)else f'F220:{chr(69+sb[1])}{219+sb[0]}'
   g.emit('MOD',f'ref-shapes-{i}',[g.r(ta),g.r(tb)],fixtures,axis='independent_reference_padding_errors')
for i,shape in enumerate(shapes):
 a=[[g.n(rng.uniform(-9,9))for _ in range(shape[1])]for _ in range(shape[0])]
 for v in [g.missing(),g.e('Ref'),g.t('x')]:
  for reverse in [False,True]:
   args=[g.a(a),v]
   if reverse:args.reverse()
   g.emit('MOD',f'scalar-array-{i}-{v["kind"]}-{reverse}',args,axis='independent_unit_and_scalar_array')
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111modprepared-')
p=out/'typed.json';p.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',tranche_id=g.TRANCHE,cases=g.cases,tranches=[dict(tranche_id=g.TRANCHE,case_ids=[c['case_id']for c in g.cases])]),separators=(',',':')),encoding='utf-8')
paths=['crates/oxfunc_core/src/functions/mod_fn.rs','formal/lean/OxFunc/RemainderPublication.lean','formal/lean/OxFunc/Functions/ModFn.lean','crates/oxfunc_core/tests/w111_mod_publication.rs','crates/oxfunc_core/src/functions/surface_dispatch_unary_numeric_spec_generator.rs','crates/oxfunc_core/src/functions/surface_dispatch_by_index_generated.rs','crates/oxfunc_core/src/functions/binary_numeric.rs','crates/oxfunc_core/src/functions/adapters.rs']
freeze=dict(frozen_utc=datetime.now(timezone.utc).isoformat(),source_sha256={p:hashlib.sha256((root/p).read_bytes()).hexdigest()for p in paths},seed=202609293309,rows=len(g.cases),case_sha256=hashlib.sha256(p.read_bytes()).hexdigest(),stage='independent_prepared_v2')
(out/'candidate-freeze.json').write_text(json.dumps(freeze,indent=2),encoding='utf-8')
for src in paths:(out/(Path(src).name+'.snapshot')).write_bytes((root/src).read_bytes())
print(len(g.cases))
