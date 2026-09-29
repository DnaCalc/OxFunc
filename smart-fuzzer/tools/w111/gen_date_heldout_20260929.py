"""Independent date holdout, generated only after the candidate freeze."""
import hashlib,json,math,random,sys
from pathlib import Path
from gen_w111_broad_20260929 import bits
import gen_broad_typed_20260929 as g
out=Path(sys.argv[1]);batch=out/'batches';batch.mkdir(parents=True,exist_ok=True)
sources=['date_fn.rs','date_parts_family.rs','date_week_family.rs']
freeze={'source_sha256':{p:hashlib.sha256((g.ROOT/'crates/oxfunc_core/src/functions'/p).read_bytes()).hexdigest() for p in sources},'seed':2026092911,'phase':'independent_after_candidate_selection'}
(out/'candidate-freeze.json').write_text(json.dumps(freeze,indent=2),encoding='utf-8')
rng=random.Random(freeze['seed']);manifest={**freeze,'generator':Path(__file__).name,'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'batches':[]}
def safe(x):return bits(x)!='0x8000000000000000' and (x==0 or abs(x)>=2**-1022) and math.isfinite(x)
def save(fn,rows):
 rows=list(dict.fromkeys(tuple(row) for row in rows if all(safe(x) for x in row)))
 p=batch/('batch-'+fn.lower()+'.json');p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'date-heldout-{fn}-{i:05d}','args':[bits(x) for x in row]}} for i,row in enumerate(rows)]},separators=(',',':')),encoding='utf-8')
 manifest['batches'].append({'function':fn,'path':p.name,'rows':len(rows),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
serials=[rng.uniform(-2,2958468) for _ in range(1000)]
for _ in range(1200):
 d=rng.choice([rng.randrange(2958466),0,59,60,2958465]);s=rng.randrange(86401)
 x=d+(s-.5)/86400;x=math.nextafter(x,rng.choice([-math.inf,math.inf]))
 serials.append(x)
serials += [rng.choice([-1,1])*10**rng.uniform(-300,300) for _ in range(300)]
for fn in ['DAY','MONTH','YEAR','HOUR','MINUTE','SECOND','ISOWEEKNUM']:save(fn,[(x,) for x in serials])
save('DAYS',[(x,rng.choice(serials)) for x in serials])
for fn in ['EDATE','EOMONTH']:save(fn,[(x,rng.choice([rng.uniform(-120100,120100),rng.uniform(-2,2),-1e300,1e300])) for x in serials])
for fn in ['WEEKDAY','WEEKNUM']:
 valid=[1,2,11,12,13,14,15,16,17]+([3] if fn=='WEEKDAY' else [21])
 save(fn,[(x,rng.choice(valid)) for x in serials])
rows=[]
for i in range(4000):
 if i%4==0:y=rng.uniform(-2147484000,2147484000)
 elif i%4==1:y=rng.choice([-1,1])*10**rng.uniform(1,308)
 else:y=rng.uniform(-1902,10002)
 m=rng.uniform(-32770,32769);d=rng.uniform(-32771,32770)
 if i%3==0:m=rng.uniform(-25,25)
 if i%5==0:d=rng.uniform(-366,366)
 rows.append((y,m,d))
save('DATE',rows)
(batch/'manifest.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
g.TRANCHE='w111-date-typed-20260929'
specs={fn:[g.n(45292.5)] for fn in ['DAY','MONTH','YEAR','HOUR','MINUTE','SECOND','ISOWEEKNUM']}
specs.update({fn:list(map(g.n,args)) for fn,args in {'DATE':[2024,2,29],'DAYS':[45293,45292],'EDATE':[45292,1],'EOMONTH':[45292,1],'WEEKDAY':[45292,2],'WEEKNUM':[45292,21]}.items()})
for fn,args in specs.items():
 g.variants(fn,args,positions=range(len(args)))
 for i in range(len(args)):
  row=args.copy();row[i]=g.missing();g.emit(fn,f'missing-{i}',row)
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111datetyped-')
doc={'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]}
(out/'typed.json').write_text(json.dumps(doc,indent=2),encoding='utf-8')
print('numeric',sum(b['rows'] for b in manifest['batches']),'typed',len(g.cases))
