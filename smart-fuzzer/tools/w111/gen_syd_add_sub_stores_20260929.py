"""SYD extended addition/subtraction discriminators from explicit halfway neighbors."""
import hashlib,json,math,struct
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=root/'smart-fuzzer/cache/w111-depreciation-20260929'
bits=lambda n:'0x'+struct.pack('>d',float(n)).hex();rows={}
def add(a,tag):
 if all(math.isfinite(x) and (x==0 or abs(x)>=2**-1022) for x in a):rows.setdefault(tuple(map(bits,a)),tag)
for delta in [-4096,-1024,-64,-8,-1,0,1,8,64,1024,4096]:
 for exponent in [-900,-200,0,200,900]:
  for sign in [-1.,1.]:
   c=math.ldexp(sign,exponent);s=math.ldexp(2**(-54 if sign>0 else -53),exponent)
   s=struct.unpack('>d',struct.pack('>Q',int(bits(s)[2:],16)+delta))[0]
   for life,period in [(1.,1.),(3.75,.125),(12.875,2.125)]:add([c,s,life,period],'basis-subtraction-halfway-neighbors')
 for scale in [1.,1e-250,1e250]:
  p=struct.unpack('>d',struct.pack('>Q',int(bits(2**-53)[2:],16)+delta))[0]
  add([scale,0.,1.,p],'remaining-life-subtraction-halfway-neighbors')
  add([scale,0.,p,p*.5],'life-plus-one-addition-halfway-neighbors')
path=out/'refinement2-syd.json';path.write_text(json.dumps(dict(function='SYD',probes=[dict(probe=dict(id=f'w111syd-addsub-{i:04d}',args=list(k)),probe_region=t) for i,(k,t) in enumerate(rows.items())]),indent=2)+'\n',encoding='utf-8')
(out/'refinement2-syd-manifest.json').write_text(json.dumps(dict(rows=len(rows),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),purpose='Discovery, explicit 64-bit extension halfway windows and cross-exponent copies. Normal Value2 inputs only.'),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),rows=len(rows))))
