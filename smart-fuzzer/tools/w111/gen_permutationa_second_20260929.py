"""Fresh power graphs and typed PERMUTATIONA after decimal-scale refinement."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/runs/w111-permutationa-second-20260929';out.mkdir(parents=True,exist_ok=True);assert not (out/'manifest.json').exists()
seed=202609292803;rng=random.Random(seed);sources={}
for file in ['functions/permutationa_fn.rs','coercion.rs','coercion_decimal.rs','coercion_decimal_powers.rs']:
 p=g.ROOT/'crates/oxfunc_core/src'/file;q=out/'candidate'/file;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(p.read_bytes());sources[file]=hashlib.sha256(p.read_bytes()).hexdigest()
if (out/'candidate-freeze.json').exists():
 assert json.loads((out/'candidate-freeze.json').read_text())['source_sha256']==sources
else:(out/'candidate-freeze.json').write_text(json.dumps({'source_sha256':sources,'frozen_utc':datetime.now(timezone.utc).isoformat(),'seed':seed},indent=2))
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex();rows=[]
for _ in range(20000):
 n=rng.randrange(2,2147483646)+rng.random();limit=max(1,int(308/math.log10(n)));k=rng.randrange(0,limit+3)+rng.random();rows.append((n,k))
rows.extend((rng.uniform(1,50),rng.uniform(0,310)) for _ in range(5000))
rows.extend((10+rng.random(),k+rng.random()) for k in range(321))
rows=list(dict.fromkeys(tuple(map(bits,row)) for row in rows));p=out/'batch-permutationa.json';p.write_text(json.dumps({'function':'PERMUTATIONA','probes':[{'probe':{'id':f'w111perma-second-{i:06d}','args':r}} for i,r in enumerate(rows)]},separators=(',',':')))
g.TRANCHE='w111-permutationa-second-20260929'
values=[g.n(0),g.n(-.75),g.n(3.5),g.n(2147483646.75),g.n(2147483647),g.n(126.75),g.t(' 10.75 '),g.t('10%'),g.t('bad'),g.b(True),g.b(False),g.blank(),g.e('Ref'),g.e('Div0'),g.missing()]
for axis in [0,1]:
 for v in values:
  args=[g.n(10),g.n(3)];args[axis]=v;g.emit('PERMUTATIONA','direct',args)
  if v['kind']!='missing_arg':
   args=[g.n(10),g.n(3)];args[axis]=g.r('B2');g.emit('PERMUTATIONA','reference',args,[g.fix('B2',v)])
for x in values[:-1]:
 if x['kind']=='empty_cell':continue
 g.emit('PERMUTATIONA','array',[g.a([[g.n(10),x,g.e('Ref')]]),g.a([[g.n(3),g.n(126)]])])
 g.emit('PERMUTATIONA','unit-array',[g.a([[x]]),g.a([[g.n(3),g.n(126)]])])
g.emit('PERMUTATIONA','reference-array',[g.r('B2:D2'),g.a([[g.n(3),g.n(126)]])],[g.fix('B2',g.n(10)),g.fix('C2',g.blank()),g.fix('D2',g.e('Ref'))])
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111perma-second-')
(out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')))
(out/'manifest.json').write_text(json.dumps({'seed':seed,'rows':len(rows),'typed_rows':len(g.cases),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()},indent=2));print(len(rows),'numeric;',len(g.cases),'typed')
