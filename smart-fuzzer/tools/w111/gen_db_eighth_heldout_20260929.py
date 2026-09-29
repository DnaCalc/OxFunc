"""Fresh eighth DB heldout after internal amount publication discrimination."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/"smart-fuzzer/cache/w111-depreciation-20260929"
source=g.ROOT/"crates/oxfunc_core/src/functions/depreciation_family.rs"
snapshot=out/"candidate-db-v8.rs"
if snapshot.exists():raise SystemExit("Refusing to overwrite candidate v8")
snapshot.write_bytes(source.read_bytes())
(out/"freeze-db-v8.json").write_text(json.dumps(dict(frozen_utc=datetime.now(timezone.utc).isoformat(),source=str(source.relative_to(g.ROOT)),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),snapshot=snapshot.name,changes_since_v7="DB final-year quotient and rate product publish subnormals as positive zero before later multiplication. DDB base subtraction also refined in shared source. All 93616 DB discovery rows match before freeze."),indent=2)+"\n",encoding="utf-8")
bits=lambda n:'0x'+struct.pack('>d',float(n)).hex()
decode=lambda b:struct.unpack('>d',b.to_bytes(8,'big'))[0]
rng=random.Random(202609292008)
for fn in ["DB"]:
 prior=set()
 paths=[g.ROOT/f"smart-fuzzer/runs/w111-extended-numeric-20260929/answers/answers-{fn.lower()}.json",out/f"answers-{fn.lower()}.json"]+list(out.glob(f"*-{fn.lower()}-answers.json"))
 for p in paths:
  prior.update(tuple(w['args']) for w in json.loads(p.read_text(encoding='utf-8-sig'))['witnesses'])
 rows={}
 def add(a,tag):
  if not all(math.isfinite(x) and (x==0 or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000' for x in a):return
  k=tuple(map(bits,a))
  if k not in prior:rows.setdefault(k,tag)
 for i in range(12000):
  cost=decode((rng.randrange(1,2047)<<52)|rng.getrandbits(52))
  salvage=decode((rng.randrange(1,2047)<<52)|rng.getrandbits(52))
  life=rng.uniform(.001,200)
  period=rng.uniform(.00001,life+1)
  extra=rng.uniform(-.1,13.2) if fn=='DB' else 10**rng.uniform(-307,307)
  add([cost,salvage,life,period,extra],'fresh-full-exponent-cost-salvage')
 for i in range(6000):
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
 for i in range(2500):
  cost=10**rng.uniform(-100,100);ratio=10**rng.uniform(-300,300);salvage=cost*ratio
  life=abs(math.log(ratio))/float.fromhex('0x1.fffffffffffffp+1023')*rng.choice([.98,.999999999,1.,1.000000001,1.02])
  add([cost,salvage,life,rng.choice([.173,1.173]),rng.uniform(1.01,11.99)],'fresh-internal-power-log-overflow-boundary')
 for i in range(5000):
  cost=2**-1022*rng.uniform(1.,2.)*2**rng.randrange(0,12)
  life=rng.uniform(2.,80.)
  rate=-(rng.randrange(1,3000)/1000.)
  salvage=cost*math.pow(1.-rate,life)
  add([cost,salvage,life,rng.uniform(.01,life+1),rng.uniform(1.,12.99)],'fresh-minimum-normal-cost-growing-book-publication')
 for i in range(3000):
  ratio=1.-2.**-rng.randrange(22,54)*rng.uniform(.5,1.5)
  target=(rng.randrange(0,900)+.5)/1000.
  exponent=round(math.log1p(-target)/math.log(ratio))
  exponent=max(1.,float(exponent)+rng.choice([-2.,-1.,0.,1.,2.]))
  add([1.,ratio,1./exponent,rng.uniform(.01,.99),rng.uniform(1.,11.99)],'fresh-large-integer-power-rate-half-neighbors')
 for i in range(6000):
  cost=2**-1022*2**rng.uniform(0.,10.)
  life=rng.uniform(.1,12.)
  rate=-(rng.randrange(1,8000)/1000.)
  salvage=cost*math.pow(1.-rate,life)
  add([cost,salvage,life,math.floor(life)+1.+rng.random()*.999,rng.uniform(1.,11.999)],'fresh-minimum-normal-final-year-stage-publication')
 path=out/f"eighth-heldout-{fn.lower()}.json"
 doc=dict(function=fn,probes=[dict(probe=dict(id=f'w111dep-eighth-{fn}-{i:05d}',args=list(k)),probe_region=tag) for i,(k,tag) in enumerate(rows.items())])
 path.write_text(json.dumps(doc,indent=2)+"\n",encoding="utf-8")
 (out/f'eighth-heldout-{fn.lower()}-manifest.json').write_text(json.dumps(dict(rows=len(rows),excluded_prior_tuples=len(prior),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),input_policy='normal binary64 plus positive zero; all prior tuples excluded; candidate frozen before generation'),indent=2)+"\n",encoding='utf-8')
 print(json.dumps(dict(path=str(path),rows=len(rows))))
