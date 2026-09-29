"""Independent ADDRESS candidate validation; no oracle results are read."""
import hashlib
import json
import math
import random
import struct
from pathlib import Path

SEED = 202609291429
rng = random.Random(SEED)
out = Path('smart-fuzzer/runs/w111-broad-20260929')
evidence = Path('docs/function-lane/evidence/w111-broad-20260929/address')
sources = ['crates/oxfunc_core/src/functions/reference_metadata_family.rs',
           'crates/oxfunc_core/src/functions/address_name_classes.rs',
           'crates/oxfunc_core/src/functions/round_fn.rs']
freeze = {'seed': SEED, 'state': 'candidate_frozen_for_independent_non_text_coercion_validation',
          'open_lanes': ['numeric text coercion syntax and host date/year context',
                         'formal sheet rendering alignment'],
          'source_sha256': {p: hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in sources}}
(evidence/'refined-candidate-freeze.json').write_text(json.dumps(freeze, indent=2), encoding='utf-8')

def bits(x):
    return f'0x{struct.unpack("<Q",struct.pack("<d",float(x)))[0]:016x}'

probes = []
def add(args, lane):
    probes.append({'probe': {'id': f'address-heldout-{SEED}-{lane}-{len(probes):05d}',
                              'args': [bits(x) if not isinstance(x, str) else x for x in args]}})

def coordinate(limit):
    center = rng.choice([0, 1, -1, limit, -limit, rng.randrange(-limit-5, limit+6)])
    flavor = rng.randrange(4)
    if flavor == 0:
        threshold = (2049/2**33 if center > 0 else 2049/2**34)
        return math.nextafter(center-threshold, rng.choice([-math.inf, math.inf]))
    if flavor == 1:
        return math.nextafter(float(center), rng.choice([-math.inf, math.inf]))
    return center + rng.choice([-1, 1]) * rng.random()

for _ in range(1800):
    add([coordinate(1048576), coordinate(16384), rng.choice([1,2,3,4,0,5])+rng.choice([0,0,0,rng.random()]),
         rng.choice([0,1])], 'coordinates')

for i in range(1800):
    if i < 1200:
        raw = (rng.getrandbits(1)<<63) | (rng.randrange(1,2047)<<52) | rng.getrandbits(52)
        x = struct.unpack('<d',struct.pack('<Q',raw))[0]
    else:
        # Fresh fifteen-digit halfway neighborhoods across the finite range.
        head = rng.randrange(10**14, 10**15)
        exponent = rng.randrange(-300, 294)
        x = float(f'{head}5e{exponent-15}')
        x = math.nextafter(x, rng.choice([-math.inf, math.inf]))
        x *= rng.choice([-1, 1])
    add([rng.randrange(1,1048577), rng.randrange(1,16385), rng.randrange(1,5), rng.randrange(2), x], 'numeric-sheet')

alphabet = list('AbcXZ_r.?09\\ -\'[]') + [chr(x) for x in range(0x100,0x10000) if not 0xD800 <= x <= 0xDFFF]
for i in range(1800):
    if i < 600:
        text = ''.join(rng.choice(alphabet) for _ in range(rng.randrange(1,18)))
    elif i < 1100:
        axis = rng.choice(['R','C','RC','CR','r','c','rC','Rc'])
        text = axis + rng.choice(['0','00','01',str(rng.randrange(1,1048580)),str(rng.randrange(1,16388))])
        text += rng.choice(['','?','_','.','..','.x','A1','C0','R1','e0','e1','e999'])
    elif i < 1400:
        text = '[' + ''.join(rng.choice(alphabet) for _ in range(rng.randrange(0,12))) + ']'
        text += ''.join(rng.choice(alphabet) for _ in range(rng.randrange(0,12)))
    else:
        text = rng.choice(['A',' ','\'','😀','\u0301']) * rng.randrange(118,265)
        text += rng.choice(['', "'", 'R1', ' '])
    add([rng.randrange(1,1048577),rng.randrange(1,16385),rng.randrange(1,5),rng.randrange(2),text], 'sheet-grammar')

path=out/'address-refined-heldout.json'
path.write_text(json.dumps({'function':'ADDRESS','probes':probes},ensure_ascii=False,separators=(',',':')),encoding='utf-8')
freeze['input_sha256']=hashlib.sha256(path.read_bytes()).hexdigest()
freeze['rows']=len(probes)
(evidence/'refined-candidate-freeze.json').write_text(json.dumps(freeze,indent=2),encoding='utf-8')
print(len(probes),path)
