"""Fresh deterministic conditional typed/shape combinations after candidate freeze.

Scalar and array arguments are prepared inputs locally; this does not test
evaluator scheduling, volatile branches or unselected-expression side effects.
"""
import json,random
from pathlib import Path
import sys
import gen_broad_typed_20260929 as t

SEED=2026092916
rng=random.Random(SEED)
t.TRANCHE='w111-conditional-heldout-20260929'
pool=[t.n(x) for x in (0,-2,0.25,8)]+[t.b(False),t.b(True),t.missing()]
pool+=[t.t(x) for x in ('TRUE','FALSE','tRuE','fAlSe','1','0',' TRUE','FALSE ','','x')]
pool+=[t.e(x) for x in t.ERR]
shapes=[(0,0),(1,1),(1,2),(1,4),(2,1),(4,1),(2,3),(3,2),(3,4)]

def argument(index):
    rows,cols=rng.choice(shapes)
    v=rng.choice(pool) if rows==0 else t.a([[rng.choice(pool) for _ in range(cols)] for _ in range(rows)])
    # Reference fixtures represent blanks but cannot contain syntactic missing arguments.
    use_reference=rng.random()<0.35
    def referencesafe(x):
        if x['kind']=='array':return t.a([[referencesafe(c) for c in row] for row in x['rows']])
        return t.blank() if x['kind']=='missing_arg' else x
    if use_reference:
        v=referencesafe(v);col=chr(65+index*5)
        target=f'{col}1' if v['kind']!='array' else f'{col}1:{chr(ord(col)+cols-1)}{rows}'
        return t.r(target),[t.fix(target,v)]
    return v,[]

for fn in ['IF','IFERROR','IFNA']:
    for i in range(220):
        args=[];fixtures=[]
        arity=2 if fn!='IF' or i%11==0 else 3
        for index in range(arity):
            arg,fix=argument(index);args.append(arg);fixtures.extend(fix)
        t.emit(fn,'fresh-mixed-shape',args,fixtures,axis='conditional_array_shape')

out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
 authority='non_semantic_exploration_input',generator=Path(__file__).name,seed=SEED,
 tranche_id=t.TRANCHE,evaluation_scope='prepared_values_only_no_evaluator_laziness_claim',
 cases=t.cases,tranches=[],summary=dict(case_count=len(t.cases),surfaces_covered=3)),indent=2),encoding='utf-8')
print(len(t.cases),out)
