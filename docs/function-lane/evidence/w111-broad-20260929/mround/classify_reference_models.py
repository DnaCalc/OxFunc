"""Offline baseline/rejection/intersection predictions; no oracle authority."""
import copy, hashlib, json, re, subprocess, sys
from pathlib import Path
source,out=map(Path,sys.argv[1:]);out.mkdir(parents=True,exist_ok=True)
cases=json.loads(source.read_text())['cases']
binary=Path('smart-fuzzer/tools/pmt_ppmt_local_eval/target/debug/array_tranche_local_eval.exe')
def point(text):
    m=re.fullmatch(r'([A-Z]+)([0-9]+)',text);assert m,text
    col=0
    for c in m[1]:col=col*26+ord(c)-64
    return int(m[2]),col
models={}
for mode in ['baseline','reference_rejection','caller_intersection']:
    rows=copy.deepcopy(cases)
    for c in rows:
        if mode=='baseline':continue
        fixtures={f['target']:f['value'] for f in c['cell_fixture']}
        for i,a in enumerate(c['args']):
            if a['kind']!='reference':continue
            data=fixtures[a['target']]
            if data['kind']!='array' or sum(map(len,data['rows']))==1:continue
            replacement={'kind':'error','code':'Value'}
            if mode=='caller_intersection':
                first,last=a['target'].split(':');r0,c0=point(first);r1,c1=point(last)
                row,col=point(c['formula_cell'])
                if c0==c1 and r0<=row<=r1:replacement=data['rows'][row-r0][0]
                elif r0==r1 and c0<=col<=c1:replacement=data['rows'][0][col-c0]
            c['args'][i]=replacement
    path=out/(mode+'-model-cases.jsonl');answer=out/(mode+'-model-outcomes.jsonl')
    path.write_text('\n'.join(json.dumps(c) for c in rows)+'\n')
    subprocess.run([str(binary),'--cases',str(path),'--out',str(answer)],check=True)
    models[mode]={r['case_id']:r['outcome']['digest_payload'] for r in map(json.loads,answer.read_text().splitlines())}
comparisons=[dict(case_id=c['case_id'],formula=c['formula_text'],formula_cell=c['formula_cell'],
    predictions={m:rows[c['case_id']] for m,rows in models.items()}) for c in cases]
report=dict(authority='offline diagnostic predictions only; no Excel outcomes used',
    local_binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),rows=len(cases),
    baseline_vs_rejection_differences=sum(r['predictions']['baseline']!=r['predictions']['reference_rejection'] for r in comparisons),
    rejection_vs_intersection_differences=sum(r['predictions']['reference_rejection']!=r['predictions']['caller_intersection'] for r in comparisons),
    cases=comparisons)
(out/'model-predictions.json').write_text(json.dumps(report,indent=2))
print(json.dumps({k:v for k,v in report.items() if k!='cases'}))
