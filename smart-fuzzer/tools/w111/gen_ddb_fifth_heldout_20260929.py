"""Fresh fifth DDB heldout after its annual product refinement."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/"smart-fuzzer/cache/w111-depreciation-20260929"
source=g.ROOT/"crates/oxfunc_core/src/functions/depreciation_family.rs"
snapshot=out/"candidate-ddb-v5.rs"
if snapshot.exists():raise SystemExit("Refusing to overwrite candidate v5")
snapshot.write_bytes(source.read_bytes())
(out/"freeze-ddb-v5.json").write_text(json.dumps(dict(frozen_utc=datetime.now(timezone.utc).isoformat(),source=str(source.relative_to(g.ROOT)),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),snapshot=snapshot.name,changes_since_v4="DDB internal power has a bounded unsigned-32-bit integer dispatch with x87 accumulator stores. All 29235 prior DDB rows match before freezing."),indent=2)+"\n",encoding="utf-8")
bits=lambda n:'0x'+struct.pack('>d',float(n)).hex()
decode=lambda b:struct.unpack('>d',b.to_bytes(8,'big'))[0]
rng=random.Random(202609291853)
for fn in ["DDB"]:
 prior=set()
 paths=[g.ROOT/f"smart-fuzzer/runs/w111-extended-numeric-20260929/answers/answers-{fn.lower()}.json",out/f"answers-{fn.lower()}.json"]+list(out.glob(f"*-{fn.lower()}-answers.json"))
 for p in paths:
  prior.update(tuple(w['args']) for w in json.loads(p.read_text(encoding='utf-8-sig'))['witnesses'])
 rows={}
 def add(a,tag):
  if not all(math.isfinite(x) and (x==0 or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000' for x in a):return
  k=tuple(map(bits,a))
  if k not in prior:rows.setdefault(k,tag)
 for i in range(4000):
  cost=decode((rng.randrange(1,2047)<<52)|rng.getrandbits(52))
  salvage=decode((rng.randrange(1,2047)<<52)|rng.getrandbits(52))
  life=rng.uniform(.001,200)
  period=rng.uniform(.00001,life+1)
  extra=rng.uniform(-.1,13.2) if fn=='DB' else 10**rng.uniform(-307,307)
  add([cost,salvage,life,period,extra],'fresh-full-exponent-cost-salvage')
 for i in range(2000):
  cost=10**rng.uniform(-300,300);life=rng.uniform(.1,100)
  if fn=='DB':
   rate=(rng.randrange(-5000,1000)+.5)/1000
   salvage=cost*math.pow(1-rate,life)
   for _ in range(rng.randrange(1,9)):salvage=math.nextafter(salvage,rng.choice([0.,math.inf]))
   add([cost,salvage,life,rng.uniform(.0001,life+1),rng.uniform(1,12.9999)],'fresh-decimal-rate-half-and-neighbors')
  else:
   factor=life
   for _ in range(rng.randrange(1,9)):factor=math.nextafter(factor,rng.choice([0.,math.inf]))
   add([cost,cost*rng.random(),life,rng.choice([rng.uniform(.0001,1),math.nextafter(1.,math.inf),rng.uniform(1,life) if life>1 else life]),factor],'fresh-rate-one-cancellation')
 for i in range(1500):
  cost=rng.choice([0.,1.,1e300,1e-300])*rng.uniform(.9,1.1)
  salvage=rng.choice([0.,1.,1e300,1e-300])*rng.uniform(.9,1.1)
  life=10**rng.uniform(-307,307)
  period=rng.choice([.125,.75,1.,2.])
  extra=rng.uniform(1,12.999) if fn=='DB' else 10**rng.uniform(-307,307)
  add([cost,salvage,life,period,extra],'fresh-extreme-life-and-stages')
 if fn=='DB':
  for boundary in [2**31,2**32-1,2**32,2**53]:
   for delta in [-128,-2,-1,-.5,0,.5,1,2,128]:
    for month in [1.01,6.999,12.25]:
     for period in [.33,1.11,2.33,1e30]:
      add([1234.5,0.,float(boundary+delta),period,month],'fresh-host-count-boundaries')
  for exponent in [2**31-1,2**31,2**31+1,2**32-2,2**32-1,2**32,2**32+1,2**32+2]:
   for ratio in [1.00001,1.25,2.,1e250]:
    for month in [1.999,7.123,11.999]:
     add([1.,ratio,1/exponent,.234,month],'fresh-power-integer-overflow-branch-boundary')
 else:
  for i in range(1000):
   life=10**rng.uniform(-2,2)
   factor=2**-1022*life
   for _ in range(rng.randrange(1,8)):factor=math.nextafter(factor,rng.choice([0.,math.inf]))
   add([10**rng.uniform(250,307),rng.choice([0.,1e-200]),life,rng.uniform(.01,life),factor],'fresh-internal-normal-subnormal-rate-boundary')
 for i in range(5000):
  life=2.**rng.uniform(20,61)
  period=rng.choice([life*rng.random(),math.nextafter(life,0.),life,float(rng.randrange(1,2**32+2048))])
  period=min(period,life)
  cost=10**rng.uniform(-250,250)
  add([cost,cost*rng.choice([0.,rng.random()]),life,period,rng.choice([.125,.5,1.,2.,10.,rng.uniform(.001,20.)])],'fresh-large-integer-fractional-period-power')
 for boundary in [2**31,2**32-1,2**32,2**53]:
  for delta in [-257,-3,-1,-.5,0.,.5,1,3,257]:
   period=float(boundary+delta)
   for factor in [.333,1.875,7.25]:
    add([123.456,0.,period*1.12345,period,factor],'fresh-internal-power-dispatch-boundary')
 path=out/f"fifth-heldout-{fn.lower()}.json"
 doc=dict(function=fn,probes=[dict(probe=dict(id=f'w111dep-fifth-{fn}-{i:05d}',args=list(k)),probe_region=tag) for i,(k,tag) in enumerate(rows.items())])
 path.write_text(json.dumps(doc,indent=2)+"\n",encoding="utf-8")
 (out/f'fifth-heldout-{fn.lower()}-manifest.json').write_text(json.dumps(dict(rows=len(rows),excluded_prior_tuples=len(prior),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),input_policy='normal binary64 plus positive zero; all prior tuples excluded; candidate frozen before generation'),indent=2)+"\n",encoding='utf-8')
 print(json.dumps(dict(path=str(path),rows=len(rows))))
