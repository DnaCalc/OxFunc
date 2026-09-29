"""Classify retained distribution discrepancies without hiding numeric failures."""
import collections,json,re,sys
from pathlib import Path

path=Path(sys.argv[1]);report=json.loads(path.read_text())
counts=collections.Counter();rows=[]
for row in report['misses']:
    expected=row['expected'];actual=row['actual']
    if row['excel_execution']!='ok' or row['local_execution']!='ok':kind='harness_limited'
    elif (isinstance(expected,str) and isinstance(actual,str)
          and re.sub(r'number:0x[0-9a-f]{16}','number:bits',expected)
             == re.sub(r'number:0x[0-9a-f]{16}','number:bits',actual)):
        kind='numeric_bits_open'
    else:kind='type_shape_or_error_open'
    counts[kind]+=1;rows.append(dict(classification=kind,**row))
out=dict(source=path.as_posix(),rows=report['rows'],matches=report['matches'],
         discrepancy_counts=dict(counts),misses=rows)
Path(sys.argv[2]).write_text(json.dumps(out,indent=2))
print(json.dumps({k:v for k,v in out.items() if k!='misses'}))
