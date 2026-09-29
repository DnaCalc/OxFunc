"""Independent full normal-exponent VDB inputs with bounded period endpoints."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
root=Path(__file__).resolve().parents[3];cache=root/'smart-fuzzer/cache/w111-depreciation-20260929'
source=root/'crates/oxfunc_core/src/functions/depreciation_family.rs';mode='production-explicit-v1'
freeze={'frozen_utc':datetime.now(timezone.utc).isoformat(),'mode':mode,'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
        'source_utf8':source.read_text(),'scope':'Production translation: bounded period start/end<1000; next corpus admits arbitrary finite normal lifetime, cost, salvage and factor. Ordered finite checks, operation-specific subnormal publication, absolute switch comparison threshold and advance book store now included. Huge-period progress remains open; known huge-period progress remains open.'}
(cache/'switched-production-v1-freeze.json').write_text(json.dumps(freeze,indent=2)+'\n')
small=float.fromhex('0x1p-1022');large=float.fromhex('0x1.fffffffffffffp+1023')
encode=lambda x:'0x'+struct.pack('>d',float(x)).hex();decode=lambda x:struct.unpack('>d',struct.pack('>Q',x))[0]
r=random.Random(0x20260929fe75d00b);known=set();probes=[];regions={}
for p in ['second-heldout-vdb-switched-publication-answers.json','heldout-vdb-switched-publication-answers.json','refinement6-vdb-switched-publication-answers.json','refinement7-vdb-switch-comparison-answers.json','discovery-vdb-switched-publication-answers.json','fourth-heldout-vdb-switched-research-answers.json',
          'third-heldout-vdb-switched-research-answers.json','second-heldout-vdb-switched-research-answers.json',
          'heldout-vdb-switched-research-answers.json','vdb-switched-bounded-discovery.json',
          'refinement3-vdb-switched-answers.json','refinement4-vdb-switched-answers.json',
          'refinement5-vdb-switched-answers.json','typed-vdb-switched-recheck-research-answers.json']:
 for row in json.loads((cache/p).read_text(encoding='utf-8-sig'))['witnesses']:
  args=row['args']+[encode(2)]*(6-len(row['args']))
  if len(args)==6:args.append(encode(0))
  known.add(tuple(args))
def normal():return decode((r.randrange(1,2047)<<52)|r.getrandbits(52))
def emit(a,region):
 if not all(math.isfinite(v) and (v==0 or abs(v)>=small) for v in a):return False
 assert 0<=a[3]<=a[4]<=a[2] and a[4]<1000
 bits=tuple(map(encode,a))
 if bits in known:return False
 known.add(bits);regions[region]=regions.get(region,0)+1
 probes.append({'probe':{'id':f'w111vdb-dispatch-independent-{len(probes):05d}','args':list(bits)},'probe_region':region});return True
for region,count in [('all-normal-exponents',2000),('tiny-stage-publication',2000),('overflow-and-clipping',2000),('absolute-switch-threshold',2000)]:
 while regions.get(region,0)<count:
  if region=='all-normal-exponents':
   cost=normal();salvage=r.choice([0.,normal(),cost,math.nextafter(cost,0)])
   life=normal();factor=r.choice([0.,normal(),.5,2.,8.])
  elif region=='tiny-stage-publication':
   cost=decode((r.randrange(1,30)<<52)|r.getrandbits(52));salvage=r.choice([0.,cost,math.nextafter(cost,0),cost*r.uniform(.25,2.)])
   life=r.choice([r.uniform(.0625,900.),normal()]);factor=r.choice([0.,decode((r.randrange(1,30)<<52)|r.getrandbits(52)),r.uniform(.01,12.)])
  elif region=='absolute-switch-threshold':
   life=r.choice([2.,4.,8.,16.,32.,64.,128.]);money=r.uniform(2.*life,32.*life);cost=money*small
   salvage=r.choice([0.,cost/8.,cost/2.]);center=1.-salvage/cost-life*small/(16.*cost)
   factor=r.choice([center,math.nextafter(center,0.),math.nextafter(center,math.inf),center+r.uniform(-.02,.02)])
   if factor<0:continue
  else:
   cost=decode((r.randrange(2000,2047)<<52)|r.getrandbits(52));salvage=r.choice([0.,cost,math.nextafter(cost,0),large,normal()])
   life=r.choice([r.uniform(.0625,900.),normal()]);factor=r.choice([0.,decode((r.randrange(2000,2047)<<52)|r.getrandbits(52)),r.uniform(.01,12.)])
  limit=min(life,900.)
  start=0. if limit<small*16 or r.randrange(4)==0 else limit*r.uniform(.001,.95)
  end=limit if limit<small*16 or r.randrange(3)==0 else start+(limit-start)*r.uniform(.001,1.)
  emit([cost,salvage,life,start,end,factor,0.],region)
path=cache/'heldout-vdb-switched-dispatch.json';path.write_text(json.dumps({'function':'VDB','probes':probes},indent=2)+'\n')
(cache/'heldout-vdb-switched-dispatch-manifest.json').write_text(json.dumps(dict(rows=len(probes),regions=regions,
    batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    source_sha256=freeze['source_sha256'],seed='0x20260929fe75d00b',mode=mode,
    qualification='Fresh after publication freeze. All inputs finite normal/+0; lifetime may span all normal exponents, while start/end<1000 keep loops bounded. No huge-period progress claims.'),indent=2)+'\n')
print(json.dumps(dict(path=str(path),rows=len(probes),regions=regions)))
