"""Fresh ATAN neighborhood probes plus repeated exact scalar/array controls."""
import json,random,struct,hashlib
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/runs/w111-atan-refinement-20260929';out.mkdir(exist_ok=True)
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
decode=lambda x:struct.unpack('>d',int(x).to_bytes(8,'big'))[0]
centers=[0x40000d685e592174,0x40001dc1079c3c90]
rows={};rng=random.Random(202609291927)
for center in centers:
 for delta in range(-512,513):
  for sign in [-1,1]:rows.setdefault(bits(sign*decode(center+delta)),'exact-miss-neighbors')
for _ in range(30000):rows.setdefault(bits(rng.uniform(-3,3)),'fresh-focused-ordinary')
for center in [0x4000000000000000,0x4003333333333333]:
 for delta in range(-512,513):
  for sign in [-1,1]:rows.setdefault(bits(sign*decode(center+delta)),'branch-neighbors')
p=out/'batch.json';assert not p.exists()
p.write_text(json.dumps({'function':'ATAN','probes':[{'probe':{'id':f'atan-refine-{i:05d}','args':[b]},'probe_region':region} for i,(b,region) in enumerate(rows.items())]},separators=(',',':')),encoding='utf-8')
g.TRANCHE='w111-atan-repeat-controls-20260929'
for repeat in range(4):
 for center in centers:
  for delta in range(-8,9):
   x=decode(center+delta)
   for sign in [-1,1]:g.emit('ATAN',f'scalar-{repeat}-{center}-{delta}-{sign}',[g.n(sign*x)])
   g.emit('ATAN',f'array-{repeat}-{center}-{delta}',[g.a([[g.n(x),g.n(-x)]])])
   g.emit('ATAN',f'reference-array-{repeat}-{center}-{delta}',[g.r('B1:B2')],[g.fix('B1:B2',g.a([[g.n(x)],[g.n(-x)]]))])
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111atanrepeat-')
(out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
(out/'manifest.json').write_text(json.dumps({'phase':'refinement_after_two_first_holdout_failures','numeric_rows':len(rows),'typed_rows':len(g.cases),'seed':202609291927,'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()},indent=2),encoding='utf-8')
print(len(rows),'numeric;',len(g.cases),'typed')
