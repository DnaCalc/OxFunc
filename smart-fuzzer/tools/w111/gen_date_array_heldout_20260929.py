"""Fresh date-part array error probes after per-cell publication repair."""
import hashlib,json,random,sys
from pathlib import Path
import gen_broad_typed_20260929 as g
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
rng=random.Random(2026092917);sources={}
for rel in ['functions/date_parts_family.rs','coercion.rs','coercion_decimal.rs']:
 p=g.ROOT/'crates/oxfunc_core/src'/rel
 dest=out/'candidate'/rel;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(p.read_bytes())
 sources[rel]=hashlib.sha256(p.read_bytes()).hexdigest()
freeze={'seed':2026092917,'phase':'independent_after_date_part_array_repair','source_sha256':sources,'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
(out/'candidate-freeze.json').write_text(json.dumps(freeze,indent=2),encoding='utf-8')
g.TRANCHE='w111-date-array-heldout-20260929'
for fn in ['DAY','MONTH','YEAR','HOUR','MINUTE','SECOND']:
 for i in range(120):
  rows,cols=rng.choice([(1,3),(3,1),(2,3),(3,2)])
  pool=[g.n(rng.uniform(-100,2958470)),g.n(rng.uniform(0,1)),g.b(False),g.b(True),g.t('x'),g.t(''),g.t(' 2 '),g.t('200%'),g.e('Div0'),g.e('Num'),g.e('NA')]
  values=[[rng.choice(pool) for _ in range(cols)] for _ in range(rows)]
  if i%2:
   # Every referenced cell is declared separately, preserving empty-cell origin.
   fixtures=[]
   for r in range(rows):
    for c in range(cols):
     value=values[r][c] if rng.randrange(8) else g.blank()
     fixtures.append(g.fix(f'{chr(66+c)}{r+1}',value))
   target=f'B1:{chr(65+cols)}{rows}'
   g.emit(fn,f'fresh-reference-array-{i}',[g.r(target)],fixtures,axis='date_array_error_publication')
  else:g.emit(fn,f'fresh-direct-array-{i}',[g.a(values)],axis='date_array_error_publication')
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111datearray-')
(out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
print(len(g.cases))
