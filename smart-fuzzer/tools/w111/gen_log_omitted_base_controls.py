"""Pair omitted LOG base, explicit ten and LOG10 through exact numeric carriers."""
import hashlib
import json
import math
import random
import struct
from pathlib import Path


def bits(number):
    return '0x' + struct.pack('>d', float(number)).hex()


def main():
    run = Path('smart-fuzzer/runs/w111-log-omitted-base-20260929')
    if (run / 'design.json').exists():
        raise SystemExit('Immutable packet exists; use a new path.')
    seed = 202609293037
    rng = random.Random(seed)
    values = {0., .5, 1., 2., 10., 100., 1000., -1.}
    for _ in range(512):
        raw = (rng.randrange(1, 2047) << 52) | rng.getrandbits(52)
        values.add(struct.unpack('>d', raw.to_bytes(8, 'big'))[0])
    for exponent in [-307, -256, -128, -64, -32, -16, -8, -4, -2, -1, 0, 1, 2, 3, 4, 8, 16, 32, 64, 128, 256, 307]:
        value = float(f'1e{exponent}')
        values.update([math.nextafter(value, 0.), value, math.nextafter(value, math.inf)])
    values.update([math.ldexp(1., -1022), math.nextafter(math.ldexp(1., -1022), math.inf),
                   float.fromhex('0x1.fffffffffffffp1023')])
    values = sorted(values)
    (run / 'batches').mkdir(parents=True, exist_ok=True)
    for function in ['LOG', 'LOG10']:
        probes = []
        for index, value in enumerate(values):
            probes.append({'probe': {'id': f'w111logomission-{function}-{index:05d}-omitted', 'args': [bits(value)]}})
            if function == 'LOG':
                probes.append({'probe': {'id': f'w111logomission-{function}-{index:05d}-explicit', 'args': [bits(value), bits(10.)]}})
        (run / 'batches' / f'batch-{function.lower()}.json').write_text(json.dumps({'function': function, 'probes': probes}, separators=(',', ':')), encoding='utf-8')
    source = Path('crates/oxfunc_core/src/functions/log_fn.rs')
    (run / 'design.json').write_text(json.dumps({'phase': 'optional_argument_numeric_discovery', 'seed': seed,
        'distinct_inputs': len(values), 'LOG_rows': 2 * len(values), 'LOG10_rows': len(values),
        'log_source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
        'purpose': 'Test whether an omitted LOG base uses a dedicated logarithm graph distinct from explicit base ten; includes full-bit normals, decimal-power neighbors, endpoints and domain controls.'}, indent=2), encoding='utf-8')
    print(len(values)*2, 'LOG;', len(values), 'LOG10')


if __name__ == '__main__':
    main()
