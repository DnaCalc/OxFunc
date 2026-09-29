"""Fresh inputs after staged multiply and reciprocal refinements."""
import hashlib,json,math,random,struct
from pathlib import Path
from datetime import datetime,timezone
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/runs/w111-radians-sech-heldout2-20260929';batch=out/'batches';batch.mkdir(parents=True,exist_ok=True)
assert not (out/'candidate-freeze.json').exists()
source={}
for name in ['functions/radians.rs','functions/sech.rs','functions/cosh.rs','excel_numeric/mod.rs','excel_numeric/x87.rs']:
 p=g.ROOT/'crates/oxfunc_core/src'/name;q=out/'candidate'/name;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(p.read_bytes());source[name]=hashlib.sha256(p.read_bytes()).hexdigest()
(out/'candidate-freeze.json').write_text(json.dumps({'source_sha256':source,'frozen_utc':datetime.now(timezone.utc).isoformat(),'phase':'independent_after_staged_arithmetic_refinement'},indent=2))
rng=random.Random(202609292512);bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
decode=lambda x:struct.unpack('>d',int(x).to_bytes(8,'big'))[0]
manifest={'phase':'independent','seed':202609292512,'batches':[]}
for fn in ['RADIANS','SECH']:
 prior=set()
 for run in ['w111-broad-20260929','w111-extended-numeric-20260929','w111-integer-publication-20260929','w111-numeric-publication-heldout-20260929']:
  p=g.ROOT/'smart-fuzzer/runs'/run/'answers'/('answers-'+fn.lower()+'.json')
  if p.exists():prior.update(w['args'][0] for w in json.loads(p.read_text(encoding='utf-8-sig'))['witnesses'])
 values=[]
 if fn=='RADIANS':
  values=[decode((rng.randrange(2)<<63)|(rng.randrange(1,2047)<<52)|rng.getrandbits(52)) for _ in range(18000)]
  values += [rng.choice([-1,1])*math.ldexp(rng.uniform(1,128),-1022) for _ in range(4000)]
 else:
  values=[rng.uniform(-730,730) for _ in range(18000)]+[rng.choice([-1,1])*rng.uniform(680,730) for _ in range(4000)]
 rows=[b for b in dict.fromkeys(map(bits,values)) if b not in prior];p=batch/('batch-'+fn.lower()+'.json')
 p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'staged-heldout2-{fn}-{i:05d}','args':[x]}} for i,x in enumerate(rows)]},separators=(',',':')))
 manifest['batches'].append({'function':fn,'path':p.name,'rows':len(rows),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
(batch/'manifest.json').write_text(json.dumps(manifest,indent=2));print(sum(x['rows'] for x in manifest['batches']),'fresh independent rows')
