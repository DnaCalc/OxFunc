"""Fresh reference-origin controls after MROUND's origin candidate freeze."""
import copy,hashlib,json,random,sys
from pathlib import Path
import gen_broad_typed_20260929 as g
freeze=Path('docs/function-lane/evidence/w111-broad-20260929/mround/reference-candidate-freeze.json')
frozen=json.loads(freeze.read_text())
for p,h in frozen['sources'].items():assert hashlib.sha256(Path(p).read_bytes()).hexdigest()==h,p
rng=random.Random(2026092970)
g.TRANCHE='w111-mround-reference-heldout-typed-20260929'
def val():return rng.choice([g.n(rng.randrange(-200,201)/4),g.n(0),g.t('2.75'),g.t('x'),g.t(''),g.b(True),g.b(False),g.blank(),g.e(rng.choice(list(g.ERR)))])
for i in range(320):
    args=[];fixtures=[];replacements=[]
    for axis in [0,1]:
        mode=rng.choice(['range','range','unit-range','scalar','array','materialized'])
        rows,cols=(1,1) if mode in ['scalar','unit-range'] else rng.choice([(1,2),(1,4),(2,1),(4,1),(2,2),(3,2)])
        cells=[[val() for _ in range(cols)] for _ in range(rows)]
        if mode in ['array','materialized']:
            cells=[[g.n(rng.randrange(-200,201)/4) for _ in range(cols)] for _ in range(rows)]
        if mode=='scalar':args.append(rng.choice([val(),g.missing()]));continue
        if mode=='array':args.append(g.a(cells));continue
        column='B' if axis==0 else 'F';start=rng.randrange(4,15)
        target=f'{column}{start}:{chr(ord(column)+cols-1)}{start+rows-1}'
        fixtures.append(g.fix(target,g.a(cells)));args.append(g.r(target))
        if mode=='materialized':replacements.append((axis,target,g.a(copy.deepcopy(cells))))
    g.emit('MROUND',f'fresh-origin-{i}',args,fixtures,axis='mround_independent_reference_origin')
    c=g.cases[-1]
    c['formula_cell']=rng.choice([f'J{rng.randrange(4,18)}',f'K{rng.randrange(30,50)}',f'{rng.choice("BCDFGH")}30'])
    for axis,target,value in replacements:
        c['formula_text']=c['formula_text'].replace(target,'+'+target);c['args'][axis]=value
for error in [g.e('Ref'),g.e('Div0'),g.e('NA'),g.t('x'),g.b(True),g.missing(),g.n(0),g.n(-7)]:
    for unit in [False,True]:
        for reverse in [False,True]:
            for array_error in [False,True]:
                target='B5:B5' if unit else 'B5:B7';cells=[[g.n(2)]] if unit else [[g.n(2)],[g.n(3)],[g.n(4)]]
                # Explicit Missing stays a scalar; CHOOSE omission has different input semantics.
                first=g.a([[error,g.n(7)]]) if array_error and error['kind']!='missing_arg' else error
                args=[first,g.r(target)]
                if reverse:args.reverse()
                g.emit('MROUND',f'order-{len(g.cases)}',args,[g.fix(target,g.a(cells))],axis='mround_reference_error_order')
                g.cases[-1]['formula_cell']='J6'
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111mroundreferenceheldout-')
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    tranche_id=g.TRANCHE,seed=2026092970,candidate_freeze_sha256=hashlib.sha256(freeze.read_bytes()).hexdigest(),
    cases=g.cases,tranches=[dict(tranche_id=g.TRANCHE,case_ids=[c['case_id'] for c in g.cases])]),separators=(',',':')))
print(len(g.cases),out)
