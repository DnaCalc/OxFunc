"""Probe SWITCH numeric comparison across decimal scaling overflow boundaries."""
import json
import math
import random
import struct
import sys
from pathlib import Path

def bits(x): return '0x%016x'%struct.unpack('<Q',struct.pack('<d',float(x)))[0]
rows=[]
for exponent in [-308,-307,-306,-300,-296,-295,-294,-293,-292,-10,0,10,290,307,308]:
    for mantissa in [1,1.23456789012345,5,9]:
        x=mantissa*10.0**exponent
        if not math.isfinite(x): continue
        for sign in [1,-1]:
            a=sign*x
            for b in [0,1,-1,a,-a,math.nextafter(a,math.inf),math.nextafter(a,-math.inf),a*(1+1e-14)]:
                if math.isfinite(b): rows.append([a,b,10,30])
r=random.Random(20260929)
for _ in range(2000):
    a=r.choice([-1,1])*10**r.uniform(-308,308)
    b=r.choice([a,math.nextafter(a,math.inf),r.choice([-1,1])*10**r.uniform(-308,308),0,1,-1])
    rows.append([a,b,10,30])
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps({'function':'SWITCH','probes':[{'probe':{'id':f'switchscale-{i:05d}',
    'args':[bits(x) for x in row]}} for i,row in enumerate(rows)]}),encoding='utf-8')
print(len(rows),out)
