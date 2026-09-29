"""Independent bounded switched-VDB validation after the stored-cursor graph."""
import hashlib,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
root=Path(__file__).resolve().parents[3]
cache=root/'smart-fuzzer/cache/w111-depreciation-20260929'
source=root/'smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/depreciation_graph_explorer.rs'
mode=1722281887954
freeze={'frozen_utc':datetime.now(timezone.utc).isoformat(),'mode':mode,
        'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'source_utf8':source.read_text(),
        'scope':'Research only, finite normal numerical inputs and bounded end<1000; stored cursor +=1 with cursor<end loop, partial take end-cursor. Huge-index non-progress remains unresolved. Production unchanged.',
        'discovery_exact_observations':13436}
(cache/'switched-research-cursor-freeze.json').write_text(json.dumps(freeze,indent=2)+'\n')
encode=lambda x:'0x'+struct.pack('>d',float(x)).hex()
r=random.Random(0x202609291955c07)
known=set()
for name in ['vdb-switched-bounded-discovery.json','heldout-vdb-switched-research-answers.json','refinement3-vdb-switched-answers.json',
             'refinement4-vdb-switched-answers.json','refinement5-vdb-switched-answers.json','typed-vdb-switched-recheck-research-answers.json']:
    for w in json.loads((cache/name).read_text(encoding='utf-8-sig'))['witnesses']:
        args=w['args']+[encode(2)]*(6-len(w['args']))
        if len(args)==6:args.append(encode(0))
        known.add(tuple(args))
probes=[];regions={}
def emit(args,region):
    assert len(args)==7 and all(math.isfinite(x) for x in args)
    assert all(x==0 or abs(x)>=float.fromhex('0x1p-1022') for x in args)
    assert 0<=args[3]<=args[4]<=args[2]<1000
    bits=tuple(map(encode,args))
    if bits in known:return
    known.add(bits);regions[region]=regions.get(region,0)+1
    probes.append({'probe':{'id':f'w111vdb-cursor-independent-{len(probes):05d}','args':list(bits)},'probe_region':region})
for i in range(6500):
    cost=math.ldexp(r.uniform(1,2),r.randrange(-400,401))
    salvage=cost*r.uniform(0,.98)
    life=math.ldexp(r.uniform(1,2),r.randrange(-8,9))
    factor=2. if i<2500 else r.choice([0.,.5,1.,1.5,math.nextafter(2.,0),2.,math.nextafter(2.,math.inf),3.,6.,r.uniform(.01,12.)])
    start=life*r.uniform(0,.97);end=start+(life-start)*r.uniform(.001,1.)
    emit([cost,salvage,life,start,end,factor,0.],'broad-cost-exponents-life-factor')
for i in range(2500):
    cost=r.uniform(100,1e7);salvage=cost*r.uniform(0,.95)
    start=math.ldexp(r.uniform(1,2),r.randrange(-12,8))
    steps=r.randrange(1,80);cursor=start
    for _ in range(steps):cursor+=1.
    life=max(cursor+1.,r.uniform(cursor+1.,min(999.,cursor+100.)))
    endpoint=r.choice([cursor,math.nextafter(cursor,0.),math.nextafter(cursor,math.inf)])
    emit([cost,salvage,life,start,endpoint,r.choice([.75,1.,1.5,2.,2.5,4.]),0.],'stored-cursor-nextafter-end')
for i in range(2000):
    cost=math.ldexp(r.uniform(1,2),r.randrange(-200,201));salvage=cost*r.uniform(1.00001,4.)
    life=r.uniform(.125,100.);start=life*r.uniform(0,.9);end=start+(life-start)*r.uniform(.01,1.)
    emit([cost,salvage,life,start,end,r.choice([0.,.5,1.,2.,4.]),0.],'cost-below-salvage')
for i in range(2000):
    cost=math.ldexp(r.uniform(1,2),r.randrange(-300,301));salvage=cost*r.uniform(0,1.)
    life=r.uniform(.25,150.);start=life*r.uniform(0,.95)
    end=math.nextafter(start,math.inf) if i%2 else start
    emit([cost,salvage,life,start,end,r.uniform(.05,8.),0.],'zero-or-adjacent-interval')
path=cache/'second-heldout-vdb-switched-research.json'
path.write_text(json.dumps({'function':'VDB','probes':probes},indent=2)+'\n')
(cache/'second-heldout-vdb-switched-research-manifest.json').write_text(json.dumps(dict(rows=len(probes),regions=regions,
    generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
    source_sha256=freeze['source_sha256'],mode=mode,seed='0x202609291955c07',
    qualification='Fresh after frozen stored-cursor graph; all earlier normalized numeric argument tuples excluded. Finite normal inputs; end/life<1000. No dangerous large-index cases.'),indent=2)+'\n')
print(json.dumps(dict(path=str(path),rows=len(probes),regions=regions)))
