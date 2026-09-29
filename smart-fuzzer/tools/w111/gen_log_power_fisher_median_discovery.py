"""Broad-sweep follow-up: publication, integer powers and stored midpoints."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=root/'smart-fuzzer/runs/w111-log-power-midpoint-20260929'
out.mkdir(parents=True,exist_ok=True);assert not (out/'manifest.json').exists()
seed=202609293100;rng=random.Random(seed)
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
decode=lambda b:struct.unpack('>d',int(b).to_bytes(8,'big'))[0]
safe=lambda x:math.isfinite(x) and (x==0. or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000'
manifest={'seed':seed,'purpose':'discovery_after_broad_sweep_mismatches','generated_utc':datetime.now(timezone.utc).isoformat(),'batches':[]}
for fn in ['log_fn.rs','power_fn.rs','fisher_fn.rs','median_fn.rs']:
    p=root/'crates/oxfunc_core/src/functions'/fn;q=out/'baseline-source'/fn;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(p.read_bytes())
def save(fn,rows):
    encoded=list(dict.fromkeys(tuple(map(bits,row)) for row in rows if all(safe(x) for x in row)))
    p=out/f'batch-{fn.lower()}.json';p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'w111lpm-{fn}-{i:05d}','args':row}} for i,row in enumerate(encoded)]},separators=(',',':')))
    manifest['batches'].append({'function':fn,'rows':len(encoded),'path':p.name,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
rows=[(b,n) for b in [10.,-10.,.1,-.1,2.,-2.] for n in range(-1100,1101)]
rows.extend((rng.uniform(-20,20),float(rng.randrange(-200,201))) for _ in range(4000))
rows.extend((decode((rng.randrange(1,2047)<<52)|rng.getrandbits(52)),rng.choice([.5,-.5,.25,-.25,1.,-1.,2.,-2.,rng.uniform(-5,5)])) for _ in range(3000))
save('POWER',rows)
rows=[]
for _ in range(4000):
    n=decode((rng.randrange(1,2047)<<52)|rng.getrandbits(52));b=decode((rng.randrange(1,2047)<<52)|rng.getrandbits(52));rows.extend([(n,b),(1.,b)])
rows.extend((n,b) for n in [0.,-1.,1.,math.nextafter(1.,0),math.nextafter(1.,2)] for b in [-1.,0.,1.,math.nextafter(1.,0),math.nextafter(1.,2),2.,10.])
save('LOG',rows)
values=[rng.uniform(-1.1,1.1) for _ in range(3500)]
for e in range(1,1023):values.extend([2.**-e,-2.**-e])
for center in [1.,.5,2**-53,2**-52,2**-26,1e-8,1e-10]:
    b=int(bits(center),16)
    for offset in range(-100,101):values.extend([decode(b+offset),-decode(b+offset)])
save('FISHER',[(x,) for x in values])
rows=[]
for _ in range(7000):
    x=math.ldexp(rng.uniform(1,2),rng.randrange(-1022,1023));y=x*rng.uniform(-1.1,1.1)
    rows.append((x,y))
    if rng.random()<.3:rows.append((x,math.nextafter(-x,rng.choice([-math.inf,math.inf]))))
rows.extend(tuple(rng.uniform(-1e8,1e8) for _ in range(rng.randrange(1,16))) for _ in range(1500))
save('MEDIAN',rows)
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));print([(r['function'],r['rows']) for r in manifest['batches']])
