"""Probe ADDRESS's implicit numeric-text grammar with verified typed ingress."""
import json
from pathlib import Path
from gen_address_typed_20260929 import build, n, t, b

texts = ['1', '0', '-1', '+1', '.5', '1.', '1e2', '1E-2', '1e+2',
         '1e', '1d2', 'Infinity', 'NaN', '0x1', '1_0', '１', '١',
         '1,000', '1,5', '1 000', '$1', 'R1', '€1',
         '1%', '100%', '.5%', '1e2%', '%1', '1%%', '1% ', ' 1%',
         '1 %', '1%2', '1% %', '%', '(1)', '(0)', '(+1)', '(-1)',
         '(1.5)', '( 1 )', '(1)%', '(1%)', '(100%)', '(1e2)',
         '(1) ', ' (1)', '(1)(2)', '-(1)', '+(1)', '--1', '++1',
         '1-', '1+', '1:00', '12:30', '1:30:45', '0:00:00',
         '25:00', '0:60', '1:00 PM', '1:00AM', '12:00am',
         '1/2', '1/2/2024', '2024/1/2', '2024-01-02', '2-Jan-2024',
         'Jan 2 2024', '31/1/2024', '2024-01-02 12:00', '1/2/24',
         '2/29/1900', '2/29/2000', '2/29/2100', '1/2/0001']
for whitespace in [' ', '\t', '\n', '\r', '\v', '\f', '\u00a0', '\u2003', '\u3000']:
    texts += [whitespace+'1', '1'+whitespace, whitespace+'1'+whitespace,
              '1'+whitespace+'%', '('+whitespace+'1)']
texts = list(dict.fromkeys(texts))
cases=[]
for text in texts:
    for axis in range(3):
        for direct in [False, True]:
            values=[n(3),n(2),n(4),b(False),t('Alpha')];values[axis]=t(text)
            fixtures=[];args=[];formula=[]
            for i,value in enumerate(values):
                if direct and i == axis:
                    args.append(value);formula.append('"'+text.replace('"','""')+'"')
                else:
                    target=f'A{1+10*i}'
                    args.append({'kind':'reference','reference_kind':'A1','target':target})
                    fixtures.append({'target':target,'value':value});formula.append(target)
            cases.append({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case.v0',
                'run_id':'assigned_by_runner','tranche_id':'address-numeric-text-syntax-20260929',
                'case_id':f'address-numeric-text-syntax-20260929-{len(cases):04d}',
                'function_id':'FUNC.ADDRESS','canonical_surface_name':'ADDRESS',
                'case_tag':'numeric-text-implicit-coercion','axis':'numeric-text-grammar',
                'expected_probe_class':'reference_value_call','formula_text':'=ADDRESS('+','.join(formula)+')',
                'args':args,'cell_fixture':fixtures,'formula_cell':'J70','category':'Lookup & Reference',
                'blocked_or_deferred_lanes':[],'known_deviation_tags':[]})
result=build();result.update({'tranche_id':'address-numeric-text-syntax-20260929','cases':cases,
                             'summary':{'case_count':len(cases),'surfaces_covered':1}})
out=Path('smart-fuzzer/runs/w111-broad-20260929/address-numeric-text-syntax.json')
out.write_text(json.dumps(result,ensure_ascii=False,separators=(',',':')),encoding='utf-8')
print(len(cases),out)
