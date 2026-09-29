"""Fresh seeded POWER width controls, generated only after a candidate freeze."""
from pathlib import Path
import json,math,random,struct,sys
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
rng=random.Random(2026092948)
bits=lambda x:'0x%016x'%struct.unpack('<Q',struct.pack('<d',float(x)))[0]
floatbits=lambda x:struct.unpack('<d',struct.pack('<Q',x))[0]
rows={}
def emit(x,y,kind):rows.setdefault((bits(x),bits(y)),kind)
bases=[]
for _ in range(48):
    offset=rng.randrange(-2**28,2**28)
    value=floatbits(0x3ff0000000000000+offset);bases.extend([value,-value])
for i,base in enumerate(bases):
    counts=[rng.randrange(2**31-2048,2**31+2048),rng.randrange(2**32-2048,2**32+2048),
            rng.randrange(2**31,2**32-1),rng.randrange(2**32,2**35),
            rng.randrange(2**30,2**34)+.75]
    counts.extend([2**32-2,2**32-1,2**32,2**31-1,2**31,2**31+1])
    if i%3==0:
        counts.extend([math.nextafter(2**32-1,-math.inf),math.nextafter(2**32-1,math.inf),2**53,2**63])
    for y in counts:
        for sign in [-1,1]:emit(base,sign*y,'fresh-near-one-width-controls')
for _ in range(240):
    base=rng.uniform(.125,12.)*rng.choice([-1,1]);exponent=rng.randrange(2**31-10000,2**32+10000)
    emit(base,exponent,'fresh-general-base');emit(base,-exponent,'fresh-general-base')
for base in [10.,-10.,0.,1.,-1.]:
    for y in [2**31-1,2**31,2**31+1,2**32-309,2**32-308,2**32-307,2**32-23,2**32-4,2**32-3,2**32-2,2**32-1,2**32,2**32+1]:
        for sign in [-1,1]:emit(base,sign*y,'stable-branch-control')
probes=[dict(probe=dict(id=f'power-width-heldout-{i:05d}-{kind}',args=list(args))) for i,(args,kind) in enumerate(rows.items())]
name='batch-power.json';(out/name).write_text(json.dumps(dict(function='POWER',probes=probes),indent=2))
manifest=dict(generator=Path(__file__).name,seed=2026092948,selection='fresh mathematical width and near-one controls plus stable branch controls',
    batches=[dict(function='POWER',path=name,rows=len(probes))])
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));print(json.dumps(manifest))
