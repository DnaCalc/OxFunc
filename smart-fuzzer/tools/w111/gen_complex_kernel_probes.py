"""Discriminate IMSQRT magnitude/trig and IMDIV division expressions.

Numeric inputs use exact Value2 encodings. A separate typed packet exercises
complex text across quadrants, scales, suffixes and both division orientations.
No model or oracle outputs select the independent numeric random inputs.
"""
import hashlib
import json
import math
from pathlib import Path
import random
import struct
import sys

import gen_broad_typed_20260929 as typed

SEED = 2026092906
bits = lambda x: f'0x{struct.unpack("<Q",struct.pack("<d",x))[0]:016x}'


def main(out):
    rng = random.Random(SEED)
    values = [0.0, 1.0, -1.0, .1, -.1]
    for exponent in [-308,-300,-200,-170,-163,-162,-161,-160,-155,-154,-153,-150,-100,-20,20,100,150,153,154,155,160,200,300,308]:
        center = 10.0**exponent
        for neighbor in [math.nextafter(center,0),center,math.nextafter(center,math.inf)]:
            values += [neighbor,-neighbor]
    for center in [math.sqrt(sys.float_info.min), math.sqrt(sys.float_info.max), math.sqrt(float.fromhex('0x0.0000000000001p-1022'))]:
        for offset in range(-6,7):
            encoded = struct.unpack('<Q',struct.pack('<d',center))[0] + offset
            n=struct.unpack('<d',struct.pack('<Q',encoded))[0]
            values += [n,-n]
    values += [rng.choice([-1,1])*rng.uniform(1,9)*10.0**rng.randint(-308,307) for _ in range(600)]
    batches=out/'batches';batches.mkdir(parents=True,exist_ok=True)
    manifest=[]
    for function in ['IMSQRT','IMDIV']:
        args = [[n] for n in values] if function=='IMSQRT' else [
            [rng.choice([-1,1])*rng.uniform(1,9)*10.0**rng.randint(-308,307), n] for n in values]
        if function=='IMDIV':
            args += [[a,b] for a in [-1e300,-1e-300,0.,1e-300,1e300] for b in [-1e300,-1e-300,0.,1e-300,1e300]]
        probes=[{'probe':{'id':f'complex-kernel-{SEED}-{function}-{i:04d}','args':list(map(bits,a))}} for i,a in enumerate(args)]
        path=batches/f'batch-{function.lower()}.json'
        path.write_text(json.dumps({'function':function,'probes':probes},indent=2),encoding='utf-8')
        manifest.append(dict(function=function,rows=len(probes),path=path.name,sha256=hashlib.sha256(path.read_bytes()).hexdigest()))
    (batches/'manifest.json').write_text(json.dumps(dict(generator=Path(__file__).name,seed=SEED,batches=manifest),indent=2),encoding='utf-8')
    typed.TRANCHE='w111-complex-kernel-typed-20260929'
    for exponent in [-200,-160,-154,-10,0,10,150,154,160,200]:
        for real,imag in [(1,1),(-1,1),(1,-1),(-1,-1),(3,4),(-3,4),(0,2),(2,0)]:
            text=f'{real}e{exponent}{imag:+}e{exponent}i'
            typed.emit('IMSQRT','quadrant-scale',[typed.t(text)])
            for denominator in ['2','2i','3+4i','-3-4i',text]:
                typed.emit('IMDIV','scaled-division',[typed.t(text),typed.t(denominator)])
    for text in ['i','-i','j','-j','1+j','-1-j','0','0i','0j','-1+0i','-1-0i','-1+0j','-1-0j']:
        typed.emit('IMSQRT','suffix-axis',[typed.t(text)])
    (out/'typed.json').write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
        authority='non_semantic_exploration_input',generator=Path(__file__).name,tranche_id=typed.TRANCHE,
        comparison_policy='exact_typed_bit_match_no_tolerance',cases=typed.cases,
        tranches=[dict(tranche_id=typed.TRANCHE,case_ids=[c['case_id'] for c in typed.cases])],
        summary=dict(case_count=len(typed.cases),surfaces_covered=2)),indent=2),encoding='utf-8')
    print(manifest)
    print(len(typed.cases),'typed probes',out/'typed.json')


if __name__=='__main__':
    main(Path(sys.argv[1]))
