"""Fresh independent INT and parity inputs after frozen boundary candidates."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/runs/w111-integer-heldout-20260929';batch=out/'batches';batch.mkdir(parents=True,exist_ok=True)
assert not (out/'candidate-freeze.json').exists()
seed=202609292610;rng=random.Random(seed);sources={}
for name in ['functions/int_fn.rs','functions/iseven_fn.rs','functions/is_predicates_family.rs','functions/round_fn.rs','coercion.rs','coercion_decimal.rs','functions/adapters.rs']:
 p=g.ROOT/'crates/oxfunc_core/src'/name;q=out/'candidate'/name;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(p.read_bytes());sources[name]=hashlib.sha256(p.read_bytes()).hexdigest()
(out/'candidate-freeze.json').write_text(json.dumps({'source_sha256':sources,'frozen_utc':datetime.now(timezone.utc).isoformat(),'seed':seed},indent=2))
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
decode=lambda x:struct.unpack('>d',int(x).to_bytes(8,'big'))[0]
safe=lambda x:math.isfinite(x) and (x==0 or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000'
manifest={'phase':'independent_after_integer_candidates','seed':seed,'batches':[]}
for fn in ['INT','ISEVEN','ISODD']:
 prior=set()
 for run in ['w111-broad-20260929','w111-integer-publication-20260929','w111-integer-refinement-20260929']:
  p=g.ROOT/'smart-fuzzer/runs'/run/'answers'/('answers-'+fn.lower()+'.json')
  if p.exists():prior.update(r['args'][0] for r in json.loads(p.read_text(encoding='utf-8-sig'))['witnesses'])
 values=[decode((rng.randrange(2)<<63)|(rng.randrange(1,2047)<<52)|rng.getrandbits(52)) for _ in range(10000)]
 for _ in range(500):
  n=rng.randrange(1,10**rng.randrange(1,16))
  center=n+rng.choice([-.5,-.25,.25,.5]) if fn=='INT' else n-1e-10
  b=int(bits(center),16)
  for delta in range(-12,13):
   for sign in [-1,1]:values.append(sign*decode(b+delta))
 values += [rng.uniform(-1e6,1e6) for _ in range(3000)]
 rows=[b for b in dict.fromkeys(bits(x) for x in values if safe(x)) if b not in prior]
 p=batch/('batch-'+fn.lower()+'.json');p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'integer-heldout-{fn}-{i:06d}','args':[b]}} for i,b in enumerate(rows)]},separators=(',',':')))
 manifest['batches'].append({'function':fn,'path':p.name,'rows':len(rows),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'prior_inputs':len(prior)})
(batch/'manifest.json').write_text(json.dumps(manifest,indent=2))
g.TRANCHE='w111-integer-heldout-20260929'
for fn in ['INT','ISEVEN','ISODD']:
 for v in [g.n(.9999999999),g.n(-9484.9999999999),g.n(962306618417396.5),g.n(-962306618417396.5),g.n(1e100),g.t(' -2.75 '),g.t('TRUE'),g.t('(25%)'),g.blank(),g.b(False),g.b(True),g.e('Ref'),g.a([[g.n(.9999999999),g.e('Num')],[g.t('2.75'),g.t('bad')]])]:
  g.emit(fn,'direct',[v])
  if v['kind']!='array':g.emit(fn,'reference',[g.r('B2')],[g.fix('B2',v)])
 g.emit(fn,'reference-array',[g.r('B2:B4')],[g.fix('B2',g.n(1e100)),g.fix('B3',g.blank()),g.fix('B4',g.t('2.75'))])
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111integer-heldout-')
(out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')))
print(sum(r['rows'] for r in manifest['batches']),'fresh numeric;',len(g.cases),'typed')
