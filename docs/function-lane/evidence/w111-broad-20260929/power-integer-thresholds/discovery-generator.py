"""Discriminate POWER's integer dispatch widths and decimal exponent conversion.

This discovery selects exact-width boundaries and mathematical controls only;
it does not consume Excel outputs or tune numerical coefficients.
"""
import json, math, random, struct, sys
from pathlib import Path

out = Path(sys.argv[1]); out.mkdir(parents=True,exist_ok=True)
bits = lambda x: '0x%016x' % struct.unpack('<Q',struct.pack('<d',float(x)))[0]
rng=random.Random(2026092947)
bases={0., 1., -1., .5, -.5, 2., -2., 10., -10.}
for center in [1.,-1.,10.,-10.]:
    for direction in [-math.inf,math.inf]:
        value=center
        for _ in range(2):value=math.nextafter(value,direction);bases.add(value)
for k in [23,29,31,37,43,49]:
    for sign in [-1,1]:
        value=1+sign*2.**-k;bases.update([value,-value])
counts=set()
for center in [32767.,65535.,2147483647.,2147483648.,4294967294.,4294967295.,4294967296.,4294967297.,9007199254740992.]:
    for offset in [-2.,-1.,-.5,-.25,0.,.25,.5,1.,2.]:counts.add(center+offset)
    for direction in [-math.inf,math.inf]:
        value=center
        for _ in range(3):value=math.nextafter(value,direction);counts.add(value)
counts.update(float(rng.randrange(2**31-256,2**32+256)) for _ in range(32))
rows={}
for base in sorted(bases):
    for magnitude in sorted(counts):
        for sign in [-1.,1.]:
            row=(bits(base),bits(sign*magnitude));rows[row]='width-boundary-and-control'
probes=[dict(probe=dict(id=f'power-width-discovery-{i:05d}',args=list(row))) for i,row in enumerate(rows)]
name='batch-power.json'
(out/name).write_text(json.dumps(dict(function='POWER',probes=probes),indent=2))
manifest=dict(generator=Path(__file__).name,seed=2026092947,bases=len(bases),magnitudes=len(counts),
    selection='signed32/unsigned32 boundaries, exact neighboring floats, independent integer controls',
    batches=[dict(function='POWER',path=name,rows=len(probes))])
(out/'manifest.json').write_text(json.dumps(manifest,indent=2))
print(json.dumps(manifest))
