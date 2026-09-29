"""Frozen research candidate test: power-of-two reduction versus general divisor."""
import hashlib,json,math,random,struct
from pathlib import Path
from datetime import datetime,timezone
root=Path(__file__).resolve().parents[3];out=root/'smart-fuzzer/runs/w111-mod-tiny-heldout-20260929';out.mkdir(exist_ok=True);assert not (out/'manifest.json').exists()
seed=202609293305;rng=random.Random(seed);bits=lambda x:'0x'+struct.pack('>d',float(x)).hex();decode=lambda b:struct.unpack('>d',b.to_bytes(8,'big'))[0]
src=root/'crates/oxfunc_core/examples/w111_mod_publication.rs';(out/'research-candidate.rs').write_bytes(src.read_bytes())
(out/'candidate-freeze.json').write_text(json.dumps({'frozen_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':hashlib.sha256(src.read_bytes()).hexdigest(),'candidate_mode':8},indent=2))
rows=[]
for _ in range(6500):
    exp=rng.randrange(1,53);db=exp<<52
    d=decode(db if rng.random()<.7 else db+rng.randrange(-1000,1001))
    if d<2**-1022:continue
    x=decode(((exp+rng.randrange(0,39))<<52)|rng.getrandbits(52))
    if x/d>=1125900000000.:continue
    rows.extend((s*x,t*d) for s in [-1,1] for t in [-1,1])
for exp in range(1,54):
    d=decode(exp<<52)
    for multiplier in [1,2,3,7,15,63]:
        center=d*multiplier
        for delta in [-3,-2,-1,0,1,2,3]:
            x=decode(int(bits(center),16)+delta)
            if x>=2**-1022:rows.extend((s*x,t*d) for s in [-1,1] for t in [-1,1])
prior=set()
for stage in ['boundary','discriminator']:
    prior.update(tuple(r['args']) for r in json.loads((root/f'smart-fuzzer/runs/w111-mod-tiny-{stage}-20260929/answers-mod.json').read_text(encoding='utf-8-sig'))['witnesses'])
encoded=[r for r in dict.fromkeys(tuple(map(bits,row)) for row in rows) if r not in prior]
p=out/'batch-mod.json';p.write_text(json.dumps({'function':'MOD','probes':[{'probe':{'id':f'w111mod-tiny-heldout-{i:05d}','args':r}} for i,r in enumerate(encoded)]},separators=(',',':')))
(out/'manifest.json').write_text(json.dumps({'seed':seed,'rows':len(encoded),'stage':'independent_frozen_research','batch_sha256':hashlib.sha256(p.read_bytes()).hexdigest()},indent=2));print(len(encoded))
