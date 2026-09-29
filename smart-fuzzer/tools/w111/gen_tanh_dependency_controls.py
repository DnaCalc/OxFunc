"""Direct SINH/COSH controls for the immutable failed TANH independent inputs."""
import json, math, struct, sys
from pathlib import Path

source, out = map(Path, sys.argv[1:3]); out.mkdir(parents=True, exist_ok=True)
report = json.loads(source.read_text())[0]
bits = lambda x: struct.unpack('<Q', struct.pack('<d', x))[0]
decode = lambda b: struct.unpack('<d', struct.pack('<Q', int(b, 16)))[0]
values = {}
for failure in report['production_misses']:
    x = decode(failure['row']['args'][0])
    values.setdefault(bits(x), 'failed-tanh-input')
    for direction in [-math.inf, math.inf]:
        y = x
        for _ in range(4):
            y = math.nextafter(y, direction)
            values.setdefault(bits(y), 'adjacent-control')
manifest = dict(generator=Path(__file__).name, source=source.as_posix(),
                authority='dependency_discovery_after_failed_frozen_tanh_holdout', batches=[])
for fn in ['SINH', 'COSH']:
    rows = [dict(probe=dict(id=f'{fn.lower()}-tanh-dependency-{i:05d}-{kind}',args=[f'0x{b:016x}']))
            for i, (b, kind) in enumerate(values.items())]
    name = f'batch-{fn.lower()}.json'
    (out/name).write_text(json.dumps(dict(function=fn, probes=rows), indent=2))
    manifest['batches'].append(dict(function=fn, path=name, rows=len(rows)))
(out/'manifest.json').write_text(json.dumps(manifest, indent=2))
print(json.dumps(manifest))
