"""Diagnose MROUND reference intersection versus explicit array materialization."""
import copy, json, sys
from pathlib import Path
import gen_broad_typed_20260929 as g
g.TRANCHE='w111-mround-reference-controls-typed-20260929'

def emit(target,rows,axis,cell,mode='reference'):
    args=[g.n(7),g.n(2)];args[axis]=g.r(target)
    g.emit('MROUND',f'{mode}-{axis}-{target}-{cell}',args,[g.fix(target,g.a(rows))],
        axis='mround_reference_intersection_discriminator')
    case=g.cases[-1];case['formula_cell']=cell
    if mode=='unary-plus-materialized':
        case['formula_text']=case['formula_text'].replace(target,'+'+target)
        case['args'][axis]=g.a(copy.deepcopy(rows))
    elif mode=='index-reference':
        case['formula_text']=case['formula_text'].replace(target,f'INDEX({target},0)')
    elif mode=='if-reference':
        case['formula_text']=case['formula_text'].replace(target,f'IF(TRUE,{target},{target})')

for axis in [0,1]:
    values=[5,7,9] if axis==0 else [2,3,4]
    vertical=[[g.n(v)] for v in values];horizontal=[[g.n(v) for v in values]]
    for cell in ['J1','J2','J3','J4','J20']:
        for mode in ['reference','unary-plus-materialized']:
            emit('B1:B3',vertical,axis,cell,mode)
    for cell in ['B20','C20','D20','E20','J20']:
        for mode in ['reference','unary-plus-materialized']:
            emit('B1:D1',horizontal,axis,cell,mode)
    for cell in ['J1','B20','J20']:
        for mode in ['reference','unary-plus-materialized']:
            emit('B1:C2',[[g.n(values[0]),g.n(values[1])],[g.n(values[2]),g.n(2)]],axis,cell,mode)
    for mode in ['index-reference','if-reference']:
        for cell in ['J2','J20']:emit('B1:B3',vertical,axis,cell,mode)
    for value in [g.blank(),g.t('2'),g.b(True),g.e('Div0')]:
        rows=[[g.n(2)],[value],[g.n(4)]]
        for cell in ['J2','J20']:emit('B1:B3',rows,axis,cell)
for cell in ['J1','J2','J3','J20']:
    g.emit('MROUND','two-vertical-'+cell,[g.r('B1:B3'),g.r('F1:F3')],
        [g.fix('B1:B3',g.a([[g.n(5)],[g.n(7)],[g.n(9)]])),
         g.fix('F1:F3',g.a([[g.n(2)],[g.n(3)],[g.n(4)]]))],
        axis='mround_both_reference_intersection')
    g.cases[-1]['formula_cell']=cell
for case in g.cases:
    case['case_id']=case['case_id'].replace('w111typed-','w111mroundreference-')
    case['blocked_or_deferred_lanes']=['reference intersection is discovery only; preserve caller position and origin']
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    tranche_id=g.TRANCHE,cases=g.cases,tranches=[dict(tranche_id=g.TRANCHE,case_ids=[c['case_id'] for c in g.cases])]),separators=(',',':')))
print(len(g.cases),out)
