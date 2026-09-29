"""Fresh candidate-frozen arithmetic validation; prior exact tuples excluded."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=root/'smart-fuzzer/runs/w111-log-power-midpoint-heldout-20260929'
out.mkdir(parents=True,exist_ok=True);assert not (out/'candidate-freeze.json').exists()
seed=202609293101;rng=random.Random(seed);sources={}
for name in ['functions/log_fn.rs','functions/power_fn.rs','functions/fisher_fn.rs','functions/median_fn.rs','excel_numeric/mod.rs','excel_numeric/x87.rs','coercion.rs']:
    p=root/'crates/oxfunc_core/src'/name;q=out/'candidate'/name;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(p.read_bytes());sources[name]=hashlib.sha256(p.read_bytes()).hexdigest()
(out/'candidate-freeze.json').write_text(json.dumps({'seed':seed,'frozen_utc':datetime.now(timezone.utc).isoformat(),'sources':sources},indent=2))
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
decode=lambda b:struct.unpack('>d',int(b).to_bytes(8,'big'))[0]
safe=lambda x:math.isfinite(x) and (x==0. or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000'
manifest={'seed':seed,'purpose':'independent_after_candidate_freeze','batches':[]}
def save(fn,rows):
    prior=set()
    for location in [f'w111-log-power-midpoint-20260929/answers-{fn.lower()}.json',f'w111-broad-20260929/answers/answers-{fn.lower()}.json',f'w111-extended-numeric-20260929/answers/answers-{fn.lower()}.json']:
        p=root/'smart-fuzzer/runs'/location
        if p.exists():prior.update(tuple(r['args']) for r in json.loads(p.read_text(encoding='utf-8-sig'))['witnesses'])
    encoded=[r for r in dict.fromkeys(tuple(map(bits,row)) for row in rows if all(safe(x) for x in row)) if r not in prior]
    p=out/f'batch-{fn.lower()}.json';p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'w111lpm-heldout-{fn}-{i:05d}','args':row}} for i,row in enumerate(encoded)]},separators=(',',':')))
    manifest['batches'].append({'function':fn,'rows':len(encoded),'path':p.name,'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'prior_tuples':len(prior)})
rows=[(rng.uniform(-30,30),float(rng.randrange(-500,501))) for _ in range(16000)]
rows.extend((decode((rng.randrange(1,2047)<<52)|rng.getrandbits(52)),rng.choice([.5,-.5,.25,-.25,1.,-1.,2.,-2.,rng.uniform(-9,9)])) for _ in range(7000))
for e in [2.**31,2.**32,2.**53,2.**63,1e20,1e100]:
    for v in [e,math.nextafter(e,0),math.nextafter(e,math.inf),-e]:
        rows.extend((b,v) for b in [0.,1.,-1.,2.,-2.,.5,-.5,10.,-10.,math.nextafter(1.,0),math.nextafter(1.,2)])
save('POWER',rows)
rows=[]
for _ in range(9000):
    n=decode((rng.randrange(1,2047)<<52)|rng.getrandbits(52));b=decode((rng.randrange(1,2047)<<52)|rng.getrandbits(52));rows.append((n,b))
    if rng.random()<.3:rows.append((1.,b))
for _ in range(1000):rows.append((decode(0x3ff0000000000000+rng.randrange(-250,251)),decode(0x3ff0000000000000+rng.randrange(-250,251))))
save('LOG',rows)
values=[rng.uniform(-1.2,1.2) for _ in range(9000)]
values.extend(decode((rng.randrange(2)<<63)|(rng.randrange(1,1024)<<52)|rng.getrandbits(52)) for _ in range(6000))
for center in [2**-53,2**-52,.5,1.,1e-10,1e-8]:
    b=int(bits(center),16)
    for delta in range(-500,501):values.extend([decode(b+delta),-decode(b+delta)])
save('FISHER',[(x,) for x in values])
rows=[]
for _ in range(22000):
    x=math.ldexp(rng.uniform(1,2),rng.randrange(-1022,1023));y=x*rng.uniform(-1.2,1.2);rows.append((x,y))
    if rng.random()<.3:rows.append((x,math.nextafter(-x,rng.choice([-math.inf,math.inf]))))
rows.extend(tuple(rng.uniform(-1e12,1e12) for _ in range(rng.randrange(1,22))) for _ in range(3000))
save('MEDIAN',rows)
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));print([(r['function'],r['rows']) for r in manifest['batches']])
