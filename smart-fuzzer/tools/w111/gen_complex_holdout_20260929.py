"""Independent normal-binary64 complex-format holdout after candidate freeze.

Produces Value2-compatible ProbeBatches; no oracle/model predictions are used
in selecting these cases. The seed is separate from discovery corpora.
"""
import hashlib
import json
from pathlib import Path
import random
import struct
import sys

SEED = int(sys.argv[2]) if len(sys.argv) > 2 else 2026092902
rng = random.Random(SEED)
bits = lambda x: '0x%016x' % struct.unpack('<Q', struct.pack('<d', x))[0]
number = lambda x: struct.unpack('<d', struct.pack('<Q', x))[0]

def random_normal():
    return number((rng.getrandbits(1) << 63) | (rng.randint(1, 2046) << 52) | rng.getrandbits(52))

values = [random_normal() for _ in range(1500)]
values += [rng.choice([-1, 1]) * number(0x10000000000000 + rng.randint(0, 100000)) for _ in range(250)]
values += [rng.choice([-1, 1]) * number(0x7fefffffffffffff - rng.randint(0, 100000)) for _ in range(250)]
values += [rng.choice([-1, 1]) * (rng.randint(100000000000000, 999999999999999) + .5) for _ in range(250)]
values += [rng.choice([-1, 1]) * rng.randint(100000000000000, 999999999999999) * 10.0**rng.randint(-36, 10)
           for _ in range(250)]

out = Path(sys.argv[1]); out.mkdir(parents=True, exist_ok=True)
manifest = []
for fn in ['COMPLEX', 'IMCONJUGATE', 'IMSUM', 'IMPRODUCT']:
    probes = []
    selected = values if fn == 'COMPLEX' else values[::4]
    for i, value in enumerate(selected):
        if fn == 'COMPLEX':
            args = [value, 0.] if i % 4 == 0 else [0., value] if i % 4 == 1 else [value, 1.] if i % 4 == 2 else [value, random_normal()]
        else:
            args = [value]
        probes.append({'probe': {'id': f'complex-heldout-{SEED}-{fn}-{i:04d}', 'args': list(map(bits, args))}})
    path = out/f'batch-{fn.lower()}.json'
    path.write_text(json.dumps({'function':fn,'probes':probes},indent=2),encoding='utf-8')
    manifest.append({'function':fn,'rows':len(probes),'path':path.name,
                     'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
(out/'manifest.json').write_text(json.dumps({'generator':Path(__file__).name,
    'runner':'smart-fuzzer/tools/Run-W109BulkBatch.ps1','seed':SEED,'batches':manifest},indent=2),encoding='utf-8')
print(json.dumps(manifest,indent=2))
