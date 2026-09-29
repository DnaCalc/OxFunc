"""Distinguish COUNTBLANK's parser reference admission from computed values."""
import json
from pathlib import Path
import gen_broad_typed_20260929 as g

g.TRANCHE='w111-countblank-computed-20260929'
values=[g.n(0),g.n(2),g.n(.5),g.b(False),g.b(True),g.t(''),g.t('2'),g.t('x'),
        g.t('TRUE'),g.t('% 200'),g.t('(2)'),g.t(' 2 ')]+[g.e(e) for e in g.ERR]
def token(v):
    k=v['kind']
    if k=='number':return str(v['value'])
    if k=='text':return '"'+v['value'].replace('"','""')+'"'
    if k=='logical':return 'TRUE' if v['value'] else 'FALSE'
    return g.ERR[v['code']]
for i,value in enumerate(values):
    text=token(value)
    for wrapper,expr in [('if',f'IF(TRUE,{text},{text})'),('choose',f'CHOOSE(1,{text})')]:
        g.emit('COUNTBLANK',f'{wrapper}-{i}',[value],axis='countblank_computed_scalar_admission')
        case=g.cases[-1];case['formula_text']=f'=COUNTBLANK({expr})';case['cell_fixture']=[]
    g.emit('COUNTBLANK',f'array-{i}',[g.a([[value,g.t('')],[g.e('Num'),g.n(2)]])],axis='countblank_computed_array_admission')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111countcomputed-')
out=Path('smart-fuzzer/runs/w111-countblank-computed-20260929');out.mkdir(parents=True,exist_ok=True)
(out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
print(len(g.cases),out/'typed.json')
