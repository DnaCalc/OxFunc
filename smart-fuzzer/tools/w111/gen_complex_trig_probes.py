"""Complex-axis trig expressions and independent IMSQRT ratio-overflow controls."""
import json
from pathlib import Path
import random
import sys
import gen_broad_typed_20260929 as typed

SEED=2026092909
rng=random.Random(SEED)
typed.TRANCHE='w111-complex-trig-20260929'
values=[]
for real in ['0','0.1','-0.1','1','-1','1.5707963267948966','-1.5707963267948966','3.141592653589793','10','134217727','134217728','1e100']:
    for imaginary in ['0','1e-100','1e-10','0.1','-0.1','1','-1','10','100','710']:
        values.append(real+('' if imaginary.startswith('-') else '+')+imaginary+'i')
for _ in range(80):
    real=rng.randint(-100000,100000)/1000
    imaginary=rng.randint(-10000,10000)/1000
    values.append(f'{real:g}{imaginary:+g}i')
for fn in ['IMCOS','IMSIN','IMTAN','IMCOT']:
    for value in values:
        typed.emit(fn,'complex-axis-graph',[typed.t(value)])
for real in ['1e-300','-1e-300','1e-200','-1e-200','1','-1','0','-0']:
    for imaginary in ['1e8','-1e8','1e9','-1e9','1e108','-1e108','1e109','-1e109']:
        value=real+('' if imaginary.startswith('-') else '+')+imaginary+'i'
        typed.emit('IMSQRT','ratio-overflow-control',[typed.t(value)])
        typed.emit('IMARGUMENT','ratio-overflow-control',[typed.t(value)])
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    authority='non_semantic_exploration_input',generator=Path(__file__).name,seed=SEED,tranche_id=typed.TRANCHE,
    comparison_policy='exact_typed_bit_match_no_tolerance',cases=typed.cases,
    tranches=[dict(tranche_id=typed.TRANCHE,case_ids=[c['case_id'] for c in typed.cases])],
    summary=dict(case_count=len(typed.cases),surfaces_covered=6)),indent=2),encoding='utf-8')
print(len(typed.cases),out)
