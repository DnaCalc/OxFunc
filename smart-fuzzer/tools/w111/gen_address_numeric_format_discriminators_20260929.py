"""ADDRESS numeric sheet text: initial midpoint and width-rounding alternatives."""
import json
import math
import struct
from decimal import Decimal
from pathlib import Path
from gen_address_probe_20260929 import bits

values=[]
for path in ['smart-fuzzer/runs/w111-complex-scaling-heldout-20260929/batches/batch-complex.json',
             'smart-fuzzer/runs/w111-complex-midpoints-20260929/batch-complex.json']:
    for row in json.loads(Path(path).read_text(encoding='utf-8-sig'))['probes']:
        for arg in row['probe']['args'][:2]:
            n=struct.unpack('<d',struct.pack('<Q',int(arg,16)))[0]
            if n!=0 and math.isfinite(n):values.append(n)
for exponent in [-101,-100,-99,-98,-97,97,98,99,100,101]:
    for mantissa in ['1.234567890123445','1.23456789012345','1.234567890123455',
                     '9.999999999999945','9.99999999999995','9.999999999999955']:
        center=float(Decimal(mantissa)*Decimal(10)**exponent)
        for sign in [-1,1]:
            for direction in [-math.inf,math.inf]:
                x=center*sign
                for _ in range(9):values.append(x);x=math.nextafter(x,direction)
seen=set();probes=[]
for x in values:
    xb=bits(x)
    if xb in seen:continue
    seen.add(xb)
    probes.append({'probe':{'id':f'address-numeric-format-20260929-{len(probes):05d}',
                            'args':[bits(1),bits(1),bits(1),bits(1),xb]}})
out=Path('smart-fuzzer/runs/w111-broad-20260929/address-numeric-format-discriminators.json')
out.write_text(json.dumps({'function':'ADDRESS','probes':probes},separators=(',',':')),encoding='utf-8')
print(len(probes),out)
