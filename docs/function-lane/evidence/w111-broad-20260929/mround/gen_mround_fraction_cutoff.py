"""Measure MROUND's fractional half boundary without selecting a fitted cutoff."""
import json,math,struct,sys
from pathlib import Path
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
bits=lambda x:'0x%016x'%struct.unpack('<Q',struct.pack('<d',float(x)))[0]
rows={}
for m in [1.,2.**-500,2.**500,.1,math.pi,1e-100]:
    for k,radius in [(0,128),(1,40),(3,24),(7,16),(15,8),(31,4),(32,4)]:
        center=m*(k+.5);values=[center]
        for direction in [-math.inf,math.inf]:
            x=center
            for _ in range(radius):x=math.nextafter(x,direction);values.append(x)
        for x in values:
            for sign in [-1,1]:rows.setdefault((bits(sign*x),bits(sign*m)),None)
probes=[dict(probe=dict(id=f'mround-fraction-cutoff-{i:05d}',args=list(args))) for i,args in enumerate(rows)]
name='batch-mround.json';(out/name).write_text(json.dumps(dict(function='MROUND',probes=probes),indent=2))
manifest=dict(generator=Path(__file__).name,selection='deep ULP sweep around half fractions, seven quotients and six scales, both signs',batches=[dict(function='MROUND',path=name,rows=len(probes))])
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));print(json.dumps(manifest))
