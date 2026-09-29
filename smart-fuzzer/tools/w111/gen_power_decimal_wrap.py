"""Discriminate the signed32 decimal-power scale after bounded magnitude conversion."""
import json,struct,sys
from pathlib import Path
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
bits=lambda x:'0x%016x'%struct.unpack('<Q',struct.pack('<d',float(x)))[0]
rows={}
for scale in range(-400,0):
    for exponent in [2**32+scale, -(2**32+scale), scale, -scale]:
        rows[(bits(10.),bits(exponent))]='wrapped-and-direct-scale'
for delta in [-2,-1,0,1,2,127,255,65535,2**30-1]:
    for exponent in [2**31+delta,-(2**31+delta)]:
        for base in [10.,-10.,1.+2.**-32,1.-2.**-32]:
            rows[(bits(base),bits(exponent))]='width-interior-control'
probes=[dict(probe=dict(id=f'power-decimal-wrap-{i:05d}-{kind}',args=list(args))) for i,(args,kind) in enumerate(rows.items())]
name='batch-power.json';(out/name).write_text(json.dumps(dict(function='POWER',probes=probes),indent=2))
manifest=dict(generator=Path(__file__).name,selection='full signed decimal tail -400..-1 and direct-scale controls',batches=[dict(function='POWER',path=name,rows=len(probes))])
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));print(json.dumps(manifest))
