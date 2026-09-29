"""Judge switched research separately from the ten production no-switch controls."""
import hashlib,json,struct,subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[3]
run=root/'smart-fuzzer/runs/w111-vdb-switched-recheck-20260929'
cache=root/'smart-fuzzer/cache/w111-depreciation-20260929'
assert (run/'rollup.json').exists(),'Capture must finish before judgement'
def rows(path):return {r['case_id']:r for r in map(json.loads,filter(str.strip,path.read_text(encoding='utf-8-sig').split('\n')))}
case_path=run/'cases/cases.jsonl';excel_path=run/'outcomes/excel.jsonl';local_path=run/'outcomes/local.jsonl'
cases,excel,local=map(rows,[case_path,excel_path,local_path]);assert cases.keys()==excel.keys()==local.keys()
witnesses=[];controls=[];harness=[]
encode=lambda v:'0x'+struct.pack('>d',float(v)).hex()
for cid,case in cases.items():
    answer=excel[cid]
    if answer['execution_status']!='ok':harness.append(cid);continue
    args=[a['value'] for a in case['args']]
    expected=answer['outcome']['digest_payload']
    if len(args)==7 and args[6] is True:
        controls.append(dict(case_id=cid,exact=local[cid]['execution_status']=='ok' and local[cid]['outcome']['digest_payload']==expected))
        continue
    assert expected.startswith(('number:0x','error:')),expected
    witnesses.append(dict(id=cid,args=list(map(encode,args)),expected_bits=expected.removeprefix('number:')))
provenance={'sources':[{'path':str(p.relative_to(root)).replace('\\','/'),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in [case_path,excel_path,local_path,run/'manifest.json']],
            'policy':'Exact typed Value2 readback; harness nonpasses withheld; no-switch controls checked against production local outcomes.'}
answer_path=cache/'typed-vdb-switched-recheck-research-answers.json'
answer_path.write_text(json.dumps(dict(function='VDB',witnesses=witnesses,capture_provenance=provenance),indent=2)+'\n')
exe=root/'smart-fuzzer/tools/pmt_ppmt_local_eval/target/debug/depreciation_graph_explorer.exe'
mode=73014446290
r=json.loads(subprocess.run([str(exe),'--vdb-shift-one',str(mode),str(answer_path)],capture_output=True,text=True,check=True).stdout)
r.update(dict(admitted=len(witnesses),harness_nonpasses=harness,no_switch_controls=controls,
              source_sha256=hashlib.sha256((root/'smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/depreciation_graph_explorer.rs').read_bytes()).hexdigest(),
              frozen_model='switched-research-absolute-end-freeze.json',research_only=True))
(cache/'typed-vdb-switched-recheck-research-judgement.json').write_text(json.dumps(r,indent=2)+'\n')
print(json.dumps(dict(switched_exact=r['exact'],switched_admitted=len(witnesses),no_switch_exact=sum(c['exact'] for c in controls),no_switch_controls=len(controls),harness_nonpasses=len(harness))))
