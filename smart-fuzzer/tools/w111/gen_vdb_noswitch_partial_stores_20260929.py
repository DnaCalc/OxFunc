"""Discriminate ordinary versus extended stores of VDB partial products."""
import hashlib, json, math, random, struct
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=root/'smart-fuzzer/cache/w111-depreciation-20260929'
bits=lambda n:'0x'+struct.pack('>d',float(n)).hex()
decode=lambda h:struct.unpack('>d',bytes.fromhex(h[2:]))[0]
w=json.loads((out/'vdb-noswitch-partial-residual.json').read_text(encoding='utf-8-sig'))['witnesses'][0]
a=list(map(decode,w['args']));rng=random.Random(202609292023);rows={}
def add(args,tag):
 if all(math.isfinite(x) and (x==0 or abs(x)>=2**-1022) for x in args):rows.setdefault(tuple(map(bits,args)),tag)
for scale in [-450,-100,0,100,450]:
 for neighbors in range(-12,13):
  c=math.ldexp(a[0],scale);s=math.ldexp(a[1],scale)
  for _ in range(abs(neighbors)):c=math.nextafter(c,0. if neighbors<0 else math.inf)
  for width in [1,2,3,5]:
   end=a[3]
   for _ in range(width):end=math.nextafter(end,math.inf)
   add([c,s,a[2],a[3],end,a[5],1.],'residual-scaled-cost-neighbors')
for _ in range(300):
 c=10**rng.uniform(-250,250);s=c*rng.uniform(0,.999);life=rng.uniform(1.,50.)
 start=rng.uniform(.0001,.999)
 end=start
 for _ in range(rng.randrange(1,10)):end=math.nextafter(end,math.inf)
 add([c,s,life,start,end,rng.uniform(.01,8),1.],'fresh-adjacent-first-year-products')
path=out/'refinement3-vdb-noswitch.json'
path.write_text(json.dumps(dict(function='VDB',probes=[dict(probe=dict(id=f'w111vdbns-store-{i:04d}',args=list(k)),probe_region=v) for i,(k,v) in enumerate(rows.items())]),indent=2)+'\n',encoding='utf-8')
(out/'refinement3-vdb-noswitch-manifest.json').write_text(json.dumps(dict(rows=len(rows),seed=202609292023,source_residual=w['id'],batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),purpose='Discovery follows frozen v3 failure; first-year short intervals only, no large-index loops'),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),rows=len(rows))))
