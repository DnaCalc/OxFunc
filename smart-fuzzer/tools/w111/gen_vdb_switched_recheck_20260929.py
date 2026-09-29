"""Exact typed readback replication and neighbors of ten retained VDB misses."""
import copy
import hashlib
import json
import math
from datetime import datetime,timezone
from pathlib import Path
import gen_broad_typed_20260929 as g

cache=g.ROOT/'smart-fuzzer/cache/w111-depreciation-20260929'
g.TRANCHE='w111-vdb-switched-recheck-20260929'
source=g.ROOT/'smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/depreciation_graph_explorer.rs'
residual=cache/'vdb-absolute-period-end-vdb-switched-bounded-discovery.json'
seeds=json.loads(residual.read_text())['misses']
for seed in seeds:
    base=seed['args'][:5]
    variants=[('default',base),('explicit-factor',base+[2.]),('explicit-false',base+[2.,False]),
              ('true-control',base+[2.,True]),('factor-down',base+[math.nextafter(2.,-math.inf)]),
              ('factor-up',base+[math.nextafter(2.,math.inf)])]
    for scale in [.5,2.]:variants.append((f'scale-{scale}',[base[0]*scale,base[1]*scale,*base[2:],2.]))
    for pos,tag in [(3,'start-up'),(4,'end-up')]:
        args=base+[2.];args[pos]=math.nextafter(args[pos],math.inf);variants.append((tag,args))
    for tag,values in variants:
        assert 0<=values[3]<values[4]<=values[2]<1000
        args=[g.b(v) if isinstance(v,bool) else g.n(v) for v in values]
        g.emit('VDB',seed['id']+'-'+tag,args,axis='switched_residual_replication')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111vdb-recheck-')
path=cache/'typed-vdb-switched-recheck.json'
path.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',tranche_id=g.TRANCHE,
    cases=g.cases,tranches=[dict(tranche_id=g.TRANCHE,case_ids=[c['case_id'] for c in g.cases])]),indent=2)+'\n')
freeze=dict(frozen_utc=datetime.now(timezone.utc).isoformat(),mode=73014446290,
    source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),source_utf8=source.read_text(),
    scope='Research only: requested-period partial amount uses end-(start+year), with separate stored absolute left endpoint. Prior known10+160 residuals remain. No production change.')
(cache/'switched-research-absolute-end-freeze.json').write_text(json.dumps(freeze,indent=2)+'\n')
(cache/'typed-vdb-switched-recheck-manifest.json').write_text(json.dumps(dict(cases=len(g.cases),
    batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),source_residual_sha256=hashlib.sha256(residual.read_bytes()).hexdigest(),
    generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),qualification='Typed live fixture readback distinguishes retained-default-factor observations from fresh reproduction, explicit factor, adjacent factor and scale controls. No known mismatch waived as a harness issue.'),indent=2)+'\n')
print(json.dumps(dict(path=str(path),cases=len(g.cases),tranche=g.TRANCHE)))
