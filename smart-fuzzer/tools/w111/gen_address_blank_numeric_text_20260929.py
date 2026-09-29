"""ADDRESS missing/blank distinctions under relative style and numeric-text syntax."""
import json
from pathlib import Path
from gen_address_typed_20260929 import build,n,t,b,a,MISSING,EMPTY

cases=[]
def add(values,tag):
    fixtures=[];args=[];formula=[]
    for i,v in enumerate(values):
        if v['kind']=='missing_arg':args.append(v);formula.append('');continue
        start=1+10*i;target=f'A{start}'
        if v['kind']=='array':target=f'A{start}:{chr(64+len(v["rows"][0]))}{start+len(v["rows"])-1}'
        fixtures.append({'target':target,'value':v})
        args.append({'kind':'reference','reference_kind':'Area' if v['kind']=='array' else 'A1','target':target})
        formula.append(target)
    cases.append({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case.v0','run_id':'assigned_by_runner',
        'tranche_id':'address-blank-numeric-text-20260929','case_id':f'address-blank-numeric-text-20260929-{len(cases):04d}',
        'function_id':'FUNC.ADDRESS','canonical_surface_name':'ADDRESS','case_tag':tag,'axis':tag,
        'expected_probe_class':'reference_value_call','formula_text':'=ADDRESS('+','.join(formula)+')',
        'args':args,'cell_fixture':fixtures,'formula_cell':'J70','category':'Lookup & Reference',
        'blocked_or_deferred_lanes':[],'known_deviation_tags':[]})
for axis in range(5):
    for value in [MISSING,EMPTY,a([[EMPTY,n(0),n(1),n(4)]]),a([[EMPTY],[n(0)],[n(1)],[n(4)]])]:
        for style in [False,True]:
            values=[n(3),n(2),n(4),b(style),t('Alpha')];values[axis]=value
            add(values,'blank-omission-and-lift')
for axis in [0,1,2]:
    for text in [' 0 ',' +1 ','-1','1e0','1E+3','1,000','1.000','$1','1%','100%',
                 '(1)','1/2','1:00','1d0','true','1_0','0x10','\u00a01\u00a0','1\u00a0',' 1\t']:
        values=[n(3),n(2),n(4),b(False),t('Alpha')];values[axis]=t(text)
        add(values,'numeric-text-grammar')
result=build();result.update({'tranche_id':'address-blank-numeric-text-20260929','cases':cases,
                             'summary':{'case_count':len(cases),'surfaces_covered':1}})
out=Path('smart-fuzzer/runs/w111-broad-20260929/address-blank-numeric-text.json')
out.write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
print(len(cases),out)
