"""Bounded public-oracle stage controls for tiny VDB annual schedules."""
import json,math,struct,hashlib
from pathlib import Path
root=Path(__file__).resolve().parents[3];cache=root/'smart-fuzzer/cache/w111-depreciation-20260929'
rows=json.loads((cache/'vdb-publication-stage-discriminator2.json').read_text())[2]['misses']
probes=[];seen=set();enc=lambda x:'0x'+struct.pack('>d',float(x)).hex()
def emit(a,region):
 if not(0<=a[3]<a[4]<=a[2] and a[4]<1000):return
 if not all(math.isfinite(x) and(x==0 or abs(x)>=float.fromhex('0x1p-1022'))for x in a):return
 bits=tuple(map(enc,a))
 if bits in seen:return
 seen.add(bits);probes.append({'probe':{'id':f'w111vdb-tiny-stages-{len(probes):04d}','args':list(bits)},'probe_region':region})
for w in rows:
 a=w['args'];c,s,l,start,end,f,_=a;label=w['id']
 emit(a,label+'/original')
 for scale in [.25,.5,2.,4.,16.,2.**100]:emit([c*scale,s*scale,l,start,end,f,0.],label+'/scale')
 points=set()
 for center in [start,end,l-l/f if f else l]:
  for d in range(-6,7):points.add(math.floor(center)+d)
 for k in sorted(points):
  if k>start:emit([c,s,l,start,min(float(k),l),f,0.],label+'/prefix')
  if k>=0:
   emit([c,s,l,float(k),min(k+1.,l),f,0.],label+'/annual')
   if k<end:emit([c,s,l,float(k),end,f,0.],label+'/suffix')
 # Similar cost below/above normal-times-rate threshold.
 for scale in [1.,2.]:
  for frac in [0.,.25,.5,.75]:
   t=max(0.,math.floor(min(end,l-l/f if f else l))-2)+frac
   emit([c*scale,s*scale,l,t,min(t+1.,l),f,0.],label+'/fractional')
path=cache/'refinement6-vdb-switched-publication.json';path.write_text(json.dumps({'function':'VDB','probes':probes},indent=2)+'\n')
print(json.dumps({'count':len(probes),'path':str(path),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}))
