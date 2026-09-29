"""Oracle-only VDB short-interval index probes; do not replay the current local loop."""
import hashlib,json,math,struct
from pathlib import Path
root=Path(__file__).resolve().parents[3]
out=root/'smart-fuzzer/cache/w111-depreciation-20260929'
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
rows={}
for start in [2**31-2,2**31-1,2**31,2**32-2,2**32-1,2**32,2**52-1,2**52,2**53-2,2**53-1,2**53,2**53+2,2**54,1e20]:
 start=float(start)
 for end in [math.nextafter(start,math.inf),max(start+1.,math.nextafter(start,math.inf))]:
  for factor in [0.,2.]:
   life=end*2.
   args=[1000.,0.,life,start,end,factor,1.]
   rows[tuple(map(bits,args))]=dict(start=start,end=end,interval_width=end-start)
path=out/'oracle-only-vdb-large-index.json'
path.write_text(json.dumps(dict(function='VDB',local_replay_policy='FORBIDDEN until local VDB progress is repaired; oracle-only packet',probes=[dict(probe=dict(id=f'w111vdb-largeindex-{i:03d}',args=list(k)),probe_region='integer-width-and-binary64-increment-boundaries') for i,k in enumerate(rows)]),indent=2)+'\n',encoding='utf-8')
(out/'oracle-only-vdb-large-index-manifest.json').write_text(json.dumps(dict(rows=len(rows),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),local_replay_policy='FORBIDDEN until local VDB progress is repaired',intervals=list(rows.values())),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),rows=len(rows))))
