"""Discovery traces around the remaining small-bit switched-VDB differences."""
import hashlib,json,math,struct
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=root/'smart-fuzzer/cache/w111-depreciation-20260929'
residual=out/'vdb-shift-duration-2258.json'
misses=json.loads(residual.read_text(encoding='utf-8-sig'))['misses']
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
rows={}
for w in misses:
 a=w['args'];c,s,l,start,end=a[:5];factor=a[5] if len(a)>5 else 2.
 frac=start-math.floor(start)
 for left in [0.,frac,start]:
  for width in [0.125,0.5,1.,2.,4.]:
   right=min(l,left+width)
   if right<=left:continue
   key=tuple(map(bits,[c,s,l,left,right,factor,0.]))
   rows.setdefault(key,dict(source=w['id'],stage='zero-origin' if left==0 else 'fractional-origin' if left==frac else 'original-origin'))
 if frac>0:rows.setdefault(tuple(map(bits,[c,s,l,0.,frac,factor,0.])),dict(source=w['id'],stage='initial-fraction'))
path=out/'refinement3-vdb-switched.json'
path.write_text(json.dumps(dict(function='VDB',probes=[dict(probe=dict(id=f'w111vdb-stage-{i:04d}',args=list(a)),probe_region=tag['stage']) for i,(a,tag) in enumerate(rows.items())]),indent=2)+'\n',encoding='utf-8')
(out/'refinement3-vdb-switched-manifest.json').write_text(json.dumps(dict(rows=len(rows),residual_source=residual.name,residual_sha256=hashlib.sha256(residual.read_bytes()).hexdigest(),source_misses=len(misses),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),lineage=list(rows.values()),qualification='Discovery selected from residuals, not independent validation. All intervals bounded by original life below1000.'),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),rows=len(rows))))
