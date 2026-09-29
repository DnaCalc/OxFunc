"""General tiny-remainder and magnitude-word boundary discovery for MOD."""
import hashlib,json,math,random,struct
from pathlib import Path
from datetime import datetime,timezone
root=Path(__file__).resolve().parents[3];out=root/'smart-fuzzer/runs/w111-mod-tiny-discriminator-20260929';out.mkdir(exist_ok=True);assert not (out/'manifest.json').exists()
seed=202609293304;rng=random.Random(seed);bits=lambda x:'0x'+struct.pack('>d',float(x)).hex();decode=lambda b:struct.unpack('>d',b.to_bytes(8,'big'))[0]
rows=[]
for exponent in range(40,54):
    base=exponent<<52
    for i in range(1,130):
        for sign in [-1,1]:rows.extend([(sign*decode(base+i),decode(base)),(sign*decode(base+i),-decode(base)),(sign*decode(base),decode(base+i)),(sign*decode(base),-decode(base+i))])
accepted=0
while accepted<5000:
    d=decode((rng.randrange(1,8)<<52)|rng.getrandbits(52));x=decode((rng.randrange(1,36)<<52)|rng.getrandbits(52));r=math.fmod(x,d)
    if not 0<r<2**-1022:continue
    accepted+=1
    rows.extend((s*x,t*d) for s in [-1,1] for t in [-1,1])
prior={tuple(r['args']) for r in json.loads((root/'smart-fuzzer/runs/w111-mod-tiny-boundary-20260929/answers-mod.json').read_text(encoding='utf-8-sig'))['witnesses']}
encoded=[r for r in dict.fromkeys(tuple(map(bits,row)) for row in rows) if r not in prior]
p=out/'batch-mod.json';p.write_text(json.dumps({'function':'MOD','probes':[{'probe':{'id':f'w111mod-tiny-discriminator-{i:05d}','args':r}} for i,r in enumerate(encoded)]},separators=(',',':')))
src=root/'crates/oxfunc_core/examples/w111_mod_publication.rs';(out/'research-candidate.rs').write_bytes(src.read_bytes())
(out/'manifest.json').write_text(json.dumps({'seed':seed,'rows':len(encoded),'generated_utc':datetime.now(timezone.utc).isoformat(),'purpose':'test MIN_NORMAL/16 adjustment threshold and output NUM for subnormal remainder','source_sha256':hashlib.sha256(src.read_bytes()).hexdigest(),'batch_sha256':hashlib.sha256(p.read_bytes()).hexdigest()},indent=2));print(len(encoded))
