"""Black-box discovery for integer conversion, overflow and tiny-result boundaries."""
import hashlib,json,math,random,struct
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/runs/w111-integer-publication-20260929';batch=out/'batches';batch.mkdir(parents=True,exist_ok=True)
seed=202609292147;rng=random.Random(seed)
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
decode=lambda x:struct.unpack('>d',int(x).to_bytes(8,'big'))[0]
manifest={'phase':'discovery_not_holdout','seed':seed,'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'batches':[]}
def safe(x):return math.isfinite(x) and (x==0 or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000'
def save(fn,rows):
 rows=list(dict.fromkeys(tuple(map(float,a)) for a in rows if all(safe(x) for x in a)))
 p=batch/('batch-'+fn.lower().replace('.','_')+'.json');assert not p.exists()
 p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'w111boundary-{fn}-{i:05d}','args':list(map(bits,a))}} for i,a in enumerate(rows)]},separators=(',',':')),encoding='utf-8')
 manifest['batches'].append({'function':fn,'path':p.name,'rows':len(rows),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
integers=[]
for n in [-2**53,-2**32,-1e10,-9485,-100,-3,-2,-1,0,1,2,3,100,9485,1e10,2**32,2**53]:
 for direction in [-math.inf,math.inf]:
  x=float(n)
  for _ in range(40):integers.append(x);x=math.nextafter(x,direction)
 for exponent in range(-60,-10):
  for sign in [-1,1]:integers.append(n+sign*max(1,abs(n))*2**exponent)
integers += [rng.uniform(-1e6,1e6) for _ in range(1000)]
integers += [decode((rng.randrange(2)<<63)|(rng.randrange(1,2047)<<52)|rng.getrandbits(52)) for _ in range(1000)]
for fn in ['INT','ISEVEN','ISODD']:save(fn,[(x,) for x in integers])
save('TRUNC',[(x,rng.choice([-20,-16,-15,-4,-1,0,1,2,14,15,16,20,308,309,32767,32768,2**31,2**32])) for x in integers])
roots=[decode((rng.randrange(1,2047)<<52)|rng.getrandbits(52)) for _ in range(5000)]
for exponent in [-1022,-512,-54,-2,0,2,54,512,1023]:
 for direction in [-math.inf,math.inf]:
  x=math.ldexp(1.,exponent)
  for _ in range(40):roots.append(x);x=math.nextafter(x,direction)
roots += [-1.,-1e300,0.,float.fromhex('0x1.fffffffffffffp1023')]
save('SQRT',[(x,) for x in roots])
limit=math.sqrt(float.fromhex('0x1.fffffffffffffp1023'))
gauss=[rng.uniform(-50,50) for _ in range(2000)]
for center in [limit,1e154,2**512,37.6157,37.8511319,0.,1.]:
 for direction in [-math.inf,math.inf]:
  x=center
  for _ in range(40):
   gauss.extend([x,-x]);x=math.nextafter(x,direction)
gauss += [rng.choice([-1,1])*10**rng.uniform(150,308) for _ in range(500)]
save('PHI',[(x,) for x in gauss])
save('NORM.S.DIST',[(x,0) for x in gauss[::4]])
save('NORM.DIST',[(x,0,1,0) for x in gauss[::4]])
tiny=[math.ldexp(rng.uniform(1,2),rng.randrange(-1022,-1000)) for _ in range(1000)]
tiny += [2**-1022*n for n in range(1,101)]
save('RADIANS',[(sign*x,) for x in tiny for sign in [-1,1]])
save('STANDARDIZE',[(sign*x,0,rng.uniform(.1,100)) for x in tiny for sign in [-1,1]]+[(1,math.nextafter(1,0),10**rng.uniform(280,308)) for _ in range(300)]+[(1e308,-1e308,2),(1e308,-1e308,1e308)])
save('SECH',[(rng.choice([-1,1])*rng.uniform(690,720),) for _ in range(1600)])
expon=[(rng.uniform(60,120),rng.uniform(5,15),0) for _ in range(1600)]
expon += [(10**rng.uniform(-300,300),10**rng.uniform(-300,300),rng.choice([0,1])) for _ in range(400)]
for fn in ['EXPONDIST','EXPON.DIST']:save(fn,expon)
perm=[]
for n in [-2,-1.999,-1,-.999,-.1,0,.1,.999,1,1.999,2,2.999,3,10,100,2**31-1,2**31,2**32,2**53,1e100]:
 for k in [-2,-1.999,-1,-.999,-.1,0,.1,.999,1,1.999,2,3,4,10,100,2**31-1,2**31,2**32-1,2**32,2**53]:perm.append((n,k))
perm += [(rng.uniform(-2,180),rng.uniform(-2,180)) for _ in range(1800)]
save('PERMUTATIONA',perm)
comb=[]
for n in [2**31-1,2**31,2**32-1,2**32,2**53-2,2**53-1,2**53,2**53+2,2**63,1e100]:
 for m in [0,1,2,3,5,7,10,n]:comb.extend([(n,m),(m,n),(n,m,0),(0,n,m)])
comb += [tuple(rng.uniform(0,10000) for _ in range(rng.randrange(1,7))) for _ in range(1800)]
for fn in ['LCM','GCD']:save(fn,comb)
(batch/'manifest.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
g.TRANCHE='w111-integer-publication-typed-20260929'
specs={'INT':[1.5],'ISEVEN':[2.5],'ISODD':[2.5],'TRUNC':[2.75,1],'SQRT':[2],'PHI':[1],'RADIANS':[180],'STANDARDIZE':[3,1,2],'SECH':[1],'EXPONDIST':[1,2,0],'EXPON.DIST':[1,2,0],'PERMUTATIONA':[3,2],'LCM':[6,8],'GCD':[6,8]}
for fn,args in specs.items():
 base=list(map(g.n,args));g.variants(fn,base,positions=range(len(base)))
 for axis in range(len(base)):
  for tag,value in [('missing',g.missing()),('false',g.b(False)),('bad',g.t('x')),('error',g.e('Div0'))]:
   row=base.copy();row[axis]=value;g.emit(fn,f'{tag}-{axis}',row)
  g.emit(fn,f'reference-{axis}',[g.r('B1') if j==axis else x for j,x in enumerate(base)],[g.fix('B1',g.t('2'))])
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111integerpub-')
(out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
print(sum(b['rows'] for b in manifest['batches']),'numeric;',len(g.cases),'typed; functions',len(manifest['batches']))
