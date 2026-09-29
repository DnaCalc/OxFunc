"""Fresh validation after integer and hyperbolic candidate freezes.

TANH/COTH still have known residuals; their captures are further characterization.
"""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3];out=ROOT/'smart-fuzzer/runs/w111-integer-hyperbolic-heldout-20260929';batch=out/'batches';batch.mkdir(parents=True,exist_ok=True)
assert not (out/'candidate-freeze.json').exists()
seed=202609292741;rng=random.Random(seed);sources={}
for name in ['int_fn.rs','round_fn.rs','rounddown_fn.rs','roundup_fn.rs','trunc_fn.rs','sinh.rs','cosh.rs','tanh.rs','coth.rs','csch.rs','surface_dispatch.rs']:
 p=ROOT/'crates/oxfunc_core/src/functions'/name;q=out/'candidate'/name;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(p.read_bytes());sources[name]=hashlib.sha256(p.read_bytes()).hexdigest()
(out/'candidate-freeze.json').write_text(json.dumps({'source_sha256':sources,'frozen_utc':datetime.now(timezone.utc).isoformat(),'seed':seed,'known_residuals':['TANH','COTH','exceptional initial decimal conversion']},indent=2))
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex();decode=lambda x:struct.unpack('>d',int(x).to_bytes(8,'big'))[0]
safe=lambda x:math.isfinite(x) and (x==0 or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000'
manifest={'phase':'fresh_after_candidate_freeze','seed':seed,'batches':[]}
def save(fn,rows):
 prior=set()
 for run in ['broad','extended-numeric','integer-publication','integer-refinement','integer-heldout','integer-thresholds','trunc-hyperbolic']:
  p=ROOT/f'smart-fuzzer/runs/w111-{run}-20260929/answers/answers-{fn.lower()}.json'
  if p.exists():prior.update(tuple(r['args']) for r in json.loads(p.read_text(encoding='utf-8-sig'))['witnesses'])
 rows=[r for r in dict.fromkeys(tuple(map(bits,row)) for row in rows if all(safe(x) for x in row)) if r not in prior]
 p=batch/('batch-'+fn.lower()+'.json');p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'w111inthyp-heldout-{fn}-{i:06d}','args':r}} for i,r in enumerate(rows)]},separators=(',',':')))
 manifest['batches'].append({'function':fn,'rows':len(rows),'path':p.name,'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'prior_inputs_excluded':len(prior)})
values=[]
for e in range(0,53):
 b=int(bits(2**e),16)
 for delta in range(-90,91):
  for sign in [-1,1]:values.append((sign*decode(b+delta),))
values.extend((decode((rng.randrange(2)<<63)|(rng.randrange(1,2047)<<52)|rng.getrandbits(52)),) for _ in range(14000))
save('INT',values)
rows=[]
for _ in range(12000):
 x=decode((rng.randrange(2)<<63)|(rng.randrange(1,2047)<<52)|rng.getrandbits(52))
 d=rng.choice([rng.uniform(-350,350),rng.choice([-1,1])*(2**31+rng.uniform(-350,350)),rng.choice([-1,1])*rng.uniform(2**32,1e12)])
 rows.append((x,d))
for fn in ['TRUNC','ROUNDDOWN','ROUNDUP','ROUND']:save(fn,rows)
values=[]
for c in [.125,.25,.5,1.,2.,20.,355.,709.782712893384,710.475860073944]:
 b=int(bits(c),16)
 for delta in range(-180,181):
  for sign in [-1,1]:values.append((sign*decode(b+delta),))
values.extend((rng.uniform(-3,3),) for _ in range(5000))
values.extend((rng.uniform(-750,750),) for _ in range(5000))
values.extend((decode((rng.randrange(2)<<63)|(rng.randrange(1,2047)<<52)|rng.getrandbits(52)),) for _ in range(7000))
for fn in ['SINH','COSH','CSCH','TANH','COTH']:save(fn,values)
(batch/'manifest.json').write_text(json.dumps(manifest,indent=2));print([(x['function'],x['rows']) for x in manifest['batches']])
