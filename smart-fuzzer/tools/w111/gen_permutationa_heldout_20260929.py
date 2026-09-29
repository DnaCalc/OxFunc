"""Validate the frozen PERMUTATIONA admission, integer power and base-ten candidate."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3];out=ROOT/'smart-fuzzer/runs/w111-permutationa-heldout-20260929';out.mkdir(parents=True,exist_ok=True)
assert not (out/'candidate-freeze.json').exists();seed=202609292802;rng=random.Random(seed)
p=ROOT/'crates/oxfunc_core/src/functions/permutationa_fn.rs';(out/'candidate.rs').write_bytes(p.read_bytes());(out/'candidate-freeze.json').write_text(json.dumps({'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'frozen_utc':datetime.now(timezone.utc).isoformat(),'seed':seed},indent=2))
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex();decode=lambda x:struct.unpack('>d',int(x).to_bytes(8,'big'))[0]
prior=set()
for run in ['extended-numeric','integer-publication','integer-thresholds']:
 p=ROOT/f'smart-fuzzer/runs/w111-{run}-20260929/answers/answers-permutationa.json'
 if p.exists():prior.update(tuple(r['args']) for r in json.loads(p.read_text(encoding='utf-8-sig'))['witnesses'])
rows=[(n,k) for n in range(31) for k in range(321)]
rows.extend((10+rng.random(),k+rng.random()) for k in range(321))
rows.extend((10**n,k) for n in range(1,10) for k in range(321))
rows.extend((rng.randrange(0,100000)+rng.random(),rng.randrange(0,320)+rng.random()) for _ in range(10000))
for c in [0.,1.,2147483647.]:
 b=int(bits(c),16)
 for delta in range(-50,51):
  if b+delta<=0:continue
  x=decode(b+delta)
  if abs(x)<2**-1022:continue
  for s in [-1,1]:
   rows.extend((s*x,k) for k in [0,1,2,.75,-.75]);rows.extend((n,s*x) for n in [0,.75,1,2,10])
rows=[r for r in dict.fromkeys(tuple(map(bits,r)) for r in rows) if r not in prior and all(int(x,16)!=0x8000000000000000 for x in r)]
p=out/'batch-permutationa.json';p.write_text(json.dumps({'function':'PERMUTATIONA','probes':[{'probe':{'id':f'w111permutationa-heldout-{i:06d}','args':r}} for i,r in enumerate(rows)]},separators=(',',':')))
(out/'manifest.json').write_text(json.dumps({'seed':seed,'rows':len(rows),'prior_inputs_excluded':len(prior),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()},indent=2));print(len(rows),'fresh PERMUTATIONA inputs')
