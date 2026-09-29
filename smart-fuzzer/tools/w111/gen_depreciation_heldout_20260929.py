"""Freeze and generate independent DB/DDB probes; never calls Excel."""
import hashlib
import json
import math
import random
import struct
from datetime import datetime, timezone
from pathlib import Path
import gen_broad_typed_20260929 as g

out=g.ROOT/"smart-fuzzer/cache/w111-depreciation-20260929"
freeze=out/"freeze-db-ddb-v1.json"
if freeze.exists(): raise SystemExit("Refusing to overwrite frozen candidate")
bits=lambda n:"0x%016x"%struct.unpack("<Q",struct.pack("<d",n))[0]
sources=[g.ROOT/"crates/oxfunc_core/src/functions/depreciation_family.rs",g.ROOT/"crates/oxfunc_core/src/functions/power_fn.rs",Path(__file__)]
freeze.write_text(json.dumps(dict(frozen_utc=datetime.now(timezone.utc).isoformat(),
    sources={str(p.relative_to(g.ROOT)).replace('\\','/'):hashlib.sha256(p.read_bytes()).hexdigest() for p in sources},
    model="DB truncated month/period, fractional life, book recurrence, explicit first/final operation graph; DDB POWER-wrapper closed form and x87 factor/life; final zero/subnormal publication"),indent=2)+"\n",encoding="utf-8")
rng=random.Random(202609290717)
for fn in ("DB","DDB"):
    prior=set()
    for path in [g.ROOT/f"smart-fuzzer/runs/w111-extended-numeric-20260929/answers/answers-{fn.lower()}.json",out/f"answers-{fn.lower()}.json"]:
        prior.update(tuple(r['args']) for r in json.loads(path.read_text(encoding='utf-8-sig'))['witnesses'])
    rows={}
    def add(args,axis):
        if not all(math.isfinite(n) and (n==0 or abs(n)>=2**-1022) and bits(n)!='0x8000000000000000' for n in args): return
        key=tuple(map(bits,args))
        if key not in prior: rows.setdefault(key,axis)
    for i in range(3500):
        cost=10**rng.uniform(-200,200)
        salvage=cost*rng.uniform(0,2)
        life=rng.uniform(.01,250)
        period=rng.choice([rng.uniform(.001,life+2),float(rng.randrange(1,math.ceil(life)+2)),life,math.nextafter(life,math.inf)])
        extra=rng.uniform(.01,14) if fn=="DB" else 10**rng.uniform(-3,3)
        add([cost,salvage,life,period,extra],"independent-log-cost-fractional-life-period-and-optional")
    for i in range(1000):
        cost=10**rng.uniform(-307,307);salvage=10**rng.uniform(-307,307)
        life=rng.uniform(.01,300); period=rng.uniform(.001,life)
        add([cost,salvage,life,period,rng.uniform(1,12.99) if fn=="DB" else 10**rng.uniform(-250,250)],"independent-extreme-ratio-and-publication")
    for i in range(750):
        cost=rng.uniform(.1,1e9);life=rng.uniform(1,40)
        if fn=="DB":
            rate=(rng.randrange(0,950)+.5)/1000
            salvage=cost*math.pow(1-rate,life)
            salvage=math.nextafter(salvage,rng.choice([0.,math.inf]))
            add([cost,salvage,life,rng.uniform(.01,life+1),float(rng.randrange(1,13))],"independent-rounded-rate-half-threshold")
        else:
            rate=math.nextafter(1.,rng.choice([0.,math.inf]))
            add([cost,cost*rng.random(),life,rng.choice([.5,1.,math.nextafter(1.,math.inf),life]),life*rate],"independent-rate-one-branch")
    doc=dict(function=fn,probes=[dict(probe=dict(id=f"w111dep-heldout-{fn}-{i:05d}",args=list(k)),probe_region=tag) for i,(k,tag) in enumerate(rows.items())])
    path=out/f"heldout-{fn.lower()}.json";path.write_text(json.dumps(doc,indent=2)+"\n",encoding="utf-8")
    (out/f"heldout-{fn.lower()}-manifest.json").write_text(json.dumps(dict(rows=len(rows),excluded_prior_tuples=len(prior),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),ingress="normal binary64 and positive zero only; source tuples excluded from discovery"),indent=2)+"\n",encoding="utf-8")
    print(json.dumps(dict(path=str(path),rows=len(rows))))

g.TRANCHE="w111-depreciation-typed-options-20260929"
for fn,args in {"DB":[g.n(1000),g.n(100),g.n(10),g.n(2),g.n(12)],"DDB":[g.n(1000),g.n(100),g.n(10),g.n(2),g.n(2)],"VDB":[g.n(1000),g.n(100),g.n(10),g.n(1),g.n(2),g.n(2),g.b(False)]}.items():
    for pos in range(4 if fn!="VDB" else 5,len(args)):
        for val in [g.missing(),g.blank(),g.t(""),g.t("2"),g.t("TRUE"),g.t("FALSE"),g.b(True),g.n(0),g.n(-1),g.e("NA"),g.a([[g.n(1),g.n(2)]])]:
            changed=args.copy();changed[pos]=val;g.emit(fn,f"optional-{pos}-{len(g.cases)}",changed)
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111dep-options-')
doc=dict(schema_version="oxfunc.smart_fuzzer.scenario_seed_case_set.v0",tranche_id=g.TRANCHE,cases=g.cases,tranches=[dict(tranche_id=g.TRANCHE,case_ids=[c['case_id'] for c in g.cases])])
path=out/"typed-options.json";path.write_text(json.dumps(doc,indent=2)+"\n",encoding="utf-8");path.with_suffix('.jsonl').write_text(''.join(json.dumps(c)+'\n' for c in g.cases),encoding='utf-8')
print(json.dumps(dict(path=str(path),rows=len(g.cases))))
