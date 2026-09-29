"""Compare explicit serializers to captured generic LEFT text, not a model."""
import json,struct
from collections import Counter
from pathlib import Path

def bits(n):return struct.pack('>d',n).hex()
def main():
    generic=json.loads(Path('docs/function-lane/evidence/w111-broad-20260929/numeric-to-text/discovery.json').read_text(encoding='utf-8'))
    text={bits(w['case']['args'][0]['value']):w['excel']['outcome']['value'] for w in generic['witnesses'] if w['case']['canonical_surface_name']=='LEFT'}
    run=Path('smart-fuzzer/runs/w111-explicit-number-text-typed-20260929')
    cases=list(map(json.loads,(run/'cases/cases.jsonl').read_text(encoding='utf-8-sig').splitlines()))
    excel={row['case_id']:row for row in map(json.loads,(run/'outcomes/excel.jsonl').read_text(encoding='utf-8-sig').splitlines())}
    counts=Counter();misses=[];witnesses=[];controls=[];withheld=[]
    def cell(v,strict):
        if v['kind']=='number':return text[bits(v['value'])]
        if v['kind']=='logical':return 'TRUE' if v['value'] else 'FALSE'
        if v['kind']=='text':return '"'+v['value'].replace('"','""')+'"' if strict else v['value']
        raise ValueError(v)
    for case in cases:
        out=excel[case['case_id']];fn=case['canonical_surface_name'];row={'case':case,'excel':out}
        if fn=='TEXT':controls.append(row);continue
        if out['execution_status']!='ok':withheld.append(row);continue
        witnesses.append(row);strict=bool(case['args'][1]['value']);arg=case['args'][0]
        if fn=='VALUETOTEXT':
            if arg['kind']=='array':
                rows=arg['rows'];expected=f'array:{len(rows)}x{len(rows[0])}:['+'|'.join('text:'+cell(v,strict) for r in rows for v in r)+']'
            else:expected='text:'+cell(arg,strict)
        else:
            rows=arg['rows'];s='{'+(';'.join(','.join(cell(v,True) for v in r) for r in rows))+'}' if strict else ', '.join(cell(v,False) for r in rows for v in r)
            expected='text:'+s
        if out['outcome']['digest_payload']==expected:counts[fn]+=1
        else:misses.append({'case_id':case['case_id'],'expected_from_generic_capture':expected,'excel':out['outcome']})
    dest=Path('docs/function-lane/evidence/w111-broad-20260929/numeric-to-text')
    (dest/'explicit-serializers.json').write_text(json.dumps({'witnesses':witnesses,'withheld':withheld,'distinct_TEXT_General_controls':controls},separators=(',',':')),encoding='utf-8')
    (dest/'explicit-serializer-analysis.json').write_text(json.dumps({'source_run':run.name,'matched_generic':dict(counts),'differences':misses,'withheld':withheld},indent=2),encoding='utf-8')
    print(dict(counts),'misses',len(misses),'withheld',len(withheld))
if __name__=='__main__':main()
