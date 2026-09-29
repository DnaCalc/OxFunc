"""QUOTIENT logical/origin and argument error precedence discriminators."""
import itertools,json
from pathlib import Path
import gen_broad_typed_20260929 as g

g.TRANCHE='w111-quotient-typed-refinement-20260929'
values=[g.n(0),g.n(3),g.b(False),g.b(True),g.t('3'),g.t('x'),g.t(''),g.missing(),g.blank(),g.e('Div0'),g.e('Num'),g.e('NA')]
for i,j in itertools.product(range(len(values)),repeat=2):
    g.emit('QUOTIENT',f'pair-{i}-{j}',[values[i],values[j]],axis='quotient_error_precedence')
for axis in [0,1]:
    for i,value in enumerate(values):
        if value['kind']=='missing_arg':continue
        args=[g.n(7),g.n(2)];args[axis]=g.r('B1')
        g.emit('QUOTIENT',f'reference-{axis}-{i}',args,[g.fix('B1',value)],axis='quotient_reference_coercion')
    for i,value in enumerate(values):
        if value['kind'] in ('missing_arg','empty_cell'):continue
        args=[g.n(7),g.n(2)];args[axis]=g.a([[value,g.n(2)]])
        g.emit('QUOTIENT',f'array-{axis}-{i}',args,axis='quotient_array_coercion')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111quotientrefine-')
out=Path('smart-fuzzer/runs/w111-broad-20260929/quotient-typed-refinement.json')
out.write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
print(len(g.cases),out)
