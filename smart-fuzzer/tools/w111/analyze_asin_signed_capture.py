"""Describe exact paired-sign Excel observations without fitted corrections."""
import collections,json,sys
from pathlib import Path

capture=json.loads(Path(sys.argv[1]).read_text(encoding='utf-8-sig'))
by_bits={int(r['args'][0],16):r for r in capture['witnesses']}
assert len(by_bits)==len(capture['witnesses'])
counts=collections.defaultdict(lambda:dict(pairs=0,non_odd=0));failures=[]
for raw,pos in by_bits.items():
    if raw>>63:continue
    neg=by_bits.get(raw|(1<<63))
    assert neg is not None
    category=pos['id'].split('-',4)[-1]
    expected_positive=int(pos['expected_bits'],16)
    expected_negative=int(neg['expected_bits'],16)
    delta=(expected_negative&((1<<63)-1))-expected_positive
    # Negative zero is independently normalized by the workbook input seam.
    odd=delta==0
    counts[category]['pairs']+=1;counts[category]['non_odd']+=not odd
    if not odd:
        failures.append(dict(positive_input=pos['args'][0],positive=pos['expected_bits'],
            negative=neg['expected_bits'],magnitude_ulp_delta=delta,category=category))
out=dict(rows=len(by_bits),pairs=len(by_bits)//2,non_odd_pairs=len(failures),by_category=dict(counts),non_odd_observations=failures)
Path(sys.argv[2]).write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({k:v for k,v in out.items() if k!='non_odd_observations'},indent=2))
