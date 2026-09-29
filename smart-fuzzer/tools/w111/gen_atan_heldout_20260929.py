"""Independent ATAN bits and typed origins after the FPATAN candidate freeze."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
import gen_broad_typed_20260929 as g

out=g.ROOT/'smart-fuzzer/runs/w111-atan-heldout-20260929'
out.mkdir(parents=True,exist_ok=True)
freeze=out/'candidate-freeze.json'
if freeze.exists():raise SystemExit('Refusing to overwrite independent candidate freeze')
sources=['functions/atan.rs','excel_numeric/mod.rs','excel_numeric/x87.rs','coercion.rs','coercion_decimal.rs','functions/adapters.rs']
hashes={}
for name in sources:
 p=g.ROOT/'crates/oxfunc_core/src'/name;q=out/'candidate'/name;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(p.read_bytes());hashes[name]=hashlib.sha256(p.read_bytes()).hexdigest()
seed=202609291803
freeze.write_text(json.dumps({'frozen_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':hashes,'seed':seed,'phase':'independent_after_1224_discovery_candidate'},indent=2),encoding='utf-8')
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
decode=lambda x:struct.unpack('>d',int(x).to_bytes(8,'big'))[0]
prior={w['args'][0] for w in json.loads((g.ROOT/'docs/function-lane/evidence/w111-broad-20260929/atan/discovery.json').read_text())['witnesses']}
rows={};rng=random.Random(seed)
def add(x,region):
 if math.isfinite(x) and (x==0 or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000' and bits(x) not in prior:rows.setdefault(bits(x),region)
for _ in range(18000):add(decode((rng.randrange(2)<<63)|(rng.randrange(1,2047)<<52)|rng.getrandbits(52)),'full-normal-exponents')
for _ in range(6000):add(rng.uniform(-8,8),'ordinary-magnitudes')
for exponent in [-1022,-1000,-512,-54,-53,-52,-28,-27,-26,-4,-2,-1,0,1,2,4,26,27,28,52,53,54,512,1000,1023]:
 for sign in [-1,1]:
  x=sign*math.ldexp(1.,exponent)
  for direction in [-math.inf,math.inf]:
   y=x
   for _ in range(25):add(y,'power-boundary-neighbors');y=math.nextafter(y,direction)
for x in [math.sqrt(2)-1,math.sqrt(2)+1,math.sqrt(3),1/math.sqrt(3),.4375,.6875,1.1875,2.4375]:
 for sign in [-1,1]:
  for direction in [-math.inf,math.inf]:
   y=sign*x
   for _ in range(100):add(y,'published-range-reduction-neighbors');y=math.nextafter(y,direction)
packet={'function':'ATAN','probes':[{'probe':{'id':f'w111atan-heldout-{i:05d}','args':[x]},'probe_region':region} for i,(x,region) in enumerate(rows.items())]}
(out/'batch.json').write_text(json.dumps(packet,separators=(',',':')),encoding='utf-8')
g.TRANCHE='w111-atan-heldout-20260929'
pool=[g.t(x) for x in ['','0','-1','1e-300','1e300','50%','(2)',' 1 ','\t1','TRUE','FALSE','x']]+[g.n(x) for x in [-1e300,-1,-.125,0,.125,1,1e300]]+[g.b(False),g.b(True)]+[g.e(e) for e in g.ERR]
for i,value in enumerate(pool):
 g.emit('ATAN',f'direct-{i}',[value])
 g.emit('ATAN',f'reference-{i}',[g.r('B1')],[g.fix('B1',value)])
g.emit('ATAN','blank-reference',[g.r('B1')],[g.fix('B1',g.blank())])
for i in range(240):
 h,w=rng.choice([(1,4),(2,3),(4,1),(3,2)])
 a=g.a([[rng.choice(pool) for _ in range(w)] for _ in range(h)])
 if i%2:g.emit('ATAN',f'array-{i}',[a])
 else:
  address=f'B1:{chr(65+w)}{h}';g.emit('ATAN',f'reference-array-{i}',[g.r(address)],[g.fix(address,a)])
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111atan-')
(out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
(out/'manifest.json').write_text(json.dumps({'numeric_rows':len(rows),'typed_rows':len(g.cases),'excluded_discovery_values':len(prior),'seed':seed,'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'batch_sha256':hashlib.sha256((out/'batch.json').read_bytes()).hexdigest(),'typed_sha256':hashlib.sha256((out/'typed.json').read_bytes()).hexdigest(),'ingress':'normal binary64 and positive zero; Value2 exact readback; prior discovery excluded'},indent=2),encoding='utf-8')
print(len(rows),'numeric;',len(g.cases),'typed')
