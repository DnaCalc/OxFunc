"""Fresh TANH/COTH inputs after the cancellation-safe composition freeze."""
import json, math, random, struct, sys
from pathlib import Path

SEED = 2026092934
rng = random.Random(SEED)
out = Path(sys.argv[1]); out.mkdir(parents=True, exist_ok=True)
values = {}
bits = lambda x: struct.unpack('<Q', struct.pack('<d', x))[0]
def add(x, kind):
    if math.isfinite(x) and (abs(x) >= sys.float_info.min or bits(x) == 0):
        values.setdefault(bits(x), kind)

for center in [0., 1., .5, math.log(2), 2**-53, 1.5e-16, 1e-8, 10., 709.5, 710.]:
    for sign in [1., -1.]:
        center = abs(center) * sign
        add(center, 'boundary')
        for direction in [-math.inf, math.inf]:
            x = center
            for _ in range(17):
                x = math.nextafter(x, direction); add(x, 'boundary-neighbor')
for _ in range(2500): add(rng.uniform(-1., 1.), 'fresh-small')
for _ in range(1500): add(math.ldexp(rng.uniform(1., 2.), rng.randrange(-1022, -2)) * rng.choice([-1., 1.]), 'fresh-tiny')
for _ in range(512): add(math.ldexp(rng.uniform(1., 2.), rng.randrange(-58, -48)) * rng.choice([-1., 1.]), 'expm1-near-one')
for _ in range(1000): add(rng.uniform(-715., 715.), 'ordinary-and-overflow')
for _ in range(1000):
    raw = rng.getrandbits(64)
    if 0 < (raw >> 52) & 2047 < 2047:
        add(struct.unpack('<d', struct.pack('<Q', raw))[0], 'fresh-normal-bits')

manifest = dict(generator=Path(__file__).name, seed=SEED,
    authority='independent_after_tanh_coth_composition_source_freeze', batches=[])
for fn in ['TANH', 'COTH']:
    rows = [dict(probe=dict(id=f'{fn.lower()}-composition-heldout-{i:05d}-{kind}', args=[f'0x{raw:016x}']))
            for i, (raw, kind) in enumerate(values.items())]
    path = f'batch-{fn.lower()}.json'
    (out/path).write_text(json.dumps(dict(function=fn, probes=rows), indent=2))
    manifest['batches'].append(dict(function=fn, path=path, rows=len(rows)))
(out/'manifest.json').write_text(json.dumps(manifest, indent=2))
print(json.dumps(manifest))
