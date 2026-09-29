"""ADDRESS sheet reference-token grammar and escaped-length discriminators."""
import json
from pathlib import Path
from gen_address_typed_20260929 import build,n,t,b

names=[]
for row in ['', '0','00','1','01','1048576','1048577']:
    for col in ['', '0','00','1','01','16384','16385','1048576','1048577','999999999999999999']:
        names.append('R'+row+'C'+col)
for escaped in range(252,261):
    for quotes in [1,2,3,126,127,128]:
        if escaped>=2*quotes:
            names.append('x'*(escaped-2*quotes)+"'"*quotes)
cases=[]
for name in names:
    for style,mode,row,col in [(True,1,1048576,16384),(False,4,-1048575,-16383)]:
        values=[n(row),n(col),n(mode),b(style),t(name)]
        fixtures=[{'target':f'A{1+10*i}','value':v} for i,v in enumerate(values)]
        cases.append({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case.v0',
                      'run_id':'assigned_by_runner','tranche_id':'address-tokens-20260929',
                      'case_id':f'address-tokens-20260929-{len(cases):04d}',
                      'function_id':'FUNC.ADDRESS','canonical_surface_name':'ADDRESS',
                      'case_tag':'reference-token-and-escaped-length','axis':'sheet_text',
                      'expected_probe_class':'reference_value_call',
                      'formula_text':'=ADDRESS(A1,A11,A21,A31,A41)',
                      'args':[{'kind':'reference','reference_kind':'A1','target':x['target']} for x in fixtures],
                      'cell_fixture':fixtures,'formula_cell':'J70','category':'Lookup & Reference',
                      'blocked_or_deferred_lanes':[],'known_deviation_tags':[]})
result=build();result.update({'tranche_id':'address-tokens-20260929','cases':cases,
                             'summary':{'case_count':len(cases),'surfaces_covered':1}})
out=Path('smart-fuzzer/runs/w111-broad-20260929/address-tokens.json')
out.write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
print(len(cases),out)
