"""Fresh ATAN holdout after extended inverse-angle reduction; math midpoint inputs."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
import mpmath as mp
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/runs/w111-atan-heldout2-20260929';out.mkdir(exist_ok=True)
freeze=out/'candidate-freeze.json';assert not freeze.exists()
sources=['functions/atan.rs','excel_numeric/mod.rs','excel_numeric/x87.rs','coercion.rs','coercion_decimal.rs','functions/adapters.rs']
hashes={}
for name in sources:
 p=g.ROOT/'crates/oxfunc_core/src'/name;q=out/'candidate'/name;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(p.read_bytes());hashes[name]=hashlib.sha256(p.read_bytes()).hexdigest()
seed=202609292011;rng=random.Random(seed)
freeze.write_text(json.dumps({'frozen_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':hashes,'seed':seed,'phase':'independent_after_extended_inverse_reduction'},indent=2),encoding='utf-8')
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
decode=lambda x:struct.unpack('>d',int(x).to_bytes(8,'big'))[0]
prior=set()
for name in ['discovery','heldout','refinement']:
 prior.update(w['args'][0] for w in json.loads((g.ROOT/f'docs/function-lane/evidence/w111-broad-20260929/atan/{name}.json').read_text())['witnesses'])
rows={}
def add(x,region):
 if math.isfinite(x) and (x==0 or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000' and bits(x) not in prior:rows.setdefault(bits(x),region)
for _ in range(40000):add(decode((rng.randrange(2)<<63)|(rng.randrange(1,2047)<<52)|rng.getrandbits(52)),'fresh-full-normal-exponents')
for _ in range(12000):add(rng.uniform(-16,16),'fresh-ordinary-magnitudes')
mp.mp.dps=110
for i in range(5000):
 a=rng.uniform(.000001,float(mp.pi/2))
 midpoint=(mp.mpf(a)+mp.mpf(math.nextafter(a,math.inf)))/2
 x=float(mp.tan(midpoint))
 if x<=0 or not math.isfinite(x):continue
 key=int(bits(x),16)
 for delta in range(-2,3):
  for sign in [-1,1]:add(sign*decode(key+delta),'fresh-tan-of-output-midpoint-neighbors')
for _ in range(1200):
 x=math.ldexp(rng.uniform(1,2),rng.choice([-1022,-54,-53,-52,-27,-26,-1,0,1,26,27,52,53,54,1000]))
 for sign in [-1,1]:add(sign*x,'fresh-selected-exponent-boundaries')
for center in [0x3fefffffffffffff,0x3ff0000000000000]:
 for delta in range(-2048,2049):
  for sign in [-1,1]:add(sign*decode(center+delta),'fresh-unit-branch-neighbors')
packet={'function':'ATAN','probes':[{'probe':{'id':f'atan-heldout2-{i:06d}','args':[x]},'probe_region':region} for i,(x,region) in enumerate(rows.items())]}
(out/'batch.json').write_text(json.dumps(packet,separators=(',',':')),encoding='utf-8')
(out/'manifest.json').write_text(json.dumps({'numeric_rows':len(rows),'seed':seed,'excluded_prior_inputs':len(prior),'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'batch_sha256':hashlib.sha256((out/'batch.json').read_bytes()).hexdigest(),'mpmath_version':mp.__version__,'mpmath_decimal_precision':mp.mp.dps,'math_use':'Select fresh inputs near mathematical output midpoints only; Excel remains the result oracle','ingress':'normal binary64 and positive zero, exact Value2 readback'},indent=2),encoding='utf-8')
print('Independent ATAN numeric rows:',len(rows))
