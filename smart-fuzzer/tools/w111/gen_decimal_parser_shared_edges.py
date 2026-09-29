"""Independent zero/decimal-position and shared-conversion controls for ABS and IMREAL."""
import json
from pathlib import Path
import sys
import gen_broad_typed_20260929 as typed

typed.TRANCHE='w111-decimal-parser-shared-edges-20260929'
values=[]
for exponent in [-310,-309,-308,-307,307,308,309,310]:
    for mantissa in ['0','00','0000','0.0','0.00','000.0','000.000']:
        for sign in ['','-']:
            values.append((f'{sign}{mantissa}e{exponent}','zero-position'))
for exponent in [-310,-309,-308,307,308,309]:
    for mantissa in ['1','01','0.1','001.00','000.01']:
        for sign in ['','-']:
            values.append((f'{sign}{mantissa}e{exponent}','nonzero-position'))
for magnitude in ['9.35037643088324e-128','935037643088324e-142','0.000935037643088324e-124',
                  '9.35037643088324000000e-128','9.35037643088324999999e-128']:
    for sign in ['','-']:
        values.append((sign+magnitude,'conversion-residual'))
for value,tag in values:
    typed.emit('ABS',tag,[typed.t(value)])
    typed.emit('IMREAL',tag,[typed.t(value+'+0i')])
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    authority='non_semantic_exploration_input',generator=Path(__file__).name,
    tranche_id=typed.TRANCHE,comparison_policy='exact_typed_bit_match_no_tolerance',
    cases=typed.cases,tranches=[],summary=dict(case_count=len(typed.cases),surfaces_covered=2)),indent=2),encoding='utf-8')
print(len(typed.cases),out)
