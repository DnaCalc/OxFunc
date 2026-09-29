"""Fresh typed MROUND controls generated after prepared-candidate freeze."""
import hashlib, json, random, sys
from pathlib import Path
import gen_broad_typed_20260929 as g

freeze=Path('docs/function-lane/evidence/w111-broad-20260929/mround/prepared-candidate-freeze.json')
frozen=json.loads(freeze.read_text())
for path,digest in frozen['sources'].items():assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,path
rng=random.Random(2026092969)
g.TRANCHE='w111-mround-prepared-heldout-typed-20260929'
def scalar(allow_missing=True,allow_blank=True):
    values=[g.n(rng.randrange(-800,801)/8),g.b(rng.choice([False,True])),
        g.t(rng.choice([' 2.75 ','-0','350%','(1.25)','+.5','TRUE','x','','0.499999999999995'])),
        g.e(rng.choice(list(g.ERR)))]
    if allow_missing:values.append(g.missing())
    if allow_blank:values.append(g.blank())
    return rng.choice(values)
for i in range(432):
    args=[];fixtures=[]
    for axis in [0,1]:
        ref=rng.choice([False,True])
        if i>=192:
            rows,cols=rng.choice([(1,1),(1,3),(3,1),(2,2),(2,3),(3,2)])
            data=[[scalar(False,ref) for _ in range(cols)] for _ in range(rows)]
            value=g.a(data)
        else:rows,cols=1,1;value=scalar()
        if ref and value['kind']!='missing_arg':
            column='B' if axis==0 else 'F'
            endpoint=chr(ord(column)+cols-1)
            target=column+'1' if (rows,cols)==(1,1) else f'{column}1:{endpoint}{rows}'
            # A unit cell fixture stores its scalar cell, not a matrix carrier.
            stored=value['rows'][0][0] if value['kind']=='array' and (rows,cols)==(1,1) else value
            fixtures.append(g.fix(target,stored));args.append(g.r(target))
        else:args.append(value)
    g.emit('MROUND',f'fresh-{i}',args,fixtures,axis='mround_independent_prepared_origins_shapes')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111mroundheldout-')
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    tranche_id=g.TRANCHE,candidate_freeze_sha256=hashlib.sha256(freeze.read_bytes()).hexdigest(),
    seed=2026092969,cases=g.cases,tranches=[dict(tranche_id=g.TRANCHE,case_ids=[c['case_id'] for c in g.cases])]),separators=(',',':')))
print(len(g.cases),out)
