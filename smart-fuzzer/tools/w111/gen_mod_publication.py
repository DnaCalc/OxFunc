"""Exact normal-input MOD publication and quotient-boundary probes."""
import argparse,hashlib,json,math,random,struct
from pathlib import Path
from datetime import datetime,timezone
p=argparse.ArgumentParser();p.add_argument('--stage',choices=['discovery','heldout'],required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];out=root/f'smart-fuzzer/runs/w111-mod-publication-{a.stage}-20260929';out.mkdir(parents=True,exist_ok=True);assert not (out/'manifest.json').exists()
seed=202609293301+(a.stage=='heldout');rng=random.Random(seed)
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex();decode=lambda b:struct.unpack('>d',int(b).to_bytes(8,'big'))[0]
safe=lambda x:math.isfinite(x) and (x==0. or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000'
rows=[]
for _ in range(12000):
    x=math.ldexp(rng.uniform(1,2),rng.randrange(-1022,1023))*rng.choice([-1,1]);y=math.ldexp(rng.uniform(1,2),rng.randrange(-1022,1023))*rng.choice([-1,1]);rows.append((x,y))
for _ in range(6000):
    d=math.ldexp(rng.uniform(1,2),rng.randrange(-1000,980))*rng.choice([-1,1]);q=rng.choice([-1,1])*rng.uniform(0,1.2e12);x=d*q
    rows.append((x,d))
for _ in range(1000):
    d=math.ldexp(rng.uniform(1,2),rng.randrange(-900,900))*rng.choice([-1,1]);x=d*rng.randrange(-10000,10001)
    rows.extend((v,d) for v in [x,math.nextafter(x,-math.inf),math.nextafter(x,math.inf)])
for d in [1.,-1.,.1,-.1,math.pi,2.**-500,2.**500]:
    for q in [1125900000000.,2.**40,2.**41]:
        for sign in [-1,1]:
            x=d*q*sign;b=int(bits(abs(x)),16)
            rows.extend((math.copysign(decode(b+i),x),d) for i in range(-20,21))
prior=set()
if a.stage=='heldout':prior={tuple(r['args']) for r in json.loads((root/'smart-fuzzer/runs/w111-mod-publication-discovery-20260929/answers-mod.json').read_text(encoding='utf-8-sig'))['witnesses']}
encoded=[r for r in dict.fromkeys(tuple(map(bits,row)) for row in rows if all(safe(x) for x in row)) if r not in prior]
source=root/'crates/oxfunc_core/src/functions/mod_fn.rs';(out/'candidate.rs').write_bytes(source.read_bytes())
p=out/'batch-mod.json';p.write_text(json.dumps({'function':'MOD','probes':[{'probe':{'id':f'w111mod-{a.stage}-{i:05d}','args':r}} for i,r in enumerate(encoded)]},separators=(',',':')))
(out/'manifest.json').write_text(json.dumps({'seed':seed,'stage':a.stage,'rows':len(encoded),'generated_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'batch_sha256':hashlib.sha256(p.read_bytes()).hexdigest()},indent=2));print(len(encoded))
