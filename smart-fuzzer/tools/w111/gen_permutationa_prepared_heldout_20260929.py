"""Fresh prepared PERMUTATIONA rows after missing/padding repair."""
import hashlib,json,random
from datetime import datetime,timezone
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/runs/w111-permutationa-prepared-heldout-20260929';out.mkdir(parents=True,exist_ok=True);assert not (out/'candidate-freeze.json').exists()
seed=202609292804;rng=random.Random(seed);sources={}
for file in ['functions/permutationa_fn.rs','functions/distribution_common.rs','coercion.rs']:
 p=g.ROOT/'crates/oxfunc_core/src'/file;q=out/'candidate'/file;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(p.read_bytes());sources[file]=hashlib.sha256(p.read_bytes()).hexdigest()
(out/'candidate-freeze.json').write_text(json.dumps({'source_sha256':sources,'frozen_utc':datetime.now(timezone.utc).isoformat(),'seed':seed},indent=2))
g.TRANCHE='w111-permutationa-prepared-heldout-20260929'
for _ in range(60):
 n=rng.uniform(0,14);k=rng.uniform(-.9,15)
 for args in [[g.missing(),g.n(k)],[g.n(n),g.missing()]]:g.emit('PERMUTATIONA','fresh-missing',args)
for error in [g.e('Ref'),g.e('Div0'),g.e('Num'),g.e('Name'),g.e('Value'),g.t('not-a-number'),g.n(-.25),g.n(2147483648)]:
 for vertical in [False,True]:
  wide=g.a([[g.n(10),g.n(3.75),error]] if not vertical else [[g.n(10)],[g.n(3.75)],[error]])
  narrow=g.a([[g.n(2.75),g.n(4.5)]] if not vertical else [[g.n(2.75)],[g.n(4.5)]])
  for args in [[wide,narrow],[narrow,wide]]:
   g.emit('PERMUTATIONA','fresh-positional-padding',args)
   for axis in [0,1]:
    unit=list(args);unit[axis]=g.a([[error]]);g.emit('PERMUTATIONA','fresh-unit-array',unit)
for value in [g.t(' 7.25 '),g.t('2.5%'),g.t('(4)'),g.t('FALSE'),g.b(False),g.b(True),g.blank(),g.n(2147483646.75),g.e('Null')]:
 for axis in [0,1]:
  args=[g.n(7.75),g.n(3.75)];args[axis]=value;g.emit('PERMUTATIONA','fresh-scalar',args)
  args=[g.n(7.75),g.n(3.75)];args[axis]=g.r('B2');g.emit('PERMUTATIONA','fresh-reference',args,[g.fix('B2',value)])
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111perma-prepared-')
p=out/'typed.json';p.write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')))
(out/'manifest.json').write_text(json.dumps({'seed':seed,'rows':len(g.cases),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()},indent=2));print(len(g.cases),'fresh typed rows')
