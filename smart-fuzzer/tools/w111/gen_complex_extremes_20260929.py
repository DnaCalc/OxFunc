"""Probe 15-digit decimal rounding across the normal binary64 limits.

All inputs are finite normal binary64 values; this deliberately avoids the
separate subnormal Value2-ingress issue. No Excel process is created here.
"""
import json
from pathlib import Path
import struct
import sys

bits = lambda x: '0x%016x' % struct.unpack('<Q', struct.pack('<d', x))[0]
number = lambda n: struct.unpack('<d', struct.pack('<Q', n))[0]
if len(sys.argv) > 2 and sys.argv[2] == '--ties':
    values = [(sign * (integer + fraction) * 10.0**exponent,
               f'tie-{integer}-{fraction}-exp{exponent}-sign{sign}')
              for integer in [100000000000000, 100000000000001, 123456789012344,
                              123456789012345, 999999999999998, 999999999999999]
              for fraction in [.25, .5, .75]
              for exponent in [-20, -10, -1, 0, 1, 10, 20] for sign in [1, -1]]
else:
    values = [(sign * number(0x10000000000000 + offset), f'minnormal+{offset}-sign{sign}')
              for offset in range(40) for sign in [1, -1]]
    values += [(sign * number(0x7fefffffffffffff - offset), f'maxnormal-{offset}-sign{sign}')
               for offset in range(48) for sign in [1, -1]]
probes = []
for value, label in values:
    layouts = [[bits(value), bits(0.)]] if '--ties' in sys.argv else [
        [bits(value), bits(0.)], [bits(0.), bits(value)], [bits(value), bits(1.)]]
    for position, args in enumerate(layouts):
        probes.append({'probe': {'id': f'{label}-pos{position}', 'args': args}})
out = Path(sys.argv[1])
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text(json.dumps({'function': 'COMPLEX', 'probes': probes}, indent=2), encoding='utf-8')
print(f'{len(probes)} rows -> {out}')
