"""Discriminate an empirically suggested absolute VDB switch threshold."""
import json,math,struct
from pathlib import Path
root=Path(__file__).resolve().parents[3];cache=root/'smart-fuzzer/cache/w111-depreciation-20260929';mn=float.fromhex('0x1p-1022')
enc=lambda x:'0x'+struct.pack('>d',float(x)).hex();rows=[];seen=set()
def emit(a,r):
 bits=tuple(map(enc,a))
 if bits in seen:return
 seen.add(bits);rows.append({'probe':{'id':f'w111vdb-switch-boundary-{len(rows):04d}','args':list(bits)},'probe_region':r})
for life in [2.,4.,8.]:
 for money in [4.,8.,16.,32.]:
  cost=money*mn
  center=1.-life/(16.*money)
  for factor in [math.nextafter(center,0.),center,math.nextafter(center,math.inf),center-2.**-40,center+2.**-40,1.]:
   for start,end in [(0.,1.),(.25,1.25),(1.,2.)]:emit([cost,0.,life,start,end,factor,0.],'absolute-MIN-over-16-neighbors')
for scale in [1.,2.**-500,2.**-1000]:
 for factor in [math.nextafter(1.,0.),1.,math.nextafter(1.,2.),1.-2.**-30]:
  emit([scale,0.,4.,0.,1.,factor,0.],'relative-tolerance-control')
p=cache/'refinement7-vdb-switch-comparison.json';p.write_text(json.dumps({'function':'VDB','probes':rows},indent=2)+'\n');print(len(rows),p)
