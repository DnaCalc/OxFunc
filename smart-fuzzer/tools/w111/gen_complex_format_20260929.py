"""Discriminate complex-number decimal/scientific formatting, without Excel access.

Numeric inputs use binary64 bits for the existing Value2 oracle. Discovery
coverage includes powers of ten, significant digits, decimal carry boundaries,
near-integer values, both component positions, and adjacent identity IM* calls.
No subnormal inputs are included; their ingress is a separate campaign concern.
"""
import json
import math
from pathlib import Path
import struct
import sys


def bits(x):
    return '0x%016x' % struct.unpack('<Q', struct.pack('<d', x))[0]


def build(out):
    out.mkdir(parents=True, exist_ok=True)
    values = {}
    def add(x, label):
        if math.isfinite(x) and (x == 0 or abs(x) >= float.fromhex('0x1p-1022')):
            values.setdefault(bits(x), (x, label))

    exponents = sorted(set(range(-35, 36)) | {-307, -300, -100, -60, -40, 60, 100, 300, 307})
    for e in exponents:
        for m in [1.0, 1.2, 1.2345678901234567, 9.999999999999996]:
            x = m * 10.0 ** e
            add(x, f'exp{e}-mantissa{m}')
            add(-x, f'exp{e}-negative-mantissa{m}')
    for n in [0.0, 1.0, 2.0, 123.0, 1e-20, 1e-19, 1e-18, 1e19, 1e20]:
        for direction in [-math.inf, math.inf]:
            x = n
            for step in range(1, 7):
                x = math.nextafter(x, direction)
                add(x, f'nextafter-{n}-{direction}-{step}')
        for gap in [1e-10, 1e-12, 5e-13, 1e-13, 1e-14]:
            add(n + gap, f'near-integer-{n}-plus-{gap}')
            add(n - gap, f'near-integer-{n}-minus-{gap}')
    for x in [float.fromhex('0x1p-1022'), math.nextafter(float.fromhex('0x1p-1022'),math.inf),
              float.fromhex('0x1.fffffffffffffp1023'), -0.0]:
        add(x, f'normal-limit-{bits(x)}')

    manifest = []
    for fn in ['COMPLEX', 'IMCONJUGATE', 'IMSUM', 'IMPRODUCT']:
        probes = []
        for h, (x, label) in values.items():
            args = [[h, bits(0.0)], [bits(0.0), h], [h, bits(1.0)]] if fn == 'COMPLEX' else [[h]]
            for component, row in enumerate(args):
                probes.append({'probe': {'id': f'complex-format-{fn}-{label}-pos{component}', 'args': row}})
        p = out / f'batch-{fn.lower()}.json'
        p.write_text(json.dumps({'function':fn,'probes':probes},indent=2),encoding='utf-8')
        manifest.append({'function':fn,'rows':len(probes),'path':p.name})
    (out/'manifest.json').write_text(json.dumps({'generator':Path(__file__).name,
        'runner':'smart-fuzzer/tools/Run-W109BulkBatch.ps1','batches':manifest},indent=2),encoding='utf-8')
    print(json.dumps(manifest,indent=2))


if __name__ == '__main__':
    build(Path(sys.argv[1]))
