"""Independent no-switch VDB heldout; switched VDB is outside this candidate."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/"smart-fuzzer/cache/w111-depreciation-20260929"
source=g.ROOT/"crates/oxfunc_core/src/functions/depreciation_family.rs"
snapshot=out/"candidate-vdb-noswitch-v5.rs"
if snapshot.exists():raise SystemExit("Refusing to overwrite no-switch candidate")
snapshot.write_bytes(source.read_bytes())
(out/"freeze-vdb-noswitch-v5.json").write_text(json.dumps(dict(frozen_utc=datetime.now(timezone.utc).isoformat(),source=str(source.relative_to(g.ROOT)),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),snapshot=snapshot.name,scope="Only no_switch=true: annual DDB now uses x87 bounded integer products, 32-bit power dispatch, and x87 base subtraction. Partial products use independent x87 stores and flush subnormals, while their final subtraction preserves a nonzero subnormal. Discovery includes the failed fourth cohort and 660 stage probes. Switched schedule and huge-period progress remain open. Earlier no-switch heldouts are discovery."),indent=2)+"\n",encoding="utf-8")
bits=lambda n:'0x'+struct.pack('>d',float(n)).hex()
rng=random.Random(202609292028)
prior=set()
for path in [g.ROOT/"smart-fuzzer/runs/w111-extended-numeric-20260929/answers/answers-vdb.json",out/"answers-vdb.json",out/"refinement-vdb-answers.json",out/"heldout-vdb-noswitch-answers.json",out/"refinement2-vdb-noswitch-answers.json",out/"second-heldout-vdb-noswitch-answers.json",out/"oracle-only-vdb-large-index-answers.json",out/"third-heldout-vdb-noswitch-answers.json",out/"refinement3-vdb-noswitch-answers.json",out/"fourth-heldout-vdb-noswitch-answers.json",out/"refinement4-vdb-noswitch-answers.json"]:
 prior.update(tuple(w['args']) for w in json.loads(path.read_text(encoding='utf-8-sig'))['witnesses'])
rows={}
def add(a,tag):
 if not all(math.isfinite(x) and (x==0 or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000' for x in a):return
 k=tuple(map(bits,a))
 if k not in prior:rows.setdefault(k,tag)
for i in range(4500):
 cost=10**rng.uniform(-300,300);salvage=cost*rng.uniform(0,2)
 life=rng.uniform(.001,80);start=rng.uniform(0,life);end=rng.uniform(start,life)
 factor=rng.choice([0.,rng.uniform(.001,5),10**rng.uniform(-307,307)])
 add([cost,salvage,life,start,end,factor,1.],"fresh-no-switch-wide-exponents-and-fractional-bounds")
for i in range(5000):
 cost=10**rng.uniform(-200,200);salvage=cost*rng.random()
 life=rng.uniform(.1,20);start=rng.uniform(0,life)
 end=math.nextafter(start,math.inf)
 for _ in range(rng.randrange(1,8)):end=math.nextafter(end,math.inf)
 add([cost,salvage,life,start,end,rng.uniform(0,4),1.],"fresh-adjacent-period-bounds")
for i in range(500):
 life=rng.uniform(1,30);start=float(rng.randrange(0,math.ceil(life)))
 start=math.nextafter(start,math.inf if rng.getrandbits(1) else 0.)
 end=min(life,start+rng.choice([.125,.5,1.,1.25]))
 cost=10**rng.uniform(-100,100)
 add([cost,cost*rng.uniform(0,2),life,start,end,rng.choice([0.,1.,2.,life]),1.],"fresh-integer-year-neighbors")
for i in range(2000):
 start=rng.uniform(2**20,2**32-4096.)
 width=rng.choice([.0001,.125,.5,1.,2.,3.125])
 end=start+width
 life=end*rng.uniform(1.01,3.)
 cost=10**rng.uniform(-250.,250.)
 add([cost,cost*rng.choice([0.,rng.random(),1.5]),life,start,end,rng.choice([0.,.125,.875,1.75,3.,17.]),1.],"fresh-large-but-progressing-short-interval")
for i in range(5000):
 life=rng.uniform(1.,80.);factor=10**rng.uniform(-250,-1);rate=factor/life
 cost=2**-1022*rng.uniform(.25,20.)/rate
 start=rng.uniform(0.,life);end=rng.uniform(start,life)
 add([cost,cost*rng.uniform(0.,1.5),life,start,end,factor,1.],"fresh-minimum-normal-annual-and-partial-products")
path=out/"fifth-heldout-vdb-noswitch.json"
path.write_text(json.dumps(dict(function="VDB",probes=[dict(probe=dict(id=f'w111vdbns-fifth-{i:05d}',args=list(k)),probe_region=tag) for i,(k,tag) in enumerate(rows.items())]),indent=2)+"\n",encoding='utf-8')
(out/'fifth-heldout-vdb-noswitch-manifest.json').write_text(json.dumps(dict(rows=len(rows),excluded_prior_tuples=len(prior),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),input_policy='normal binary64 plus positive zero; all prior exact tuples excluded; large indices strictly below 2^32-4096 and intervals at most 3.125 years; no dangerous huge-start row'),indent=2)+"\n",encoding='utf-8')
print(json.dumps(dict(path=str(path),rows=len(rows))))
