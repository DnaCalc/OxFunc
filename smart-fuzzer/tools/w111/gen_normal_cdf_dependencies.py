"""Public ERFC dependency probes, retaining failed NORM wrappers and controls."""
import json, random, struct, sys
from pathlib import Path

SEED = 2026092935
rng = random.Random(SEED)
race_path, bank_path, out = map(Path, sys.argv[1:4]); out.mkdir(parents=True, exist_ok=True)
race = json.loads(race_path.read_text()); bank = json.loads(bank_path.read_text())
decode = lambda x: struct.unpack('<d', struct.pack('<Q', int(x, 16)))[0]
bits = lambda x: f'0x{struct.unpack("<Q", struct.pack("<d", x))[0]:016x}'
inverse_sqrt_two = decode('0x3fe6a09e667f3bcd')
zs = {r['z_bits']: 'failed-normal-dependency' for r in race['dependencies']}
xs = {r['x_bits']: 'failed-normal-wrapper' for r in race['dependencies']}
controls = [r for r in bank['witnesses'] if r['args'][1] != '0x0000000000000000'
            and 0 < abs(decode(r['args'][0])) < 40 and r['args'][0] not in xs]
for r in rng.sample(controls, min(128, len(controls))):
    xs.setdefault(r['args'][0], 'matched-normal-control')
    zs.setdefault(bits(abs(decode(r['args'][0])) * inverse_sqrt_two), 'matched-normal-control')
for _ in range(80): zs.setdefault(bits(rng.uniform(0., 27.)), 'fresh-erfc-control')
manifest = dict(generator=Path(__file__).name, seed=SEED,
    source_race=race_path.as_posix(), source_bank=bank_path.as_posix(),
    authority='normal_cdf_wrapper_vs_public_erfc_dependency_discovery', batches=[])
for fn, values in [('ERFC', zs), ('ERFC.PRECISE', zs), ('NORM.S.DIST', xs)]:
    probes = [dict(probe=dict(id=f'{fn.lower()}-cdf-dependency-{i:05d}-{kind}',
        args=[value] + (['0x3ff0000000000000'] if fn == 'NORM.S.DIST' else [])))
        for i, (value, kind) in enumerate(values.items())]
    filename = f'batch-{fn.lower()}.json'
    (out/filename).write_text(json.dumps(dict(function=fn, probes=probes), indent=2))
    manifest['batches'].append(dict(function=fn, path=filename, rows=len(probes)))
(out/'manifest.json').write_text(json.dumps(manifest, indent=2))
print(json.dumps(manifest))
