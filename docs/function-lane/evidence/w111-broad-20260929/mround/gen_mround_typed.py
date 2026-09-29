"""MROUND prepared values, origins, and shape discovery; no evaluator laziness claim."""
import itertools, json, sys
from pathlib import Path
import gen_broad_typed_20260929 as g

g.TRANCHE='w111-mround-prepared-typed-20260929'
values=[g.n(0),g.n(7),g.n(-7),g.n(2),g.b(False),g.b(True),g.t('2'),g.t(' 2 '),
        g.t('200%'),g.t('(2)'),g.t('TRUE'),g.t('x'),g.t(''),g.missing(),g.blank(),
        g.e('Div0'),g.e('Num'),g.e('NA'),g.e('Ref'),g.e('Value')]
for i,j in itertools.product(range(len(values)),repeat=2):
    g.emit('MROUND',f'pair-{i}-{j}',[values[i],values[j]],axis='mround_scalar_coercion_precedence')
for axis in [0,1]:
    for i,value in enumerate(values):
        if value['kind']=='missing_arg':continue
        args=[g.n(7),g.n(2)];args[axis]=g.r('B1')
        g.emit('MROUND',f'reference-{axis}-{i}',args,[g.fix('B1',value)],axis='mround_reference_coercion')
        if value['kind']=='empty_cell':continue
        for shape in ['unit','row','column']:
            cells={'unit':[[value]],'row':[[value,g.n(2)]],'column':[[value],[g.n(2)]]}[shape]
            args=[g.n(7),g.n(2)];args[axis]=g.a(cells)
            g.emit('MROUND',f'array-{shape}-{axis}-{i}',args,axis='mround_array_coercion')
for earlier in [g.e('Ref'),g.e('Div0'),g.t('x'),g.n(-3),g.n(0),g.n(7)]:
    for later in [g.e('Num'),g.e('NA'),g.n(2)]:
        for vertical in [False,True]:
            first=[[g.n(7),g.n(7),earlier]]
            second=[[g.n(2),later]]
            if vertical:first=list(map(list,zip(*first)));second=list(map(list,zip(*second)))
            for reverse in [False,True]:
                args=[g.a(first),g.a(second)]
                if reverse:args.reverse()
                g.emit('MROUND',f'padding-{len(g.cases)}',args,axis='mround_ordered_shape_padding')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111mroundtyped-')
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    tranche_id=g.TRANCHE,cases=g.cases,tranches=[dict(tranche_id=g.TRANCHE,case_ids=[c['case_id'] for c in g.cases])]),separators=(',',':')))
print(len(g.cases),out)
