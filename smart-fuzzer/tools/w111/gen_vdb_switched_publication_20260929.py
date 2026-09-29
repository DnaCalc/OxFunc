"""Switched VDB publication boundaries, kept separate from moderate heldouts."""
import hashlib,json,math,struct
from pathlib import Path
root=Path(__file__).resolve().parents[3];cache=root/'smart-fuzzer/cache/w111-depreciation-20260929'
small=float.fromhex('0x1p-1022');large=float.fromhex('0x1.fffffffffffffp+1023')
encode=lambda x:'0x'+struct.pack('>d',float(x)).hex()
costs=[small,2*small,2.**-1000,2.**-500,1.,2.**500,2.**1000,large*.5,large]
probes=[];seen=set()
for cost in costs:
 for salvage in [0.,cost*.5,cost]:
  if salvage!=0 and salvage<small:continue
  for life in [.125,.5,1.,3.5,32.,999.]:
   for factor in [0.,small,.5,2.,8.,2.**1000,large]:
    for start,end in [(0.,life),(life*.125,life*.5),(life*.75,life)]:
     args=[cost,salvage,life,start,end,factor,0.]
     assert all(math.isfinite(x) and (x==0 or abs(x)>=small) for x in args)
     assert 0<=start<end<=life<1000
     bits=tuple(map(encode,args))
     if bits in seen:continue
     seen.add(bits);probes.append({'probe':{'id':f'w111vdb-publication-{len(probes):04d}','args':list(bits)},
                                 'probe_region':'finite-IEEE-normal-extrema-and-derived-overflow-underflow'})
path=cache/'discovery-vdb-switched-publication.json';path.write_text(json.dumps({'function':'VDB','probes':probes},indent=2)+'\n')
(cache/'discovery-vdb-switched-publication-manifest.json').write_text(json.dumps(dict(rows=len(probes),
    batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    qualification='Publication discovery, not moderate-domain heldout. Inputs finite and normal or+0; all life/end<1000. Expected semantic errors remain outcomes; no output normalization.'),indent=2)+'\n')
print(json.dumps(dict(path=str(path),rows=len(probes))))
