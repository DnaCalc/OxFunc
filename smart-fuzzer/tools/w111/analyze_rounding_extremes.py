"""Summarize frozen rounding candidate failures without changing the candidate."""
import json,math,struct
from collections import Counter
from pathlib import Path

def decode(text):return struct.unpack('>d',bytes.fromhex(text[2:]))[0]
def main():
    source=Path('.tmp/w111-rounding-frozen-heldout-judgement.json')
    reports=json.loads(source.read_text())
    for report in reports:
        print(report['function_id'],report['rows_judged'],report['rows_agree'])
        groups={}
        for row in report['misses']:
            x,d=map(decode,row['args']);exp=math.floor(math.log10(abs(x)));mode='negative' if d<-1e6 else 'positive' if d>1e6 else 'ordinary'
            groups.setdefault(mode,[]).append((exp,x,d,row['expected'],row['actual']))
        for mode,rows in groups.items():
            print(mode,len(rows),'exponents',min(r[0] for r in rows),max(r[0] for r in rows))
            print(sorted(rows,key=lambda r:abs(r[0]))[:8])
    out=Path('docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries');out.mkdir(parents=True,exist_ok=True)
    (out/'frozen-independent-judgement.json').write_bytes(source.read_bytes())
if __name__=='__main__':main()
