"""Probe engineering places validation and BASE bounds through Value2."""
import json, math, random, struct, sys
from pathlib import Path
def bits(v):return '0x%016x'%struct.unpack('<Q',struct.pack('<d',float(v)))[0]
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
seed=int(sys.argv[2]) if len(sys.argv)>2 else 20260929
def save(fn,rows):
    data={'function':fn,'probes':[{'probe':{'id':f'radix-{seed}-{fn}-{i:05d}',
        'args':[bits(v) for v in row]}} for i,row in enumerate(rows)]}
    (out/('batch-'+fn.lower()+'.json')).write_text(json.dumps(data),encoding='utf-8')
places=[-1,-.9,0,.1,.9,1,1.9,9.9,10,math.nextafter(11,0),11,11.1,255,256,1e10]
for fn,low,high in [('DEC2BIN',-512,511),('DEC2OCT',-2**29,2**29-1),('DEC2HEX',-2**39,2**39-1)]:
    r=random.Random(f'{seed}-{fn}')
    values=[low-1,low,math.nextafter(float(low),-math.inf),low+.5,-1,-.9,0,.9,1,high,high+.9,high+1]
    rows=[(v,p) for v in values for p in places]
    rows += [(r.uniform(low-1,high+2),r.uniform(-2,12)) for _ in range(1500)]
    save(fn,rows)
for fn in ['BIN2HEX','BIN2OCT','OCT2BIN','OCT2HEX','HEX2BIN','HEX2OCT']:
    save(fn,[(v,p) for v in [0,1,10,1111111111,7777777777,9999999999] for p in places])
rows=[(v,rad,p) for v in [-1,-.9,0,.9,1,2**53-1,2**53,2**53+2] for rad in [1,2,10,36,37]
      for p in [-1,-.9,0,.9,1,10,255,256]]
rows += [(v,rad) for v in [0,.9,1,2**53-1,2**53] for rad in [2,10,36]]
save('BASE',rows)
print('Generated radix/BASE bound corpus',seed,out)
