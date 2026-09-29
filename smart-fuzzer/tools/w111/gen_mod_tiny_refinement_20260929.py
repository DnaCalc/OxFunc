"""MOD tiny-remainder quotient-path discriminator; normal ingress only."""
import json,math,struct,hashlib
from pathlib import Path
root=Path(__file__).resolve().parents[3]
out=root/'smart-fuzzer/runs/w111-mod-tiny-refinement-20260929';out.mkdir(exist_ok=True)
bits=lambda x:'0x'+struct.pack('>d',x).hex()
rows=[]
for de in [-1022,-1021,-1018,-1008,-990,-974]:
 d=math.ldexp(1.,de)
 for q in [1.,2.**24,2.**25,2.**26-1,2.**26,2.**26+1,2.**27,2.**30]:
  for rf in [2.**-6,2.**-4,.125,.5,.875,1.,1.5]:
   x=d*q+math.ldexp(rf,-1022)
   for sx,sd in [(1,1),(-1,1),(1,-1),(-1,-1)]:rows.append((sx*x,sd*d))
# Exact neighbors around quotient cutoff without imposing a remainder magnitude.
for de in [-1022,-1021,-1018,-1008]:
 d=math.ldexp(1.,de);x=d*2.**26
 for i in [-3,-2,-1,0,1,2,3]:
  v=struct.unpack('>d',(int(bits(x),16)+i).to_bytes(8,'big'))[0]
  for sx,sd in [(1,1),(-1,1),(1,-1),(-1,-1)]:rows.append((sx*v,sd*d))
encoded=list(dict.fromkeys(tuple(map(bits,r)) for r in rows))
p=out/'batch-mod.json';p.write_text(json.dumps({'function':'MOD','probes':[{'probe':{'id':f'w111mod-tiny-refinement-{i:05d}','args':r}}for i,r in enumerate(encoded)]},separators=(',',':')),encoding='utf-8')
(out/'manifest.json').write_text(json.dumps({'rows':len(encoded),'stage':'discovery after failed frozen34600','purpose':'Power-of-two tiny remainder quotient path at2^26; no lookup candidate','batch_sha256':hashlib.sha256(p.read_bytes()).hexdigest()},indent=2),encoding='utf-8')
print(len(encoded))
