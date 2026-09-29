"""Probe COMPLEX on computed IMSQRT coefficient bits to isolate rendering from arithmetic.

The first argument is the research program's JSONL report, not Excel outputs.
Only ordinary-quadrant holdout failures are selected; neighbors are exact bits.
"""
import hashlib
import json
from pathlib import Path
import sys

report=next(json.loads(line) for line in Path(sys.argv[1]).read_text(encoding='utf-8-sig').splitlines()
            if json.loads(line)['candidate']=='term-ftz-sum-overflow-zero-ratio-angle')
probes=[]
for row in report['misses']:
    if not row['id'].endswith('ordinary-quadrant'):
        continue
    real,imag=[int(n,16) for n in row['coefficients']]
    for dr in range(-2,3):
        for di in range(-2,3):
            probes.append({'probe':{'id':f'imsqrt-output-{row["id"]}-{dr:+}-{di:+}',
                                   'args':[f'0x{real+dr:016x}',f'0x{imag+di:016x}']}})
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
path=out/'batch-complex.json'
path.write_text(json.dumps(dict(function='COMPLEX',probes=probes),indent=2),encoding='utf-8')
(out/'manifest.json').write_text(json.dumps(dict(generator=Path(__file__).name,
    source_report_sha256=hashlib.sha256(Path(sys.argv[1]).read_bytes()).hexdigest(),
    batches=[dict(function='COMPLEX',rows=len(probes),path=path.name,sha256=hashlib.sha256(path.read_bytes()).hexdigest())]),indent=2),encoding='utf-8')
print(len(probes),path)
