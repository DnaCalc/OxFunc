"""Independent seeded MROUND controls, generated after the empirical graph freeze."""
from pathlib import Path
from fractions import Fraction
import datetime, hashlib, json, math, random, struct, sys

out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
freeze=Path('docs/function-lane/evidence/w111-broad-20260929/mround/candidate-freeze.json')
frozen=json.loads(freeze.read_text())
for path,digest in frozen['sources'].items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,path
rng=random.Random(2026092967)
bits=lambda x:'0x%016x'%struct.unpack('<Q',struct.pack('<d',float(x)))[0]
frombits=lambda b:struct.unpack('<d',struct.pack('<Q',b))[0]
tiny=frombits(0x0010000000000000)
rows={}
def emit(x,m,kind):
    if all(math.isfinite(v) and (v==0 or abs(v)>=tiny) for v in (x,m)):
        rows.setdefault((bits(x if x else 0.),bits(m if m else 0.)),kind)
cutoff=frombits(0x3fdfffffffffffa6)
for i in range(192):
    m=math.ldexp(rng.uniform(1.,2.),rng.randrange(-900,901))
    counts=[0,rng.randrange(1,7),rng.randrange(7,33),rng.randrange(33,129),rng.randrange(2**10,2**40)]
    for k in counts:
        # Exact rational composition avoids an unrecorded intermediate binary
        # addition before selecting the input center.
        center=float(Fraction.from_float(m)*(k+Fraction.from_float(cutoff)))
        values=[center]
        for direction in [-math.inf,math.inf]:
            value=center
            for _ in range(3):
                value=math.nextafter(value,direction);values.append(value)
        for value in values:
            for sign in [-1,1]:emit(sign*value,sign*m,'fresh-cutoff-neighborhood')
for _ in range(1600):
    x=frombits(rng.randrange(0x0010000000000000,0x7ff0000000000000))
    m=frombits(rng.randrange(0x0010000000000000,0x7ff0000000000000))
    sign=rng.choice([-1,1]);emit(sign*x,sign*m,'fresh-full-bit-normal')
    if rng.randrange(4)==0:emit(sign*x,-sign*m,'fresh-sign-mismatch')
for value in [tiny,math.nextafter(tiny,math.inf),1.,2.,math.pi,1e-100,1e100,
              frombits(0x7feffffffffffffe),frombits(0x7fefffffffffffff)]:
    for multiple in [0.,tiny,math.nextafter(tiny,math.inf),1.,3.,frombits(0x7fefffffffffffff)]:
        for sign in [-1,1]:emit(sign*value,sign*multiple,'stable-domain-publication')
    emit(0.,value,'stable-zero');emit(0.,-value,'stable-zero')
probes=[dict(probe=dict(id=f'mround-cutoff-heldout-{i:05d}-{kind}',args=list(args)))
        for i,(args,kind) in enumerate(rows.items())]
name='batch-mround.json'
(out/name).write_text(json.dumps(dict(function='MROUND',probes=probes),indent=2))
manifest=dict(generator=Path(__file__).name,seed=2026092967,
    generated_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
    candidate_freeze_sha256=hashlib.sha256(freeze.read_bytes()).hexdigest(),
    selection='192 fresh scales with seven source-ULP neighbors at five quotient ranges, both signs; 1600 full-bit normal pairs; domain/publication controls',
    batches=[dict(function='MROUND',path=name,rows=len(probes))])
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));print(json.dumps(manifest))
