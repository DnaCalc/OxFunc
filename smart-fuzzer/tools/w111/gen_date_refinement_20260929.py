"""Discriminate date year-width conversion, HMS operation order and typed admission."""
import hashlib,itertools,json,math,random,sys
from pathlib import Path
from gen_w111_broad_20260929 import bits
import gen_broad_typed_20260929 as g

out=Path(sys.argv[1]);batch=out/'batches';batch.mkdir(parents=True,exist_ok=True)
rng=random.Random(2026092913)
manifest={'generator':Path(__file__).name,'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'phase':'discovery_after_failed_independent_holdout','seed':2026092913,'batches':[]}
def save(fn,rows):
 rows=list(dict.fromkeys(tuple(row) for row in rows))
 p=batch/('batch-'+fn.lower()+'.json')
 p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'date-refinement-{fn}-{i:05d}','args':[bits(x) for x in row]}} for i,row in enumerate(rows)]},separators=(',',':')),encoding='utf-8')
 manifest['batches'].append({'function':fn,'path':p.name,'rows':len(rows),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
years=[]
for base in [-2**31,-2**16,-32768,0]:
 for offset in [-32768,-1901,-1900,-1,0,1,1899,1900,32767,32768,65535]:
  y=base+offset;years.extend([y-1,y,y+1,y-.000001,y-2**-23])
years.extend([9999,9999.9999999,10000,10001,32767,32768,65536,1e100,-1e100])
save('DATE',[(y,m,d) for y in years for m,d in [(1,1),(-12,1),(13,1),(32766,1),(-32768,32767),(0,-1)]])
serials=[]
for _ in range(1500):
 day=rng.choice([0,59,60,45292,2958465]);seconds=rng.randrange(86401)
 x=day+(seconds-.5)/86400
 serials.extend([math.nextafter(x,-math.inf),x,math.nextafter(x,math.inf)])
serials=[x for x in serials if x>=0]
for fn in ['HOUR','MINUTE','SECOND']:save(fn,[(x,) for x in serials])
(batch/'manifest.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')

g.TRANCHE='w111-date-typed-refinement-20260929'
values=[g.n(0),g.n(1),g.b(False),g.b(True),g.t('1'),g.t('x'),g.t(''),g.missing(),g.blank(),g.e('Div0'),g.e('Num'),g.e('NA')]
specs={'DATE':[g.n(2024),g.n(2),g.n(29)],'DAYS':[g.n(45293),g.n(45292)],'EDATE':[g.n(45292),g.n(1)],'EOMONTH':[g.n(45292),g.n(1)],'WEEKDAY':[g.n(45292),g.n(2)],'WEEKNUM':[g.n(45292),g.n(21)]}
for fn,base in specs.items():
 for i,j in itertools.product(range(len(values)),repeat=2):
  args=base.copy();args[:2]=[values[i],values[j]];g.emit(fn,f'pair-{i}-{j}',args,axis='date_error_precedence')
 for axis in range(len(base)):
  for i,value in enumerate(values):
   if value['kind']=='missing_arg':continue
   args=base.copy();args[axis]=g.r('B1');g.emit(fn,f'ref-{axis}-{i}',args,[g.fix('B1',value)],axis='date_reference_coercion')
   if value['kind']=='empty_cell':continue
   args=base.copy();args[axis]=g.a([[value,base[axis]]]);g.emit(fn,f'array-{axis}-{i}',args,axis='date_array_coercion')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111daterefine-')
(out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
print('numeric',sum(b['rows'] for b in manifest['batches']),'typed',len(g.cases))
