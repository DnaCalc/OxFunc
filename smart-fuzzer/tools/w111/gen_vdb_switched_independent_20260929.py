"""Independent bounded switched-VDB research cohort; production is unchanged."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=root/'smart-fuzzer/cache/w111-depreciation-20260929'
source=root/'smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/depreciation_graph_explorer.rs'
freeze=out/'switched-research-independent-freeze.json'
if freeze.exists():raise SystemExit('Refusing to overwrite switched research freeze')
freeze.write_text(json.dumps(dict(frozen_utc=datetime.now(timezone.utc).isoformat(),mode=2258,source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),source_utf8=source.read_text(encoding='utf-8'),scope='Research only. Coarse shifted-cost graph selected on bounded discovery; known50+273 small-bit differences remain. Production switched kernel unchanged.'),indent=2)+'\n',encoding='utf-8')
bits=lambda n:'0x'+struct.pack('>d',float(n)).hex();rng=random.Random(202609292031);prior=set()
for name in ['vdb-switched-bounded-discovery.json','refinement3-vdb-switched-answers.json','refinement4-vdb-switched-answers.json']:
 prior.update(tuple(w['args']) for w in json.loads((out/name).read_text(encoding='utf-8-sig'))['witnesses'])
rows={}
for _ in range(5000):
 c=10**rng.uniform(-10,10);s=c*rng.uniform(0,1.2);life=rng.uniform(.05,100.);start=rng.uniform(0,life);end=rng.uniform(start,life);factor=rng.choice([0.,rng.uniform(.001,6)])
 a=list(map(bits,[c,s,life,start,end,factor,0.]))
 if tuple(a) not in prior:rows[tuple(a)]='independent-bounded-switched-ordinary-scale'
path=out/'heldout-vdb-switched-research.json';path.write_text(json.dumps(dict(function='VDB',probes=[dict(probe=dict(id=f'w111vdb-switch-independent-{i:04d}',args=list(a)),probe_region=t) for i,(a,t) in enumerate(rows.items())]),indent=2)+'\n',encoding='utf-8')
(out/'heldout-vdb-switched-research-manifest.json').write_text(json.dumps(dict(rows=len(rows),seed=202609292031,excluded_prior_tuples=len(prior),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),qualification='Independent research validation, not production closure; finite normal operands, life/start/end below100. Source mode remains known partial.'),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),rows=len(rows))))
