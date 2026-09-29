"""Independent zero-count and count-admission controls after the branch freeze."""
import itertools, json, math, random, struct, sys
from pathlib import Path

SEED = 2026092933
rng = random.Random(SEED)
out = Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=True)
bits = lambda x: f'0x{struct.unpack("<Q", struct.pack("<d", float(x)))[0]:016x}'
rows = []
def emit(x, mean, flag, kind):
    rows.append(dict(probe=dict(id=f'poisson-branch-heldout-{len(rows):05d}-{kind}',
                               args=list(map(bits, [x, mean, flag])))))

# Fresh signed count neighborhoods and independent means; no oracle selection.
counts = [-math.nextafter(1., 0.), -2**-44, 0., 2**-42,
          math.nextafter(1., 0.), 1., math.nextafter(1., math.inf), 1.75, 4.25]
means = [-708.75, -12.125, -2.75, -2**-42, 0., 2**-40, .375, 6.125,
         708.5, 711.25, 744.75, 750.]
for x, mean, flag in itertools.product(counts, means, [0, 1]):
    emit(x, mean, flag, 'count-domain')
for boundary in [-math.log(sys.float_info.max), math.log(2**1022), 745.]:
    for direction in [-math.inf, math.inf]:
        mean = boundary
        for _ in range(7):
            mean = math.nextafter(mean, direction)
            for x, flag in itertools.product([0., .625], [0, 1]):
                emit(x, mean, flag, 'exponential-boundary')
for _ in range(180):
    mean = rng.uniform(-709.7, 745.)
    for flag in [0, 1]:
        emit(rng.choice([0., rng.random(), -rng.random()]), mean, flag, 'fresh-random')
packet = dict(function='POISSON.DIST', probes=rows)
(out/'batch-poisson.dist.json').write_text(json.dumps(packet, indent=2))
manifest = dict(generator=Path(__file__).name, seed=SEED,
                authority='independent_after_poisson_branch_candidate_freeze',
                batches=[dict(function='POISSON.DIST', path='batch-poisson.dist.json', rows=len(rows))])
(out/'manifest.json').write_text(json.dumps(manifest, indent=2))
print(json.dumps(manifest))
