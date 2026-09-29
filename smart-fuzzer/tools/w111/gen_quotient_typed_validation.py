"""Fresh QUOTIENT typed validation after family preparation freeze."""
import copy,hashlib,json,random
from pathlib import Path
import gen_broad_typed_20260929 as g

SEED=202609291822
rng=random.Random(SEED)
g.TRANCHE='w111-quotient-typed-validation-20260929'
def val():
    return rng.choice([g.n(rng.uniform(-100,100)),g.n(0),g.n(1.7976931348623157e308),
        g.n(2.2250738585072014e-308),g.b(rng.choice([True,False])),
        g.t(rng.choice(['-0',' 3.5 ','(1.25)','% 200','-25%','x','','TRUE'])),
        g.e(rng.choice(list(g.ERR))),g.missing(),g.blank()])
for i in range(256):
    args=[val(),val()];fixtures=[]
    for axis in range(2):
        if args[axis]['kind']!='missing_arg' and rng.choice([True,False]):
            target=['B1','C1'][axis];fixtures.append(g.fix(target,args[axis]));args[axis]=g.r(target)
    g.emit('QUOTIENT',f'mixed-{i}',args,fixtures,axis='quotient_independent_typed_validation')
for i in range(32):
    args=[]
    for axis in range(2):
        rows,cols=rng.choice([(1,1),(1,2),(2,1),(2,2)])
        cells=[[val() for _ in range(cols)] for _ in range(rows)]
        for row in cells:
            for j,cell in enumerate(row):
                if cell['kind'] in ['missing_arg','empty_cell']:row[j]=g.n(0)
        args.append(cells[0][0] if (rows,cols)==(1,1) else g.a(cells))
    g.emit('QUOTIENT',f'array-{i}',args,axis='quotient_independent_broadcast_validation')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111quotientvalidation-')
out=Path('smart-fuzzer/runs/w111-broad-20260929/quotient-typed-validation.json')
out.write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
freeze={'seed':SEED,'count':len(g.cases),'phase':'before_independent_oracle_capture','input_sha256':hashlib.sha256(out.read_bytes()).hexdigest(),'source_sha256':{p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in ['crates/oxfunc_core/src/functions/quotient_fn.rs','crates/oxfunc_core/src/coercion.rs',__file__]}}
Path('docs/function-lane/evidence/w111-broad-20260929/quotient/typed-candidate-freeze.json').write_text(json.dumps(freeze,indent=2),encoding='utf-8')
print(len(g.cases),out)
