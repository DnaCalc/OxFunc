"""Independent numerical publication probes after W111 boundary candidates."""
import hashlib,json,math,random,struct
from fractions import Fraction
from datetime import datetime,timezone
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/runs/w111-numeric-publication-heldout-20260929';batch=out/'batches';batch.mkdir(parents=True,exist_ok=True)
assert not (out/'candidate-freeze.json').exists()
seed=202609292411;rng=random.Random(seed);sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
sources=['excel_numeric/mod.rs','excel_numeric/x87.rs','functions/sqrt_fn.rs','functions/radians.rs','functions/sech.rs','functions/standardize_fn.rs','functions/phi_fn.rs','functions/normal_log_family.rs','functions/normal_dist_common.rs','functions/discrete_dist_family.rs','coercion.rs','coercion_decimal.rs','functions/adapters.rs']
hashes={}
for name in sources:
 p=g.ROOT/'crates/oxfunc_core/src'/name;q=out/'candidate'/name;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(p.read_bytes());hashes[name]=sha(p)
(out/'candidate-freeze.json').write_text(json.dumps({'frozen_utc':datetime.now(timezone.utc).isoformat(),'seed':seed,'source_sha256':hashes,'math_use':'Exact rational output midpoint squares select inputs only; Excel remains the oracle.'},indent=2))
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
decode=lambda x:struct.unpack('>d',int(x).to_bytes(8,'big'))[0]
normal=lambda:decode((rng.randrange(1,2047)<<52)|rng.getrandbits(52))
safe=lambda x:math.isfinite(x) and (x==0 or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000'
manifest={'phase':'independent_after_numeric_publication_candidates','seed':seed,'generator_sha256':sha(Path(__file__)),'batches':[]}
def save(fn,rows):
 prior=set()
 for run in ['w111-broad-20260929','w111-extended-numeric-20260929','w111-integer-publication-20260929']:
  p=g.ROOT/'smart-fuzzer/runs'/run/'answers'/('answers-'+fn.lower()+'.json')
  if p.exists():prior.update(tuple(r['args']) for r in json.loads(p.read_text(encoding='utf-8-sig'))['witnesses'])
 unique=dict.fromkeys(tuple(map(bits,row)) for row in rows if all(safe(x) for x in row));admitted=[x for x in unique if x not in prior]
 p=batch/('batch-'+fn.lower().replace('.','_')+'.json')
 p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'w111pub-heldout-{fn}-{i:06d}','args':x}} for i,x in enumerate(admitted)]},separators=(',',':')))
 manifest['batches'].append({'function':fn,'path':p.name,'rows':len(admitted),'sha256':sha(p),'prior_tuples':len(prior)})
roots=[(normal(),) for _ in range(16000)]
for _ in range(2800):
 y=math.ldexp(rng.uniform(1,2),rng.randrange(-511,511))
 mid=(Fraction(y)+Fraction(math.nextafter(y,math.inf)))/2
 try:x=float(mid*mid)
 except OverflowError:continue
 b=int(bits(x),16)
 for delta in range(-3,4):
  if 0<b+delta<0x7ff0000000000000:roots.append((decode(b+delta),))
roots += [(-normal(),) for _ in range(300)]
save('SQRT',roots)
density=[]
limit=math.sqrt(float.fromhex('0x1.fffffffffffffp1023'))
for _ in range(4500):density.append((rng.choice([-1,1])*normal(),))
for _ in range(1500):density.append((rng.uniform(-45,45),))
for center in [limit,37.6157,37.8511319]:
 b=int(bits(center),16)
 for delta in range(-300,301):
  for sign in [-1,1]:density.append((sign*decode(b+delta),))
save('PHI',density)
save('NORM.S.DIST',[(x,flag) for (x,) in density[::2] for flag in [0,1]])
save('NORM.DIST',[(x,0,1,flag) for (x,) in density[::2] for flag in [0,1]]+[(rng.uniform(-100,100),rng.uniform(-10,10),10**rng.uniform(-305,305),rng.randrange(2)) for _ in range(1200)])
save('RADIANS',[(rng.choice([-1,1])*normal(),) for _ in range(5000)]+[(rng.choice([-1,1])*math.ldexp(rng.uniform(1,128),-1022),) for _ in range(2500)])
save('STANDARDIZE',[(rng.choice([-1,1])*normal(),rng.choice([-1,1])*normal(),normal()) for _ in range(4000)]+[(rng.choice([-1,1])*math.ldexp(rng.uniform(1,128),-1022),0,rng.uniform(.01,200)) for _ in range(2500)])
save('SECH',[(rng.choice([-1,1])*rng.uniform(680,730),) for _ in range(4000)]+[(rng.uniform(-20,20),) for _ in range(1600)])
expon=[(normal(),normal(),rng.randrange(2)) for _ in range(4500)]+[(rng.uniform(60,120),rng.uniform(5,15),rng.randrange(2)) for _ in range(2000)]
save('EXPONDIST',expon);save('EXPON.DIST',expon)
(batch/'manifest.json').write_text(json.dumps(manifest,indent=2))
g.TRANCHE='w111-numeric-publication-heldout-20260929'
specs={'SQRT':[g.n(3.25)],'PHI':[g.n(-2.125)],'RADIANS':[g.n(7.875)],'SECH':[g.n(-.75)],'STANDARDIZE':[g.n(3.25),g.n(-2.125),g.n(4.875)],'NORM.S.DIST':[g.n(-2.125),g.n(0)],'NORM.DIST':[g.n(3.25),g.n(-2.125),g.n(4.875),g.n(0)],'EXPONDIST':[g.n(3.25),g.n(2.875),g.n(0)],'EXPON.DIST':[g.n(3.25),g.n(2.875),g.n(0)]}
for fn,args in specs.items():
 for axis in range(len(args)):
  for v in [g.t('3.25'),g.t('bad'),g.b(False),g.b(True),g.blank(),g.missing(),g.e('Ref'),g.e('Num'),g.a([[g.n(2.125),g.n(-.75)]]),g.a([[g.t('2.125')],[g.e('Div0')]])]:
   values=list(args);values[axis]=v;g.emit(fn,f'axis-{axis}',values)
  g.emit(fn,f'reference-{axis}',[g.r('B2:B3') if j==axis else v for j,v in enumerate(args)],[g.fix('B2',g.n(2.125)),g.fix('B3',g.blank())])
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111pub-heldout-')
(out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')))
print(sum(x['rows'] for x in manifest['batches']),'independent numeric rows;',len(g.cases),'typed')
