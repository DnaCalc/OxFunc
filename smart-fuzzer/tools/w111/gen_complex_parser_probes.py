"""Distinguish complex coefficient text parsing from numeric-function kernels."""
import json
from pathlib import Path
import sys
import gen_broad_typed_20260929 as typed

typed.TRANCHE='w111-complex-parser-20260929'
centers=['1.5707963267948966','3.141592653589793','1.234567890123456789',
         '1.234567890123454999','9.99999999999999999','0.0000001234567890123456789',
         '12345678901234567890','123456789012345.5','0.1234567890123456789',
         '1.0000000000000001','0.0000000000000001','0001.234567890123456789',
         '1.234567890123456789e-200','1.234567890123456789E+200',
         '2.2250738585072014e-308','4.9406564584124654e-324',
         '1.7976931348623157e308']
for center in centers:
    for sign in ('','-','+'):
        value=sign+center
        for fn,suffix in [('IMREAL',''),('IMREAL','+0i'),('IMAGINARY','i')]:
            typed.emit(fn,'coefficient-digits',[typed.t(value+suffix)])
for value in ['.1234567890123456789','1234567890123456789.','1.23456789012345e2',
              '1.234567890123456e2','1.2345678901234567e2','1e309','0e999',
              '1e-999','1e999','1e+',' 1','1 ','1 2','NaN','inf']:
    typed.emit('IMREAL','grammar-boundary',[typed.t(value+'+0i')])
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    authority='non_semantic_exploration_input',generator=Path(__file__).name,
    tranche_id=typed.TRANCHE,comparison_policy='exact_typed_bit_match_no_tolerance',
    cases=typed.cases,tranches=[],summary=dict(case_count=len(typed.cases),surfaces_covered=2)),indent=2),encoding='utf-8')
print(len(typed.cases),out)
