"""Separate VDB partial-product publication from final subnormal subtraction."""
import hashlib,json,math,random,struct
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=root/'smart-fuzzer/cache/w111-depreciation-20260929'
bits=lambda n:'0x'+struct.pack('>d',float(n)).hex();decode=lambda h:struct.unpack('>d',bytes.fromhex(h[2:]))[0]
judge=json.loads((out/'fourth-heldout-vdb-noswitch-judgement.json').read_text(encoding='utf-8-sig'))[0]
rows={}
def add(args,tag):
 if all(math.isfinite(x) and (x==0 or abs(x)>=2**-1022) for x in args):rows.setdefault(tuple(map(bits,args)),tag)
for w in judge['misses']:
 a=list(map(decode,w['args']))
 for scale in [.25,.5,1.,2.,4.,8.]:
  for left,right in [(a[3],a[4]),(math.floor(a[3]),a[4]),(a[3],math.ceil(a[4])),(math.floor(a[3]),math.ceil(a[4]))]:
   if right<=a[2]:add([a[0]*scale,a[1]*scale,a[2],left,right,a[5],1.],'failed-independent-stage-separation')
rng=random.Random(202609292026)
for _ in range(600):
 life=rng.uniform(3.,70.);factor=10**rng.uniform(-200,-3);rate=factor/life
 cost=2**-1022*rng.uniform(1.,12.)/rate
 start=rng.uniform(0.,life-1);end=rng.uniform(start,life)
 add([cost,cost*rng.uniform(0.,.9),life,start,end,factor,1.],'partial-product-normal-subnormal-neighbors')
path=out/'refinement4-vdb-noswitch.json'
path.write_text(json.dumps(dict(function='VDB',probes=[dict(probe=dict(id=f'w111vdbns-sub-{i:04d}',args=list(a)),probe_region=t) for i,(a,t) in enumerate(rows.items())]),indent=2)+'\n',encoding='utf-8')
(out/'refinement4-vdb-noswitch-manifest.json').write_text(json.dumps(dict(rows=len(rows),seed=202609292026,source_judgement_sha256=hashlib.sha256((out/'fourth-heldout-vdb-noswitch-judgement.json').read_bytes()).hexdigest(),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),purpose='Discovery: normal source inputs, bounded loops, final subtraction may publish a nonzero subnormal'),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),rows=len(rows))))
