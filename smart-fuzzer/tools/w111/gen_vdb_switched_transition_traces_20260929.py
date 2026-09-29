"""Locate the remaining small-bit VDB difference across straight-line transitions."""
import hashlib,json,math,struct
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=root/'smart-fuzzer/cache/w111-depreciation-20260929'
residual=out/'vdb-shift-duration-2258.json'
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
rows={}
for w in json.loads(residual.read_text(encoding='utf-8-sig'))['misses']:
 c,s,l,start,end=w['args'][:5];factor=w['args'][5] if len(w['args'])>5 else 2.
 for year in range(4,math.ceil(end-start)+1):
  for offset in [-.125,0.,.125]:
   right=min(l,start+year+offset)
   if right<=start:continue
   rows.setdefault(tuple(map(bits,[c,s,l,start,right,factor,0.])),w['id'])
 for right in [math.nextafter(end,0.),end,math.nextafter(end,math.inf)]:
  if start<right<=l:rows.setdefault(tuple(map(bits,[c,s,l,start,right,factor,0.])),w['id'])
path=out/'refinement4-vdb-switched.json'
path.write_text(json.dumps(dict(function='VDB',probes=[dict(probe=dict(id=f'w111vdb-transition-{i:04d}',args=list(a)),probe_region='straight-line-transition-prefix-and-neighbors') for i,a in enumerate(rows)]),indent=2)+'\n',encoding='utf-8')
(out/'refinement4-vdb-switched-manifest.json').write_text(json.dumps(dict(rows=len(rows),source_residual=residual.name,residual_sha256=hashlib.sha256(residual.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),selection='Discovery selected from 50 small-bit residuals after the 770 short-stage observations all matched; all intervals below1000'),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),rows=len(rows))))
