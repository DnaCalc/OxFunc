"""Provisional typed reduction model; all coercion precedes numeric domain checks."""
import json,math,struct,sys
from pathlib import Path

class Error(Exception):pass
def main():
    run=Path(sys.argv[1]);cases=list(map(json.loads,(run/'cases/cases.jsonl').read_text(encoding='utf-8-sig').splitlines()));excel={r['case_id']:r for r in map(json.loads,(run/'outcomes/excel.jsonl').read_text(encoding='utf-8-sig').splitlines())}
    witnesses=[];withheld=[];misses=[]
    for case in cases:
        fn=case['canonical_surface_name']
        if fn not in ['GCD','LCM']:continue
        outcome=excel[case['case_id']];record={'case':case,'excel':outcome}
        if outcome['execution_status']!='ok':withheld.append(record);continue
        witnesses.append(record);fixtures={f['target']:f['value'] for f in case['cell_fixture']};numbers=[]
        def add(value,in_array=False):
            kind=value['kind']
            if kind=='reference':return add(fixtures[value['target']])
            if kind=='array':
                rows=value['rows'];is_many=len(rows)*len(rows[0])>1
                for row in rows:
                    for cell in row:add(cell,is_many)
            elif kind=='empty_cell':
                if in_array:numbers.append(0)
            elif kind=='missing_arg':pass
            elif kind=='error':raise Error(value['code'])
            elif kind=='logical':raise Error('Value')
            elif kind=='number':numbers.append(value['value'])
            elif kind=='text':
                try:numbers.append(float(value['value']))
                except ValueError:raise Error('Value')
            else:raise ValueError(kind)
        try:
            for i,arg in enumerate(case['args']):
                if i==0 and arg['kind']=='missing_arg':raise Error('NA')
                add(arg)
            if not numbers:raise Error('Value')
            if any(n<0 or n>2**53 for n in numbers):raise Error('Num')
            ints=list(map(int,numbers))
            if fn=='GCD':n=float(math.gcd(*ints))
            elif 0 in ints:n=0.0
            else:
                n=1
                for item in reversed(ints):
                    n=float((int(n)//math.gcd(int(n),item))*item)
                    if n>2**53:raise Error('Num')
            predicted='number:0x'+struct.pack('>d',n).hex()
        except Error as error:predicted='error:'+str(error)
        if predicted!=outcome['outcome']['digest_payload']:misses.append({'case_id':case['case_id'],'predicted':predicted,'excel':outcome['outcome']['digest_payload']})
    out={'source_run':run.name,'witnesses':witnesses,'withheld':withheld,'model_differences':misses}
    (run/'gcd-lcm-model.json').write_text(json.dumps(out,separators=(',',':')),encoding='utf-8');print('admitted',len(witnesses),'withheld',len(withheld),'differences',len(misses));print(misses[:10])
if __name__=='__main__':main()
