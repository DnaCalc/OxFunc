"""Retain exact typed generic number-to-text observations; exclude COMPLEX controls."""
import hashlib,json,sys
from pathlib import Path

def main():
    run=Path(sys.argv[1]);out=Path(sys.argv[2])
    cases=[json.loads(line) for line in (run/'cases/cases.jsonl').read_text(encoding='utf-8-sig').splitlines()]
    outcomes={r['case_id']:r for r in map(json.loads,(run/'outcomes/excel.jsonl').read_text(encoding='utf-8-sig').splitlines())}
    witnesses=[];withheld=[];controls=[]
    for case in cases:
        row={'case':case,'excel':outcomes[case['case_id']]}
        if case['canonical_surface_name']=='COMPLEX':controls.append(row)
        elif row['excel']['execution_status']!='ok':withheld.append(row)
        else:witnesses.append(row)
    report={'source_run':run.name,'scope_completeness':'scope_partial','target_completeness':'target_partial','integration_completeness':'partial','open_lanes':['locale/context','unexercised consumers','upstream acknowledgement'],'witnesses':witnesses,'withheld':withheld,'distinct_formatter_controls':controls,'source_sha256':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [run/'cases/cases.jsonl',run/'outcomes/excel.jsonl']}}
    out.write_text(json.dumps(report,separators=(',',':')),encoding='utf-8');print('retained',len(witnesses),'withheld',len(withheld),'COMPLEX_controls',len(controls))
if __name__=='__main__':main()
