"""Independent second DB/DDB heldout, after the first exposed model gaps."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/"smart-fuzzer/cache/w111-depreciation-20260929"
source=g.ROOT/"crates/oxfunc_core/src/functions/depreciation_family.rs"
snapshot=out/"candidate-db-ddb-v2.rs"
if snapshot.exists():raise SystemExit("Refusing to overwrite candidate v2")
snapshot.write_bytes(source.read_bytes())
(out/"freeze-db-ddb-v2.json").write_text(json.dumps(dict(frozen_utc=datetime.now(timezone.utc).isoformat(),source=str(source.relative_to(g.ROOT)),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),snapshot=snapshot.name,changes_since_v1="DB normalizes internal ratio/power stages, uses decimal ROUND and x87 book products; DDB checks declining overflow before clipping and uses x87 book product; typed absence differs from explicit missing; VDB admission and logical flag preparation"),indent=2)+"\n",encoding="utf-8")
bits=lambda n:'0x'+struct.pack('>d',float(n)).hex()
decode=lambda b:struct.unpack('>d',b.to_bytes(8,'big'))[0]
rng=random.Random(202609290912)
for fn in ["DB","DDB"]:
 prior=set()
 for p in [g.ROOT/f"smart-fuzzer/runs/w111-extended-numeric-20260929/answers/answers-{fn.lower()}.json",out/f"answers-{fn.lower()}.json",out/f"heldout-{fn.lower()}-answers.json",out/f"refinement-{fn.lower()}-answers.json"]:
  prior.update(tuple(w['args']) for w in json.loads(p.read_text(encoding='utf-8-sig'))['witnesses'])
 rows={}
 def add(a,tag):
  if not all(math.isfinite(x) and (x==0 or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000' for x in a):return
  k=tuple(map(bits,a))
  if k not in prior:rows.setdefault(k,tag)
 for i in range(3000):
  cost=decode((rng.randrange(1,2047)<<52)|rng.getrandbits(52))
  salvage=decode((rng.randrange(1,2047)<<52)|rng.getrandbits(52))
  life=rng.uniform(.01,300)
  period=rng.uniform(.001,life) if i%2 else float(rng.randrange(1,math.ceil(life)+2))
  extra=rng.uniform(0,13) if fn=='DB' else 10**rng.uniform(-307,307)
  add([cost,salvage,life,period,extra],'independent-full-binary64-cost-and-salvage')
 for i in range(1800):
  cost=rng.uniform(.01,1e12);life=rng.uniform(.01,100)
  if fn=='DB':
   rate=(rng.randrange(-4000,1000)+.5)/1000
   salvage=cost*math.pow(1-rate,life)
   for _ in range(rng.randrange(1,5)):salvage=math.nextafter(salvage,rng.choice([0.,math.inf]))
   add([cost,salvage,life,rng.uniform(.001,life+1),rng.uniform(1,12.999)],'independent-positive-negative-half-rounded-rate')
  else:
   factor=life
   for _ in range(rng.randrange(1,5)):factor=math.nextafter(factor,rng.choice([0.,math.inf]))
   period=rng.choice([.5,1.,math.nextafter(1.,math.inf),rng.uniform(1.,max(1.,life))])
   add([cost,cost*rng.random(),life,period,factor],'independent-rate-one-and-cancellation')
 for i in range(500):
  cost=rng.choice([0.,1.,1e300,1e-300]);salvage=rng.choice([0.,1.,1e300,1e-300])
  life=10**rng.uniform(-307,307)
  period=rng.choice([life,.5,1.,2.])
  extra=rng.uniform(1,12.999) if fn=='DB' else 10**rng.uniform(-307,307)
  add([cost,salvage,life,period,extra],'independent-extreme-life-and-zero-shortcuts')
 path=out/f"second-heldout-{fn.lower()}.json"
 doc=dict(function=fn,probes=[dict(probe=dict(id=f'w111dep-second-{fn}-{i:05d}',args=list(k)),probe_region=tag) for i,(k,tag) in enumerate(rows.items())])
 path.write_text(json.dumps(doc,indent=2)+"\n",encoding="utf-8")
 (out/f'second-heldout-{fn.lower()}-manifest.json').write_text(json.dumps(dict(rows=len(rows),excluded_prior_tuples=len(prior),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),input_policy='normal binary64 plus positive zero; all prior tuples excluded'),indent=2)+"\n",encoding='utf-8')
 print(json.dumps(dict(path=str(path),rows=len(rows))))
