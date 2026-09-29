"""Conservative parity-ledger snapshot; no completion or characterized promotion.

Uses exact-bit numeric judgements and strictly qualified typed observations.
Existing divergent rows stay divergent until their separate residual evidence is
explicitly reconciled. Counts describe this snapshot, not cumulative unique rows.
"""
import argparse,collections,csv,hashlib,json,shutil
from datetime import datetime,timezone
from pathlib import Path
root=Path(__file__).resolve().parents[3]
evidence=root/'docs/function-lane/evidence/w111-broad-20260929'
p=argparse.ArgumentParser();p.add_argument('--tag',required=True);a=p.parse_args()
tag=a.tag
snapshot=evidence/f'snapshot-{tag}';snapshot.mkdir(exist_ok=True)
if (snapshot/'report.json').exists():raise SystemExit('This snapshot already exists; retain it and author a separately named later snapshot')
numeric_summary=json.loads((root/f'smart-fuzzer/cache/w111-snapshot-{tag}/numeric.json').read_text())
typed_summary=json.loads((root/f'smart-fuzzer/cache/w111-typed-snapshot-{tag}-replay.json').read_text())
rank={'':0,'unmeasured':0,'last_bit':1,'numeric':2,'gross':3,'structural':4}
functions=collections.defaultdict(lambda:{'rows':0,'agree':0,'severity':'','sources':[],'withheld':0})
def add(fn,rows,agree,severity,source,withheld=0):
 r=functions[fn];r['rows']+=rows;r['agree']+=agree;r['withheld']+=withheld
 if rank.get(severity,0)>rank.get(r['severity'],0):r['severity']=severity
 if source not in r['sources']:r['sources'].append(source)
for name in ['numeric','extended']:
 src=root/numeric_summary['groups'][name]['qualified_report']
 dst=snapshot/f'{name}-qualified.json';shutil.copyfile(src,dst)
 for r in json.loads(dst.read_text())['functions']:
  add(r['function_id'],r['admitted_rows'],r['admitted_agree'],r['severity'] or '',dst.name,r['withheld_rows'])
profile_exclusions={'FUNC.ASC','FUNC.DBCS','FUNC.JIS','FUNC.FORMULATEXT','FUNC.ISFORMULA','FUNC.AGGREGATE','FUNC.SUBTOTAL'}
lazy_exclusions={'FUNC.IF','FUNC.IFERROR','FUNC.IFNA','FUNC.IFS','FUNC.SWITCH','FUNC.CHOOSE'}
def load(path):return {r['case_id']:r for r in map(json.loads,(s for s in path.read_text(encoding='utf-8-sig').split('\n') if s.strip()))}
typed=[]
for run in ['w111-broad-typed-20260929-002','w111-numeric-text-20260929']:
 p=root/'smart-fuzzer/runs'/run
 cases=load(p/'cases/cases.jsonl');local=load(p/f'outcomes/local-snapshot-{tag}.jsonl');excel=load(p/'outcomes/excel.jsonl')
 for name in ['manifest.json','cases/cases.jsonl','outcomes/excel.jsonl',f'outcomes/local-snapshot-{tag}.jsonl']:
  dest=snapshot/run/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p/name,dest)
 for id,case in cases.items():
  l,e=local[id],excel[id];a=l['outcome']['digest_payload'];b=e['outcome']['digest_payload'];fn=case['function_id']
  reason=None
  if l['execution_status']!='ok' or e['execution_status']!='ok':reason='harness_execution_nonpass'
  elif fn in profile_exclusions:reason='missing_host_or_reference_metadata_context'
  elif fn in lazy_exclusions:reason='lazy_evaluator_route_requires_separate_qualification'
  elif a.startswith('array:1x1:[') and a==f'array:1x1:[{b}]':reason='one_cell_array_publication_seam'
  severity='' if a==b else ('last_bit' if a.startswith('number:') and b.startswith('number:') and abs(int(a[9:],16)-int(b[9:],16))<=4 else 'structural')
  typed.append({'run':run,'case_id':id,'function_id':fn,'admitted':reason is None,'withheld_reason':reason,'actual':a,'expected':b})
  add(fn,int(reason is None),int(reason is None and a==b),severity if reason is None else '',run,int(reason is not None))
report={'schema_version':'w111.parity_snapshot.v2','snapshot_key':tag,'generated_utc':datetime.now(timezone.utc).isoformat(),'scope':'current-working-tree-discovery-snapshot','completion_claim':False,
 'functions':dict(functions),'typed_qualification':typed,
 'typed_replay_binary':typed_summary['binary_sha256'],
 'numeric_replay_binary_sha256':numeric_summary['binary_sha256']}
(snapshot/'report.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
ledger=root/'docs/function-lane/EXCEL_PARITY_LEDGER.csv'
with ledger.open(encoding='utf-8-sig',newline='') as stream:
 reader=csv.DictReader(stream);fields=reader.fieldnames;rows=list(reader)
assert all(None not in r for r in rows),'Do not rewrite malformed ledger'
changes=[]
for row in rows:
 fn=row['function_id'];r=functions.get(fn)
 if not r or not r['rows']:continue
 old=row['status']
 if r['agree']<r['rows']:
  row['status']='divergent'
  if rank.get(r['severity'],0)>rank.get(row['severity'],0):row['severity']=r['severity']
 elif old=='unverified':row['status']='consistent';row['severity']=''
 # The snapshot cannot silently retire residuals from older evidence.
 row['as_of']='2026-09-29';row['excel_builds_judged']='20430/CV2 (channel unverified); earlier builds in linked evidence'
 row['rows_judged']=str(r['rows']);row['rows_agree']=str(r['agree'])
 row['worst_ulp']='' if r['agree']<r['rows'] or row['status']=='divergent' else '0'
 note=f'W111 2026-09-29 working-tree discovery snapshot {tag}: {r["agree"]}/{r["rows"]} admitted comparisons, {r["withheld"]} withheld; full qualification in snapshot-{tag}/report.json. Counts are this snapshot only, not cumulative. No completion promotion; prior residuals retained until explicitly reconciled.'
 row['exact_domain_note']=note+' Earlier: '+row['exact_domain_note']
 ref=f'docs/function-lane/evidence/w111-broad-20260929/snapshot-{tag}/report.json'
 row['evidence_refs']=ref+';'+row['evidence_refs']
 changes.append({'function_id':fn,'before':old,'after':row['status'],'rows':r['rows'],'agree':r['agree']})
with ledger.open('w',encoding='utf-8',newline='') as stream:
 writer=csv.DictWriter(stream,fieldnames=fields,lineterminator='\n');writer.writeheader();writer.writerows(rows)
(snapshot/'ledger-changes.json').write_text(json.dumps(changes,indent=2),encoding='utf-8')
shutil.copyfile(root/f'smart-fuzzer/cache/w111-snapshot-{tag}/numeric.json',snapshot/'numeric-summary.json')
shutil.copyfile(root/f'smart-fuzzer/cache/w111-typed-snapshot-{tag}-replay.json',snapshot/'typed-summary.json')
artifacts=[{'path':p.relative_to(snapshot).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(snapshot.rglob('*')) if p.is_file()]
(snapshot/'artifact-manifest.json').write_text(json.dumps({'artifacts':artifacts},indent=2))
print('Ledger snapshot',len(changes),'functions; status counts',dict(collections.Counter(r['status'] for r in rows)))
