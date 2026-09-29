"""Compare retained typed local/oracle outcomes exactly, keeping every difference."""
import collections,json
from pathlib import Path
import sys

def records(path):
    rows={}
    for line in path.read_text(encoding='utf-8-sig').split('\n'):
        if not line.strip():
            continue
        row=json.loads(line); key=row['case_id']; assert key not in rows; rows[key]=row
    return rows

def main(case_path,oracle_path,local_path,out):
    cases=json.loads(case_path.read_text(encoding='utf-8-sig'))['cases']
    oracle=records(oracle_path); local=records(local_path)
    misses=[];counts=collections.defaultdict(lambda:dict(rows=0,matches=0))
    for case in cases:
        key=case['case_id']; e=oracle[key]; a=local[key]
        assert case['function_id']==e['function_id']==a['function_id']
        assert case['formula_text']==e['formula_text']==a['formula_text']
        expected=e.get('outcome',{}).get('digest_payload')
        actual=a.get('outcome',{}).get('digest_payload')
        same=e['execution_status']=='ok' and a['execution_status']=='ok' and actual==expected
        counts[case['function_id']]['rows']+=1
        counts[case['function_id']]['matches']+=same
        if not same:
            misses.append(dict(case_id=key,formula=case['formula_text'],expected=expected,actual=actual,
                               excel_execution=e['execution_status'],local_execution=a['execution_status']))
    report=dict(rows=len(cases),matches=len(cases)-len(misses),by_function=dict(counts),misses=misses)
    out.write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(json.dumps({k:v for k,v in report.items() if k!='misses'}))
    for row in misses[:20]:print(json.dumps(row))

if __name__=='__main__': main(*map(Path,sys.argv[1:]))
