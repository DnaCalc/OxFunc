"""MROUND half-way controls across small/large quotients, scales and signs."""
import json,math,struct,sys
from pathlib import Path
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
bits=lambda x:'0x%016x'%struct.unpack('<Q',struct.pack('<d',float(x)))[0]
decode=lambda x:struct.unpack('<d',struct.pack('<Q',int(x,16)))[0]
rows={}
def emit(x,m,kind):
    if all(math.isfinite(v) and (v==0 or abs(v)>=2**-1022) for v in [x,m]):rows.setdefault((bits(x),bits(m)),kind)
counts=list(range(41))+[63,64,127,128,255,256,511,512,1023,1024,2047,2048]
for m in [.1,.3,1.,3.,7.,math.sqrt(2.),1e-100,1e100]:
    for k in counts:
        center=m*(k+.5);values=[center]
        for direction in [-math.inf,math.inf]:
            x=center
            for _ in range(4):x=math.nextafter(x,direction);values.append(x)
        for x in values:
            for sign in [-1,1]:emit(sign*x,sign*m,'half-boundary')
# Public discovery counterexamples and their neighboring inputs; these are
# labelled dependent controls, separate from the mathematical grid above.
for xb,mb in [('4285a46ce755024f','4240e43c05b8fb90'),('5d9aa77eefc958ef','5d73628adcef8680'),('320c47deaea45726','31cd3169fe992028')]:
    center,m=decode(xb),decode(mb);values=[center]
    for direction in [-math.inf,math.inf]:
        x=center
        for _ in range(8):x=math.nextafter(x,direction);values.append(x)
    for x in values:
        for sign in [-1,1]:emit(sign*x,sign*m,'retained-counterexample-neighbor')
probes=[dict(probe=dict(id=f'mround-half-discovery-{i:05d}-{kind}',args=list(args))) for i,(args,kind) in enumerate(rows.items())]
name='batch-mround.json';(out/name).write_text(json.dumps(dict(function='MROUND',probes=probes),indent=2))
manifest=dict(generator=Path(__file__).name,selection='mathematical quotient half-boundaries plus labelled retained counterexample neighbors',batches=[dict(function='MROUND',path=name,rows=len(probes))])
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));print(json.dumps(manifest))
