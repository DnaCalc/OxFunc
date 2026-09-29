"""POISSON positive-count publication boundaries; mathematical input selection."""
import json, math, random, struct, sys
from pathlib import Path

out = Path(sys.argv[1]); out.mkdir(parents=True, exist_ok=True)
phase = sys.argv[2] if len(sys.argv) > 2 else 'discovery'
assert phase in ['discovery', 'heldout']
seed = 2026092937 if phase == 'discovery' else 2026092938
rng = random.Random(seed)
counts = [1, 2, 3, 7, 19, 53] if phase == 'discovery' else [4, 11, 29, 89]
radius = 20 if phase == 'discovery' else 24
bits = lambda x: f'0x{struct.unpack("<Q", struct.pack("<d", float(x)))[0]:016x}'
rows = {}
def emit(x, mean, flag, kind):
    key = tuple(map(bits, [x, mean, flag])); rows.setdefault(key, kind)

def log_probability(k, mean, cdf):
    terms = [j * math.log(mean) - mean - math.lgamma(j+1) for j in range(k+1)]
    if not cdf: return terms[-1]
    lead = max(terms)
    return lead + math.log(math.fsum(math.exp(t-lead) for t in terms))

for k in counts:
    for cdf in [False, True]:
        low, high = 600., 2000.
        for _ in range(64):
            mid = (low+high)/2
            if log_probability(k, mid, cdf) > math.log(sys.float_info.min): low = mid
            else: high = mid
        center = (low+high)/2
        means = [center]
        for direction in [-math.inf, math.inf]:
            v = center
            for _ in range(radius):
                v = math.nextafter(v, direction); means.append(v)
        for mean in means:
            for flag in [0,1]: emit(k, mean, flag, 'normal-subnormal-boundary')
        for shift in [-.75, -.125, .125, .75, 9., 37.]:
            for flag in [0,1]: emit(k + (0.375 if phase=='heldout' else 0.), center+shift, flag, 'boundary-controls')
    for _ in range(20):
        mean = rng.uniform(695., 890.)
        for flag in [0,1]: emit(k, mean, flag, 'fresh-near-underflow')
for x in [0., .625, sys.float_info.min, -.625, -sys.float_info.min]:
    for mean in [708.25, 709.5, 711.75, 744.875, 746.25, -2.125]:
        for flag in [0,1]: emit(x, mean, flag, 'zero-branch-and-raw-negative-controls')
probes = [dict(probe=dict(id=f'poisson-publication-{phase}-{i:05d}-{kind}', args=list(args)))
          for i,(args,kind) in enumerate(rows.items())]
name = 'batch-poisson.dist.json'
(out/name).write_text(json.dumps(dict(function='POISSON.DIST',probes=probes),indent=2))
manifest = dict(generator=Path(__file__).name, seed=seed, phase=phase,
    authority='mathematical_boundary_selection_without_oracle_outputs',
    batches=[dict(function='POISSON.DIST',path=name,rows=len(probes))])
(out/'manifest.json').write_text(json.dumps(manifest,indent=2))
print(json.dumps(manifest))
