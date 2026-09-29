"""Distinguish individual squared terms from their sum at normal/overflow limits.

These are independent algebraic boundary controls; no expected outcomes are
manufactured. Components are normal input values encoded as complex text.
"""
import json
from pathlib import Path
import sys
import gen_broad_typed_20260929 as typed

typed.TRANCHE='w111-imsqrt-boundaries-20260929'
for exponent, coefficients in [(-154,[0,.5,1,1.1,1.2,1.49,1.5,2]),(154,[0,.5,.9,1,1.1,1.2,1.34,1.35])]:
    for a in coefficients:
        for b in coefficients:
            for sa,sb in [(1,1),(-1,1),(1,-1),(-1,-1)]:
                value=f'{sa*a:g}e{exponent}{sb*b:+g}e{exponent}i'
                typed.emit('IMSQRT','squared-sum-boundary',[typed.t(value)])
for value in ['0','-0','0i','-0i','-1+0i','-1-0i','1-0i','-0+1i','-0-1i','-0+0i','-0-0i']:
    typed.emit('IMSQRT','signed-zero-axis',[typed.t(value)])
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    authority='non_semantic_exploration_input',generator=Path(__file__).name,tranche_id=typed.TRANCHE,
    comparison_policy='exact_typed_bit_match_no_tolerance',cases=typed.cases,
    tranches=[dict(tranche_id=typed.TRANCHE,case_ids=[c['case_id'] for c in typed.cases])],
    summary=dict(case_count=len(typed.cases),surfaces_covered=1)),indent=2),encoding='utf-8')
print(len(typed.cases),out)
