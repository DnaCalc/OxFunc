"""Fresh INT validation plus independent integer-admission discriminators."""
import hashlib,json,math,random,struct
from pathlib import Path
from datetime import datetime,timezone
ROOT=Path(__file__).resolve().parents[3]
out=ROOT/'smart-fuzzer/runs/w111-integer-thresholds-20260929'; batch=out/'batches';batch.mkdir(parents=True,exist_ok=True)
assert not (out/'manifest.json').exists()
seed=202609292633;rng=random.Random(seed)
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
decode=lambda x:struct.unpack('>d',int(x).to_bytes(8,'big'))[0]
safe=lambda x:math.isfinite(x) and (x==0 or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000'
sources={}
for name in ['int_fn.rs','round_fn.rs','permutationa_fn.rs','trunc_fn.rs','rounddown_fn.rs']:
 p=ROOT/'crates/oxfunc_core/src/functions'/name;q=out/'candidate'/name;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(p.read_bytes());sources[name]=hashlib.sha256(p.read_bytes()).hexdigest()
(out/'candidate-freeze.json').write_text(json.dumps({'source_sha256':sources,'frozen_utc':datetime.now(timezone.utc).isoformat(),'seed':seed,'validation_only':['INT'],'discovery':['PERMUTATIONA','TRUNC','ROUND','ROUNDDOWN','ROUNDUP']},indent=2))
manifest={'seed':seed,'batches':[]}
def save(fn,rows,prior=set()):
 rows=[r for r in dict.fromkeys(tuple(map(bits,row)) for row in rows if all(safe(x) for x in row)) if r not in prior]
 p=batch/('batch-'+fn.lower()+'.json');p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'w111threshold-{fn}-{i:06d}','args':r}} for i,r in enumerate(rows)]},separators=(',',':')))
 manifest['batches'].append({'function':fn,'rows':len(rows),'path':p.name,'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'prior_inputs_excluded':len(prior)})
prior=set()
for run in ['broad','integer-publication','integer-refinement','integer-heldout']:
 p=ROOT/f'smart-fuzzer/runs/w111-{run}-20260929/answers/answers-int.json'
 if p.exists():prior.update(tuple(r['args']) for r in json.loads(p.read_text(encoding='utf-8-sig'))['witnesses'])
rows=[]
for center in [1e14,1e15,2**49,2**50,2**51,2**52]:
 b=int(bits(center),16)
 for delta in range(-400,401):
  for sign in [-1,1]:rows.append((sign*decode(b+delta),))
for _ in range(15000):
 x=decode((rng.randrange(2)<<63)|(rng.randrange(1023,1076)<<52)|rng.getrandbits(52));rows.append((x,))
save('INT',rows,prior)
controls=[]
for c in [0,1,2**15,2**16,2**30,2**31-1,2**31,2**32]:
 b=int(bits(c),16)
 for delta in range(-20,21):
  if b+delta<0:continue
  x=decode(b+delta)
  if safe(x):
   for sign in [-1,1]:
    controls.extend([(sign*x,k) for k in [0,1,2,.9,-.9]])
    controls.extend([(n,sign*x) for n in [0,.9,1,1.9,2]])
for n in [-.999,-.5,-1e-10,0,.5,1,2,3,7,10,13,100,1e4,1e9]:
 for k in [-2,-1.9,-1,-.999,-.5,0,.1,1,3,10,31,32,99,100,101,127,128,255,256,1000]:controls.append((n,k))
controls.extend((rng.randrange(0,10000)+rng.choice([0,.125,.999]),rng.randrange(0,1100)+rng.choice([0,.25,.75])) for _ in range(10000))
save('PERMUTATIONA',controls)
digits=[]
for center in [2**31,2**32,3*2**31,2**33,5*2**31,2**63]:
 for offset in range(-20,21):
  for frac in [0,.25,.75]:
   for sign in [-1,1]:digits.append(sign*(center+offset+frac))
rows=[(n,d) for n in [1.125,-1.125,12.875,-12.875,123456789012345.5] for d in digits]
for fn in ['TRUNC','ROUNDDOWN','ROUND','ROUNDUP']:save(fn,rows)
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));(batch/'manifest.json').write_text(json.dumps(manifest,indent=2))
print([(r['function'],r['rows']) for r in manifest['batches']])
