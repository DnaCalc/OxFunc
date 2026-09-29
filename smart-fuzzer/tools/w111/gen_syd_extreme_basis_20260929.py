"""SYD endpoint discovery beyond decimal-exponent random sampling."""
import hashlib,itertools,json,math,struct
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=root/'smart-fuzzer/cache/w111-depreciation-20260929'
bits=lambda n:'0x'+struct.pack('>d',float(n)).hex();maximum=float.fromhex('0x1.fffffffffffffp+1023')
rows={}
for cost,salvage,life,fraction in itertools.product([-maximum,-maximum/2,maximum/2,maximum],[0.,maximum/2,maximum],[.125,1.,2.,10.,1e154,1e155],[.5,1.]):
 a=[cost,salvage,life,life*fraction];rows[tuple(map(bits,a))]='extreme-cost-basis-overflow-and-error-order'
path=out/'refinement3-syd.json';path.write_text(json.dumps(dict(function='SYD',probes=[dict(probe=dict(id=f'w111syd-extreme-{i:04d}',args=list(a)),probe_region=t) for i,(a,t) in enumerate(rows.items())]),indent=2)+'\n',encoding='utf-8')
(out/'refinement3-syd-manifest.json').write_text(json.dumps(dict(rows=len(rows),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),purpose='Discovery: exact finite IEEE maximum, including overflowing negative cost minus positive salvage, which decimal-exponent cohorts did not cover.'),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),rows=len(rows))))
