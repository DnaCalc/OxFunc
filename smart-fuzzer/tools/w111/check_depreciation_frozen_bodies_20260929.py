"""Check selected function bodies against snapshots in the shared source file."""
import hashlib,json,re
from pathlib import Path
root=Path(__file__).resolve().parents[3];cache=root/'smart-fuzzer/cache/w111-depreciation-20260929'
source=root/'crates/oxfunc_core/src/functions/depreciation_family.rs'
def body(text,name):
 match=re.search(r'(?m)^(?:pub )?fn '+re.escape(name)+r'\b',text)
 assert match,name
 start=text.index('{',match.start());depth=1;end=start+1
 while depth:
  depth+=(text[end]=='{')-(text[end]=='}');end+=1
 return text[match.start():end]
current=source.read_text(encoding='utf-8')
groups={
 'DB':('candidate-db-v8.rs',['db_kernel','db_rate','depreciation_integer_power','publish_depreciation','validate_finite']),
 'DDB':('candidate-ddb-v6.rs',['ddb_kernel','ddb_period_depreciation','depreciation_integer_power','publish_depreciation','validate_finite']),
 'VDB_no_switch':('candidate-vdb-noswitch-v5.rs',['declining_interval_depreciation','ddb_period_depreciation','depreciation_integer_power','publish_depreciation','validate_finite']),
 'SYD':('candidate-syd-v3.rs',['syd_kernel','publish_depreciation','validate_finite']),
}
rows=[]
for family,(snapshot,names) in groups.items():
 original=(cache/snapshot).read_text(encoding='utf-8')
 for name in names:
  a=body(original,name);b=body(current,name)
  rows.append(dict(family=family,snapshot=snapshot,function=name,equal=a==b,body_sha256=hashlib.sha256(b.encode()).hexdigest()))
report=dict(source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),comparison='Exact selected Rust function text, after universal-newline decoding. Whole-file snapshots remain retained; this does not compare external dependencies or the complete binary.',rows=rows)
path=cache/'frozen-function-body-check.json';path.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(function_checks=len(rows),equal=sum(r['equal'] for r in rows),path=str(path))))
assert all(r['equal'] for r in rows)
