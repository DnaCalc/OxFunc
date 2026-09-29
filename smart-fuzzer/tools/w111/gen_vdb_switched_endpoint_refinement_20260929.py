"""Freeze a research graph and select bounded endpoint/phase discriminators."""
import hashlib
import json
import math
import struct
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parents[3]
cache = root / 'smart-fuzzer/cache/w111-depreciation-20260929'
source = root / 'smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/depreciation_graph_explorer.rs'
mode = 12884904146
encode = lambda x: '0x'+struct.pack('>d', float(x)).hex()
decode = lambda x: struct.unpack('>d', bytes.fromhex(x[2:]))[0]
known = set()
for filename in ['vdb-switched-bounded-discovery.json','refinement3-vdb-switched-answers.json',
                 'refinement4-vdb-switched-answers.json','heldout-vdb-switched-research-answers.json']:
    for row in json.loads((cache/filename).read_text(encoding='utf-8-sig'))['witnesses']:
        known.add(tuple(row['args']))
residual_path = cache/'vdb-unswitched-clipping-heldout-replay.json'
seeds = json.loads(residual_path.read_text())['misses']
probes = []
def emit(args, origin, variant):
    cost, salvage, life, start, end, factor, no_switch = args
    assert 0 <= start <= end <= life < 1000
    bits = tuple(map(encode,args))
    if bits in known: return
    known.add(bits)
    probes.append({'probe':{'id':f'w111vdb-endpoint-{len(probes):04d}','args':list(bits)},
                   'probe_region':variant,'selection_source':origin})
for seed in seeds:
    args = seed['args']; start,end = args[3:5]
    for direction in [-math.inf,math.inf]:
        changed=args.copy(); changed[3]=math.nextafter(start,direction)
        if 0<=changed[3]<=end: emit(changed,seed['id'],'adjacent-start')
        changed=args.copy(); changed[4]=math.nextafter(end,direction)
        if start<=changed[4]<=args[2]:emit(changed,seed['id'],'adjacent-end')
    for duration in [.125,.75,1.,1.25,2.,2.5]:
        changed=args.copy();changed[4]=start+duration
        if changed[4]<=args[2]:emit(changed,seed['id'],'short-interval-from-original-start')
    for endpoint in [math.floor(end),math.ceil(end),math.floor(end)+.25,math.floor(end)+.75]:
        if start<=endpoint<=args[2]:
            changed=args.copy();changed[4]=endpoint;emit(changed,seed['id'],'integer-quarter-end')
    for shifted_start in [math.floor(start), math.floor(start)+.25, math.floor(start)+.75]:
        if shifted_start<=end:
            changed=args.copy();changed[3]=shifted_start;emit(changed,seed['id'],'integer-quarter-start')
for life in [19.5,31.25,65.75]:
    for factor in [.75,1.5,3.25]:
        for fraction in [.125,.625]:
            start=math.floor(life*.6)+fraction
            emit([1375.,83.,life,start,min(life,start+3.375),factor,0.],
                 'independent-control','phase-control')
freeze={'frozen_utc':datetime.now(timezone.utc).isoformat(),'mode':mode,
        'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
        'source_utf8':source.read_text(encoding='utf-8'),
        'scope':'Research only; inherited straight-line phase uses original interval subtraction. Known numeric residuals persist. Production unchanged.',
        'prior_discovery':{'independent5000_reclassified_discovery':4990,'bounded4996':4946,'short770':770,'transition2256':1983}}
(cache/'switched-research-endpoint-freeze.json').write_text(json.dumps(freeze,indent=2)+'\n')
path=cache/'refinement5-vdb-switched.json'
path.write_text(json.dumps({'function':'VDB','probes':probes},indent=2)+'\n')
manifest={'rows':len(probes),'mode':mode,'source_sha256':freeze['source_sha256'],
          'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
          'batch_sha256':hashlib.sha256(path.read_bytes()).hexdigest(),
          'seed_report_sha256':hashlib.sha256(residual_path.read_bytes()).hexdigest(),
          'qualification':'Selected endpoint/phase discovery, not independent heldout. Previously observed tuples excluded; all period endpoints below1000.'}
(cache/'refinement5-vdb-switched-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps({'path':str(path),'rows':len(probes),'mode':mode}))
