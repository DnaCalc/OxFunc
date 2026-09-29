"""Repeat broad numeric judging with explicit Value2 and unstable-WEEKNUM exclusions."""
import argparse,collections,hashlib,json,math,subprocess
from pathlib import Path
from datetime import datetime,timezone
root=Path(__file__).resolve().parents[3]
p=argparse.ArgumentParser();p.add_argument('--tag',required=True);p.add_argument('--binary',required=True);a=p.parse_args()
binary=(root/a.binary).resolve();out=root/f'smart-fuzzer/cache/w111-snapshot-{a.tag}';out.mkdir(exist_ok=True)
report_path=out/'numeric.json';assert not report_path.exists()
groups={'numeric':[root/'smart-fuzzer/runs/w111-broad-20260929/answers'],
        'extended':[root/'smart-fuzzer/runs/w111-extended-numeric-20260929/answers',root/'smart-fuzzer/runs/w111-extended-nullary-20260929/answers']}
def qualify(fn,row):
 if fn=='ISREF':return 'worksheet_reference_origin_differs_from_local_numeric_value'
 for arg in row['args']:
  bits=int(arg,16);exp=(bits>>52)&2047
  if bits==0x8000000000000000 or (exp==0 and bits&0xfffffffffffff):return 'Value2_changes_signed_zero_or_subnormal_input'
 if fn=='WEEKNUM' and len(row['args'])>1:
  import struct
  x=struct.unpack('>d',int(row['args'][1],16).to_bytes(8,'big'))[0]
  if math.isfinite(x):
   upper=math.ceil(x);eps=2049/8589934592 if upper>0 else 2049/17179869184
   selector=upper if upper-x<=eps else math.floor(x)
   if selector not in {1,2,11,12,13,14,15,16,17,21}:return 'unsupported_WEEKNUM_selector_oracle_not_deterministically_qualified'
 return None
summary={'schema_version':'w111.numeric_snapshot.v2','tag':a.tag,'captured_utc':datetime.now(timezone.utc).isoformat(),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'groups':{}}
for group,dirs in groups.items():
 metadata={};paths=[]
 for directory in dirs:
  for source in sorted(directory.glob('answers-*.json')):
   bank=json.loads(source.read_text(encoding='utf-8-sig'));fn=bank['function'];admitted=[];withheld={}
   for row in bank['witnesses']:
    reason=qualify(fn,row)
    if reason:withheld[row['id']]=reason
    else:admitted.append(row)
   target=out/'inputs'/group/source.name;target.parent.mkdir(parents=True,exist_ok=True)
   target.write_text(json.dumps({**bank,'witnesses':admitted},separators=(',',':')),encoding='utf-8')
   metadata['FUNC.'+fn]={'source':source.relative_to(root).as_posix(),'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'raw_rows':len(bank['witnesses']),'withheld_ids':withheld,'withheld_rows':len(withheld)}
   paths.append(str(target.relative_to(root)))
 run=subprocess.run([str(binary),'judge-witnesses',*paths,'--max-misses','100000','--json'],cwd=root,check=True,capture_output=True,text=True,encoding='utf-8')
 reports=json.loads(run.stdout);functions=[]
 for report in reports:
  meta=metadata[report['function_id']]
  functions.append({**meta,**report,'admitted_rows':report['rows_judged'],'admitted_agree':report['rows_agree']})
 doc={'comparison_policy':'exact_typed_bit_match_no_tolerance','input_limit_evidence':'numeric-ingress.json and dates/weeknum-direct-analysis.json','functions':functions,'totals':{key:sum(r[key] for r in functions) for key in ['raw_rows','admitted_rows','admitted_agree','withheld_rows']}}
 dest=out/f'{group}-qualified.json';dest.write_text(json.dumps(doc,separators=(',',':')),encoding='utf-8')
 summary['groups'][group]={'functions':len(functions),**doc['totals'],'qualified_report':dest.relative_to(root).as_posix(),'sha256':hashlib.sha256(dest.read_bytes()).hexdigest(),'functions_with_misses':sum(r['rows_agree']<r['rows_judged'] for r in functions)}
 print(group,summary['groups'][group])
report_path.write_text(json.dumps(summary,indent=2),encoding='utf-8')
