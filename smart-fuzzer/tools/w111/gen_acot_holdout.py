"""Independent ACOT normals plus general reciprocal double-rounding controls.

The seeded search selects disagreements between two mathematical reciprocal
graphs; it uses no Excel answers or coefficient-specific correction.
"""
import hashlib,json,math,random,struct,sys
from fractions import Fraction as F
from pathlib import Path
from gen_complex_midpoint_search_20260929 import round_binary

SEED=int(sys.argv[2]) if len(sys.argv)>2 else 2026092918
rng=random.Random(SEED)
bits=lambda x:struct.unpack('<Q',struct.pack('<d',x))[0]
number=lambda n:struct.unpack('<d',struct.pack('<Q',n))[0]
values={}
def add(x,kind):
    if math.isfinite(x) and (x==0 or abs(x)>=sys.float_info.min):values.setdefault(bits(x),kind)
def normal():return number((rng.getrandbits(1)<<63)|(rng.randint(1,2046)<<52)|rng.getrandbits(52))
for _ in range(2500):add(normal(),'normal-bits')
for _ in range(2500):add(rng.uniform(-1000,1000),'ordinary')
for _ in range(2500):add(rng.uniform(-4,4),'angle-curvature')
centers=0
for _ in range(100000):
    x=abs(normal())
    # Very tiny normal inputs overflow the binary64 reciprocal, a separate
    # boundary already covered by explicit scale controls below.
    if x<1/sys.float_info.max:continue
    rounded=float(round_binary(1/F(x),64))
    if rounded!=1/x:
        centers+=1
        for step in range(-2,3):
            for sign in [-1,1]:add(sign*number(bits(x)+step),'reciprocal-double-round')
for x in [0.,sys.float_info.min,sys.float_info.max,1/sys.float_info.min,1/sys.float_info.max,
          0.5,1.,2.,4.,1e-300,1e300,2**-53,2**53]:
    for step in range(-4,5):
        raw=bits(x)+step
        if 0<=raw<=0x7fefffffffffffff:
            for sign in [-1,1]:add(sign*number(raw),'boundary-neighbor')
# A published in-session ATAN discrepancy is an explicit diagnostic control,
# never presented as an independent random draw.
center=abs(1/number(0xc0000d685e592174))
for step in range(-5,6):
    for sign in [-1,1]:add(sign*number(bits(center)+step),'prior-atan-residual-control')
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
rows=[dict(probe=dict(id=f'acot-heldout-{SEED}-{i:05d}-{kind}',args=[f'0x{raw:016x}']))
      for i,(raw,kind) in enumerate(values.items())]
p=out/'batch-acot.json';p.write_text(json.dumps(dict(function='ACOT',probes=rows),indent=2),encoding='utf-8')
(out/'manifest.json').write_text(json.dumps(dict(generator=Path(__file__).name,seed=SEED,
 authority='non_semantic_exploration_input',reciprocal_search_draws=100000,reciprocal_centers=centers,
 batches=[dict(function='ACOT',path=p.name,rows=len(rows),sha256=hashlib.sha256(p.read_bytes()).hexdigest())]),indent=2),encoding='utf-8')
print(len(rows),'rows,',centers,'reciprocal centers ->',out)
