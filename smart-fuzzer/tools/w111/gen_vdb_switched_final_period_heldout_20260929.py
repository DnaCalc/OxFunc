"""Fresh bounded VDB validation after initial stores and final-period termination."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
root=Path(__file__).resolve().parents[3];cache=root/'smart-fuzzer/cache/w111-depreciation-20260929'
source=root/'smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/depreciation_graph_explorer.rs';mode=52299816765658
freeze={'frozen_utc':datetime.now(timezone.utc).isoformat(),'mode':mode,'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
        'source_utf8':source.read_text(),'discovery_exact_observations':42436,
        'scope':'Research only. Stored cursor with x87 initial book/product and partial-product stores; final remaining period terminates at end-cursor<=1; general negative-basis annual clipping. Life/end<1000, moderate normal exponents; huge-index progress and full publication lanes remain open. Production unchanged.'}
(cache/'switched-research-final-period-freeze.json').write_text(json.dumps(freeze,indent=2)+'\n')
encode=lambda x:'0x'+struct.pack('>d',float(x)).hex();r=random.Random(0x202609292030f84);known=set()
for name in ['vdb-switched-bounded-discovery.json','heldout-vdb-switched-research-answers.json','second-heldout-vdb-switched-research-answers.json','third-heldout-vdb-switched-research-answers.json',
             'refinement3-vdb-switched-answers.json','refinement4-vdb-switched-answers.json','refinement5-vdb-switched-answers.json','typed-vdb-switched-recheck-research-answers.json']:
    for w in json.loads((cache/name).read_text(encoding='utf-8-sig'))['witnesses']:
        args=w['args']+[encode(2)]*(6-len(w['args']))
        if len(args)==6:args.append(encode(0))
        known.add(tuple(args))
probes=[];regions={}
def emit(a,region):
    assert all(math.isfinite(x) and (x==0 or abs(x)>=float.fromhex('0x1p-1022')) for x in a)
    assert 0<=a[3]<=a[4]<=a[2]<1000
    bits=tuple(map(encode,a))
    if bits in known:return
    known.add(bits);regions[region]=regions.get(region,0)+1
    probes.append({'probe':{'id':f'w111vdb-general-independent-{len(probes):05d}','args':list(bits)},'probe_region':region})
for i in range(11000):
    c=math.ldexp(r.uniform(1,2),r.randrange(-300,301))
    s=r.choice([c,math.nextafter(c,0),math.nextafter(c,math.inf),c*r.uniform(1.0001,8.),c*r.uniform(.3,1.)])
    life=math.ldexp(r.uniform(1,2),r.randrange(-4,9))
    start=r.choice([0.,life*r.uniform(0,.92),min(life*.7,r.choice([.5,math.nextafter(.5,0),math.nextafter(.5,math.inf),.5-2.**-13,.5+2.**-13,r.uniform(0,2.)]))])
    end=min(life,math.nextafter(start+1.,math.inf if i%2 else 0.)) if i%3==0 else start+(life-start)*r.uniform(.01,1.)
    factor=r.choice([0.,2.**-80,2.**-40,.125,.5,1.,2.,4.,r.uniform(.01,9.)])
    emit([c,s,life,start,end,factor,0.],'basis-neighbors-zero-tiny-factor')
for i in range(5000):
    c=math.ldexp(r.uniform(1,2),r.randrange(-450,451));s=c*r.uniform(0,.999)
    life=r.uniform(.03125,900.);start=life*r.uniform(0,.999)
    end=r.choice([start,math.nextafter(start,math.inf),start+(life-start)*r.uniform(.001,1.)])
    emit([c,s,life,start,end,r.uniform(.001,15.),0.],'independent-positive-schedule')
for i in range(4000):
    start=math.ldexp(r.uniform(1,2),r.randrange(-15,7));steps=r.randrange(1,150);cursor=start
    for _ in range(steps):cursor+=1.
    end=r.choice([cursor,math.nextafter(cursor,0),math.nextafter(cursor,math.inf)])
    life=end+r.uniform(.001,80.);c=r.uniform(1,1e9);s=c*r.uniform(0,1.5)
    emit([c,s,life,start,end,r.choice([0.,.25,.5,1.,2.,3.75]),0.],'stored-cursor-integer-boundaries')
path=cache/'fourth-heldout-vdb-switched-research.json';path.write_text(json.dumps({'function':'VDB','probes':probes},indent=2)+'\n')
(cache/'fourth-heldout-vdb-switched-research-manifest.json').write_text(json.dumps(dict(rows=len(probes),regions=regions,
    source_sha256=freeze['source_sha256'],generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),mode=mode,seed='0x202609292030f84',
    qualification='Fresh after x87 initial/partial stores and final-remaining-period freeze; earlier argument tuples excluded. All life/end<1000; no dangerous huge-period inputs. Exponents deliberately avoid underflow/overflow publication.'),indent=2)+'\n')
print(json.dumps(dict(path=str(path),rows=len(probes),regions=regions)))
