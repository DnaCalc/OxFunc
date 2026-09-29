"""ADDRESS's numeric fifth argument exercises number-to-sheet-text coercion."""
import json
import math
import random
from pathlib import Path
from gen_address_probe_20260929 import bits

values=[0.,1.,-1.,1.2345678901234567,100000000000000.5,123456789012344.5,
        999999999999998.5,float.fromhex('0x1p-1022'),float.fromhex('0x1.fffffffffffffp1023')]
for exponent in list(range(-20,21))+[-307,-306,-100,100,306,307]:
    for scale in [1.,1.2345678901234567,9.99999999999999]:
        x=scale*10.**exponent
        if math.isfinite(x) and x>=float.fromhex('0x1p-1022'):
            values += [x,-x,math.nextafter(x,0),math.nextafter(x,math.inf)]
rng=random.Random(202609290703)
for _ in range(1200):
    x=rng.uniform(1,9.999999999)*10.**rng.randint(-307,307)
    if math.isfinite(x) and x>=float.fromhex('0x1p-1022'):values.append(x*rng.choice([-1,1]))
probes=[];seen=set()
for x in values:
    xb=bits(x)
    if xb in seen:continue
    seen.add(xb)
    probes.append({'probe':{'id':f'address-numeric-sheet-20260929-{len(probes):05d}',
                            'args':[bits(1),bits(1),bits(1),bits(1),xb]}})
out=Path('smart-fuzzer/runs/w111-broad-20260929/address-numeric-sheet.json')
out.write_text(json.dumps({'function':'ADDRESS','probes':probes},separators=(',',':')),encoding='utf-8')
print(len(probes),out)
