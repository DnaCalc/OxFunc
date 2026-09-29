"""Freeze refined date surfaces and generate independent numeric/typed holdouts."""
import hashlib,json,math,random,sys
from pathlib import Path
from gen_w111_broad_20260929 import bits
import gen_broad_typed_20260929 as g

out=Path(sys.argv[1]);batch=out/'batches';batch.mkdir(parents=True,exist_ok=True)
seed=2026092916;rng=random.Random(seed)
sources=['functions/date_fn.rs','functions/date_parts_family.rs','functions/date_week_family.rs','functions/adapters.rs','coercion.rs']
freeze={'phase':'independent_after_second_candidate','seed':seed,'source_sha256':{p:hashlib.sha256((g.ROOT/'crates/oxfunc_core/src'/p).read_bytes()).hexdigest() for p in sources}}
(out/'candidate-freeze.json').write_text(json.dumps(freeze,indent=2),encoding='utf-8')
manifest={**freeze,'generator':Path(__file__).name,'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'batches':[]}
def save(fn,rows):
 rows=list(dict.fromkeys(tuple(row) for row in rows if all(math.isfinite(x) and bits(x)!='0x8000000000000000' and (x==0 or abs(x)>=2**-1022) for x in row)))
 p=batch/('batch-'+fn.lower()+'.json')
 p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'date-heldout2-{fn}-{i:05d}','args':[bits(x) for x in row]}} for i,row in enumerate(rows)]},separators=(',',':')),encoding='utf-8')
 manifest['batches'].append({'function':fn,'path':p.name,'rows':len(rows),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
serials=[]
for _ in range(2500):
 day=rng.choice([0,59,60,rng.randrange(2958466),2958465]);second=rng.randrange(86401)
 x=day+(second-.5)/86400
 for direction in [-math.inf,math.inf]:serials.append(math.nextafter(x,direction))
serials.extend(rng.uniform(-5,2958470) for _ in range(500))
for fn in ['DAY','MONTH','YEAR','HOUR','MINUTE','SECOND','ISOWEEKNUM']:save(fn,[(x,) for x in serials])
save('DAYS',[(x,rng.choice(serials)) for x in serials])
for fn in ['EDATE','EOMONTH']:save(fn,[(x,rng.uniform(-100000,100000)) for x in serials])
for fn in ['WEEKDAY','WEEKNUM']:
 selectors=[1,2,11,12,13,14,15,16,17]+([3] if fn=='WEEKDAY' else [21])
 save(fn,[(x,rng.choice(selectors)) for x in serials])
rows=[]
for i in range(7000):
 if i%4==0:y=rng.uniform(-2**31-100,2**31+100)
 elif i%4==1:y=rng.choice([-1,1])*10**rng.uniform(1,308)
 elif i%4==2:y=rng.uniform(-32770,32770)+rng.randrange(-32768,1)*65536
 else:y=rng.uniform(-10002,10002)
 m=rng.choice([rng.uniform(-32770,32770),rng.uniform(-25,25),-32768,-32767,32766,32767])
 d=rng.choice([rng.uniform(-50000,50000),rng.uniform(-100,100),-32768,32767])
 rows.append((y,m,d))
save('DATE',rows)
(batch/'manifest.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')

g.TRANCHE='w111-date-typed-heldout2-20260929'
specs={fn:[g.n(45292.5)] for fn in ['DAY','MONTH','YEAR','HOUR','MINUTE','SECOND','ISOWEEKNUM']}
specs.update({fn:list(map(g.n,args)) for fn,args in {'DATE':[2024,2,29],'DAYS':[45293,45292],'EDATE':[45292,1],'EOMONTH':[45292,1],'WEEKDAY':[45292,2],'WEEKNUM':[45292,21]}.items()})
for fn,base in specs.items():
 for i in range(60):
  args=base.copy();fixtures=[]
  for axis in range(len(args)):
   pool=[g.t('x'),g.e('Div0'),g.e('Num'),g.e('NA'),g.b(False),g.b(True),g.t('2'),base[axis]]
   if fn in ['WEEKDAY','WEEKNUM'] and axis==1:pool += [g.n(rng.choice([1,2,11,12,13,14,15,16,17]))]
   else:pool += [g.n(rng.uniform(-10,3000000))]
   kind=rng.randrange(4)
   if kind==0:args[axis]=rng.choice(pool)
   elif kind==1:args[axis]=g.a([[rng.choice(pool),rng.choice(pool),rng.choice(pool)]])
   elif kind==2:args[axis]=g.a([[rng.choice(pool)],[rng.choice(pool)]])
   else:
    target=f'{chr(66+axis)}1';args[axis]=g.r(target);fixtures.append(g.fix(target,rng.choice(pool)))
  g.emit(fn,f'fresh-mixed-{i}',args,fixtures,axis='date_independent_mixed_broadcast')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111dateheldout2-')
(out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
print('numeric',sum(b['rows'] for b in manifest['batches']),'typed',len(g.cases))
