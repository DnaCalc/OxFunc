"""Probe lexical exponent and magnitude boundaries of complex coefficient parsing."""
import json
from pathlib import Path
import sys
import gen_broad_typed_20260929 as typed

typed.TRANCHE='w111-complex-parser-edges-20260929'
for exponent in [-1000,-325,-324,-323,-310,-309,-308,-307,-306,306,307,308,309,310,999]:
    for mantissa in ['0','1','2.22507385850720','2.22507385850721','1.79769313486230','1.79769313486231','1.79769313486232','9']:
        for sign in ['','-']:
            typed.emit('IMREAL','exponent-boundary',[typed.t(f'{sign}{mantissa}e{exponent}+0i')])
for value in ['1e308','10e307','100e306','0.1e309','0.01e310',
              '1e-308','10e-309','100e-310','0.1e-307','0.01e-306']:
    typed.emit('IMREAL','equivalent-scale',[typed.t(value)])
for ws in [' ','\t','\r','\n','\u00a0']:
    for value in [ws+'1','1'+ws,'1'+ws+'+2i','1+'+ws+'2i','1+2'+ws+'i']:
        typed.emit('IMREAL','coefficient-whitespace',[typed.t(value)])
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    authority='non_semantic_exploration_input',generator=Path(__file__).name,
    tranche_id=typed.TRANCHE,comparison_policy='exact_typed_bit_match_no_tolerance',
    cases=typed.cases,tranches=[],summary=dict(case_count=len(typed.cases),surfaces_covered=1)),indent=2),encoding='utf-8')
print(len(typed.cases),out)
