"""Independent MOD tiny/native-remainder numeric validation after failed-first freeze."""
import json,math,struct,random,hashlib
from pathlib import Path
from datetime import datetime,timezone
root=Path(__file__).resolve().parents[3];out=root/'smart-fuzzer/runs/w111-mod-tiny-second-heldout-20260929';out.mkdir(exist_ok=True)
assert not (out/'candidate-freeze.json').exists()
rng=random.Random(202609293307);bits=lambda x:'0x'+struct.pack('>d',x).hex();decode=lambda b:struct.unpack('>d',b.to_bytes(8,'big'))[0]
prior=set()
for p in (root/'smart-fuzzer/runs').glob('w111-mod-*/answers-mod.json'):
 prior.update(tuple(r['args'])for r in json.loads(p.read_text(encoding='utf-8-sig'))['witnesses'])
rows=[]
for i in range(4500):
 de=rng.randrange(-1022,-960);d=math.ldexp(1.,de)
 if i%3:d=decode(int(bits(d),16)+rng.choice([-1,1])*rng.randint(1,4096))
 if abs(d)<2**-1022:continue
 q=math.ldexp(rng.uniform(1,2),rng.choice([0,1,8,16,24,25,26,27,32,38,40]))
 x=d*q
 for sx,sd in [(1,1),(-1,1),(1,-1),(-1,-1)]:rows.append((sx*x,sd*d))
for de in range(-1022,-972):
 d=math.ldexp(1.,de)
 for qi in [2.**26-2,2.**26-1,2.**26,2.**26+1,2.**26+2]:
  for i in range(8):
   r=math.ldexp(rng.random()*1.25,-1022);x=qi*d+r
   for sx,sd in [(1,1),(-1,1),(1,-1),(-1,-1)]:rows.append((sx*x,sd*d))
encoded=list(dict.fromkeys(tuple(map(bits,r))for r in rows if all(math.isfinite(x)and abs(x)>=2**-1022 for x in r)))
encoded=[r for r in encoded if r not in prior];rng.shuffle(encoded)
p=out/'batch-mod.json';p.write_text(json.dumps({'function':'MOD','probes':[{'probe':{'id':f'w111mod-tiny-second-heldout-{i:05d}','args':r}}for i,r in enumerate(encoded)]},separators=(',',':')),encoding='utf-8')
src=root/'smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/mod_publication_explorer.rs';(out/'research-candidate.rs').write_bytes(src.read_bytes())
(out/'candidate-freeze.json').write_text(json.dumps({'frozen_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':hashlib.sha256(src.read_bytes()).hexdigest(),'candidate_mode':14},indent=2),encoding='utf-8')
(out/'manifest.json').write_text(json.dumps({'seed':202609293307,'rows':len(encoded),'stage':'independent_frozen_research_v2','prior_overlap':0,'batch_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()},indent=2),encoding='utf-8')
print(len(encoded))
