"""SYD discovery: sign admission, tiny lives, and stored arithmetic boundaries."""
import hashlib, itertools, json, math, random, struct
from pathlib import Path

root = Path(__file__).resolve().parents[3]
out = root / 'smart-fuzzer/cache/w111-depreciation-20260929'
bits = lambda n: '0x' + struct.pack('>d', float(n)).hex()
rows = {}

def add(args, region):
    if not all(math.isfinite(n) and (n == 0 or abs(n) >= 2**-1022) and bits(n) != '0x8000000000000000' for n in args):
        return
    rows.setdefault(tuple(map(bits, args)), region)

for cost, salvage, life, period in itertools.product([-100., 0., 100.], [-20., 0., 20., 200.], [-10., 0., .5, 10.], [-1., 0., .25, 1., 10., 11.]):
    add([cost, salvage, life, period], 'sign-and-admission')
for life in [2**-1022, 1e-300, 1e-100, 1e-20, 1e-13, 1e-12, 1e-11, 1e-8, .5, 1., 2**26, 2**52, 2**53, 1e100, 1e154, 1e155, 1e300, 1.7976931348623157e308]:
    for cost, salvage in [(100., 0.), (0., 100.), (-100., 0.), (1e-300, 0.), (1e300, 0.), (life, 0.)]:
        for period in [life/2, life, math.nextafter(life, math.inf)]:
            add([cost, salvage, life, period], 'lifetime-denominator-range')
rng = random.Random(202609292022)
for _ in range(1200):
    cost = 10**rng.uniform(-307, 307)
    salvage = cost*rng.choice([0., rng.uniform(.001, .999), rng.uniform(1., 3.)])
    life = 10**rng.uniform(-307, 307)
    period = life*rng.choice([rng.uniform(.001, .999), 1.])
    add([cost, salvage, life, period], 'full-exponent-stage-range')
for _ in range(400):
    cost = 2**-1022 * 2**rng.uniform(1, 50)
    salvage = math.nextafter(cost, rng.choice([0., math.inf]))
    life = 10**rng.uniform(-20, 20)
    add([cost, salvage, life, life*rng.uniform(.01, 1)], 'normal-input-subnormal-difference')
path = out/'discriminator-syd.json'
path.write_text(json.dumps(dict(function='SYD', probes=[dict(probe=dict(id=f'w111syd-discovery-{i:04d}', args=list(args)), probe_region=region) for i, (args, region) in enumerate(rows.items())]), indent=2)+'\n', encoding='utf-8')
(out/'discriminator-syd-manifest.json').write_text(json.dumps(dict(rows=len(rows), seed=202609292022, source_sha256=hashlib.sha256((root/'crates/oxfunc_core/src/functions/depreciation_family.rs').read_bytes()).hexdigest(), batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(), generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), purpose='Discovery only; source normal finite IEEE inputs. No candidate change or admission assumption.'), indent=2)+'\n', encoding='utf-8')
print(json.dumps(dict(path=str(path), rows=len(rows))))
