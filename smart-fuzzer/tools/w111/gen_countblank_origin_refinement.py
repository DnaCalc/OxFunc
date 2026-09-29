"""Preserve IF reference-returning origin; force numeric value with arithmetic."""
import json,random
from pathlib import Path
import gen_broad_typed_20260929 as g

def main():
    g.TRANCHE='w111-countblank-origin-refinement-20260929';rng=random.Random(202609292113)
    for i in range(48):
        number=rng.randrange(-1000000,1000001)/rng.choice([1,2,8])
        g.emit('COUNTBLANK',f'computed-number-{i}',[g.n(number)],axis='countblank_forced_numeric_value')
        case=g.cases[-1];case['formula_text']='=COUNTBLANK(IF(FALSE,A100+0,A100+0))'
    values=[g.n(0),g.n(44.5),g.t(''),g.t('x'),g.b(False),g.b(True),g.blank()]+[g.e(e) for e in g.ERR]
    for i,value in enumerate(values):
        for cond in ['TRUE','FALSE']:
            g.emit('COUNTBLANK',f'reference-{cond}-{i}',[g.r('B1')],[g.fix('B1',value)],axis='countblank_if_reference_result')
            g.cases[-1]['formula_text']=f'=COUNTBLANK(IF({cond},B1,B1))'
    for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111countoriginrefine-')
    out=Path('smart-fuzzer/runs/w111-countblank-origin-refinement-20260929');out.mkdir(parents=True,exist_ok=True)
    (out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
    print(len(g.cases))
if __name__=='__main__':main()
