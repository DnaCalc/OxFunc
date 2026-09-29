"""Paired signs and arithmetic-neighbor controls for ASIN; no fitted answers."""
import hashlib,json,math,random,struct,sys
from pathlib import Path

SEED=2026092922
rng=random.Random(SEED)
bits=lambda x:struct.unpack('<Q',struct.pack('<d',x))[0]
number=lambda n:struct.unpack('<d',struct.pack('<Q',n))[0]
values={}
def add(raw,kind):
    if 0<=raw<=bits(1.0):
        n=number(raw)
        if n==0 or n>=sys.float_info.min:
            for sign in [0,1]:values.setdefault(raw|(sign<<63),kind)
for _ in range(1600):add(bits(rng.random()),'uniform-domain-pair')
for _ in range(800):add((rng.randint(1,1022)<<52)|rng.getrandbits(52),'scaled-domain-pair')
for x in [0.,sys.float_info.min,2**-53,2**-27,0.25,0.5,math.sqrt(0.5),0.75,1.]:
    for k in range(-12,13):add(bits(x)+k,'mathematical-boundary-neighbor')
# Previously observed asymmetry is labelled as diagnostic, not fresh validation.
for k in range(-256,257):add(0x3fe52a3a8df2eadb+k,'prior-asymmetry-neighbor')
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
rows=[dict(probe=dict(id=f'asin-sign-{SEED}-{i:05d}-{kind}',args=[f'0x{raw:016x}']))
      for i,(raw,kind) in enumerate(values.items())]
p=out/'batch-asin.json';p.write_text(json.dumps(dict(function='ASIN',probes=rows),indent=2),encoding='utf-8')
(out/'manifest.json').write_text(json.dumps(dict(generator=Path(__file__).name,seed=SEED,
 authority='non_semantic_exploration_input',batches=[dict(function='ASIN',path=p.name,rows=len(rows),
 sha256=hashlib.sha256(p.read_bytes()).hexdigest())]),indent=2),encoding='utf-8')
print(len(rows),'rows ->',out)
