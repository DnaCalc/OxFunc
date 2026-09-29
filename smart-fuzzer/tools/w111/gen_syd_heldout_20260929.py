"""Independent SYD numeric cohort after stage and arithmetic-store discovery."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=root/'smart-fuzzer/cache/w111-depreciation-20260929'
source=root/'crates/oxfunc_core/src/functions/depreciation_family.rs';snapshot=out/'candidate-syd-v1.rs'
if snapshot.exists():raise SystemExit('Candidate v1 already exists; do not overwrite')
snapshot.write_bytes(source.read_bytes())
(out/'freeze-syd-v1.json').write_text(json.dumps(dict(frozen_utc=datetime.now(timezone.utc).isoformat(),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),snapshot=snapshot.name,scope='SYD negative cost, published comparison/difference/denominator, lifetime-plus-one stage and x87 multiply/divide.3699 discovery observations; full numeric and typed claims remain partial.'),indent=2)+'\n',encoding='utf-8')
bits=lambda n:'0x'+struct.pack('>d',float(n)).hex();rng=random.Random(202609292027)
prior=set()
for path in [root/'smart-fuzzer/runs/w111-broad-20260929/answers/answers-syd.json',out/'discriminator-syd-answers.json',out/'refinement-syd-answers.json']:
 prior.update(tuple(w['args']) for w in json.loads(path.read_text(encoding='utf-8-sig'))['witnesses'])
rows={}
def add(a,tag):
 if all(math.isfinite(x) and (x==0 or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000' for x in a):
  k=tuple(map(bits,a))
  if k not in prior:rows.setdefault(k,tag)
for _ in range(5000):
 c=10**rng.uniform(-307,307)*rng.choice([-1.,1.]);s=10**rng.uniform(-307,307)*rng.choice([-1.,1.,1.,1.]);l=10**rng.uniform(-307,307)
 add([c,s,l,l*rng.uniform(.0001,1.05)],'fresh-full-exponent-signed-cost')
for _ in range(3000):
 l=rng.choice([2**-1022,2**-1021,math.sqrt(1.7976931348623157e308),2**-53,2**53])*rng.uniform(.99999999999,1.00000000001)
 p=rng.choice([l,l*rng.uniform(.01,1.),math.nextafter(l,math.inf),math.nextafter(l,0.)]);c=10**rng.uniform(-307,307)
 add([c,c*rng.uniform(0.,2.),l,p],'fresh-admission-and-denominator-boundaries')
for _ in range(1500):
 c=2**-1022*2**rng.uniform(0,50);s=math.nextafter(c,rng.choice([0.,math.inf]));l=10**rng.uniform(-307,307)
 add([c,s,l,l*rng.uniform(.01,1.)],'fresh-published-subnormal-basis')
for _ in range(2500):
 c=rng.uniform(-1e12,1e12);s=rng.uniform(0.,1e12);l=rng.uniform(.0001,150);p=l*rng.uniform(.001,1.01)
 add([c,s,l,p],'fresh-ordinary-rounding-and-error-boundary')
path=out/'heldout-syd.json';path.write_text(json.dumps(dict(function='SYD',probes=[dict(probe=dict(id=f'w111syd-heldout-{i:05d}',args=list(k)),probe_region=t) for i,(k,t) in enumerate(rows.items())]),indent=2)+'\n',encoding='utf-8')
(out/'heldout-syd-manifest.json').write_text(json.dumps(dict(rows=len(rows),seed=202609292027,excluded_prior_tuples=len(prior),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest()),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),rows=len(rows))))
