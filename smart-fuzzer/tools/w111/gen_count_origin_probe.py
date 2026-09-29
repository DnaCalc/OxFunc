"""COUNT/COUNTA/COUNTBLANK origin, error, blank and omission discriminators.

Small exact number literals deliberately remain literals: COUNTBLANK can
distinguish a number value from a cell reference. Numbers are 0, 2 and 0.5,
whose decimal and binary representations are exact; no numeric ingress guess.
"""
import copy,json,random
from pathlib import Path
import gen_broad_typed_20260929 as g

SEED=202609291936
rng=random.Random(SEED)
g.TRANCHE='w111-count-origin-20260929'
def token(v):
    kind=v['kind']
    if kind=='number':return str(v['value'])
    if kind=='text':return '"'+v['value'].replace('"','""')+'"'
    if kind=='logical':return 'TRUE' if v['value'] else 'FALSE'
    if kind=='error':return g.ERR[v['code']]
    if kind=='missing_arg':return ''
    if kind=='reference':return v['target']
    if kind=='array':
        rows=v['rows'];n=0;indices=[];tokens=[]
        for row in rows:
            indices.append(','.join(str(i) for i in range(n+1,n+len(row)+1)));n+=len(row)
            tokens.extend(token(cell) for cell in row)
        return 'CHOOSE({'+(';'.join(indices))+'},'+','.join(tokens)+')'
    raise ValueError(kind)
def emit(fn,tag,args,fixtures=(),axis='count_argument_origin'):
    g.emit(fn,tag,args,fixtures,axis=axis)
    case=g.cases[-1];case['formula_text']='='+fn+'('+','.join(token(v) for v in args)+')'
    case['cell_fixture']=copy.deepcopy(list(fixtures))

values=[g.n(0),g.n(2),g.n(.5),g.b(False),g.b(True),g.t(''),g.t('2'),g.t('x'),
        g.t('TRUE'),g.t('% 200'),g.t('(2)'),g.t(' 2 ')]+[g.e(e) for e in g.ERR]
for fn in ['COUNT','COUNTA','COUNTBLANK']:
    for i,v in enumerate(values):
        emit(fn,f'direct-{i}',[v])
        emit(fn,f'reference-{i}',[g.r('B1')],[g.fix('B1',v)])
        emit(fn,f'array-{i}',[g.a([[v,g.n(2)]])])
    emit(fn,'blank-reference',[g.r('B1')],[g.fix('B1',g.blank())])
    for i in range(64):
        rows=[[rng.choice(values+[g.blank()])] for _ in range(rng.randrange(2,13))]
        target=f'B1:B{len(rows)}'
        emit(fn,f'range-{i}',[g.r(target)],[g.fix(target,g.a(rows))],axis='count_mixed_reference_scan')
for fn in ['COUNT','COUNTA']:
    emit(fn,'two-omissions',[g.missing(),g.missing()])
    for i,v in enumerate(values+[g.missing()]):
        emit(fn,f'leading-omission-{i}',[g.missing(),v])
        emit(fn,f'trailing-omission-{i}',[v,g.missing()])
    for error in g.ERR:
        emit(fn,f'error-fold-{error}',[g.t('x'),g.e(error),g.n(2)])
        emit(fn,f'reference-error-fold-{error}',[g.r('B1:B3'),g.n(2)],
             [g.fix('B1:B3',g.a([[g.t('x')],[g.e(error)],[g.n(2)]]))])
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111countorigin-')
out=Path('smart-fuzzer/runs/w111-count-origin-20260929');out.mkdir(parents=True,exist_ok=True)
(out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
print(len(g.cases),out/'typed.json')
