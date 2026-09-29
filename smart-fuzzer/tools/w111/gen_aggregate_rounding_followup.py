"""Normal-input follow-up for aggregate store graphs and MROUND publication."""
import argparse,hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--stage',choices=['discovery','heldout'],required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];out=root/f'smart-fuzzer/runs/w111-aggregate-rounding-{a.stage}-20260929'
out.mkdir(parents=True,exist_ok=True);assert not (out/'manifest.json').exists()
seed=202609293201+(a.stage=='heldout');rng=random.Random(seed)
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
decode=lambda b:struct.unpack('>d',int(b).to_bytes(8,'big'))[0]
safe=lambda x:math.isfinite(x) and (x==0. or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000'
manifest={'seed':seed,'stage':a.stage,'generated_utc':datetime.now(timezone.utc).isoformat(),'batches':[],'source_sha256':{}}
for name in ['harmean_fn.rs','devsq_fn.rs','mround.rs']:
    p=root/'crates/oxfunc_core/src/functions'/name;q=out/'candidate'/name;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(p.read_bytes());manifest['source_sha256'][name]=hashlib.sha256(p.read_bytes()).hexdigest()
def save(fn,rows):
    prior=set()
    if a.stage=='heldout':
        p=root/f'smart-fuzzer/runs/w111-aggregate-rounding-discovery-20260929/answers-{fn.lower()}.json'
        prior={tuple(r['args']) for r in json.loads(p.read_text(encoding='utf-8-sig'))['witnesses']}
    encoded=[r for r in dict.fromkeys(tuple(map(bits,row)) for row in rows if all(safe(x) for x in row)) if r not in prior]
    p=out/f'batch-{fn.lower()}.json';p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'w111agg-{a.stage}-{fn}-{i:05d}','args':row}} for i,row in enumerate(encoded)]},separators=(',',':')))
    manifest['batches'].append({'function':fn,'rows':len(encoded),'path':p.name,'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'prior_tuples':len(prior)})
for fn in ['HARMEAN','DEVSQ']:
    rows=[]
    for _ in range(4000):
        length=rng.randrange(1,33);e=rng.randrange(-1022,1020)
        rows.append(tuple(math.ldexp(rng.uniform(1,2),e+rng.randrange(0,min(3,1024-e))) for _ in range(length)))
    for _ in range(3000):
        length=rng.randrange(1,20)
        rows.append(tuple(math.ldexp(rng.uniform(1,2),rng.randrange(-1022,1023)) for _ in range(length)))
    for _ in range(2000):
        length=rng.randrange(1,20);e=rng.randrange(-500,500);b=int(bits(math.ldexp(rng.uniform(1,2),e)),16)
        rows.append(tuple(decode(b+rng.randrange(-100,101)) for _ in range(length)))
    rows.extend([(0.,),(1.,-1.),(-1.,), (1.,1.,1.)])
    if fn=='DEVSQ':rows.extend(tuple(x*rng.choice([-1,1]) for x in row) for row in rows[:2000])
    save(fn,rows)
rows=[]
for _ in range(7000):
    multiple=math.ldexp(rng.uniform(1,2),rng.randrange(-1022,1023));q=rng.uniform(0,1000);sign=rng.choice([-1,1]);rows.append((sign*multiple*q,sign*multiple))
for _ in range(3000):
    multiple=math.ldexp(rng.uniform(1,2),rng.randrange(-500,500));n=multiple*(rng.randrange(0,100000)+.5)
    for v in [n,math.nextafter(n,0),math.nextafter(n,math.inf)]:
        sign=rng.choice([-1,1]);rows.append((sign*v,sign*multiple))
rows.extend((x,y) for x in [0.,1.,-1.,2**-1022,-2**-1022,float.fromhex('0x1.fffffffffffffp1023')] for y in [0.,1.,-1.,2**-1022,-2**-1022,float.fromhex('0x1.fffffffffffffp1023')])
save('MROUND',rows)
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));print([(r['function'],r['rows']) for r in manifest['batches']])
