"""Independent admission/period probes for DB, DDB and VDB; no Excel calls."""
import hashlib
import itertools
import json
import struct
from pathlib import Path
import gen_broad_typed_20260929 as g

out=g.ROOT/"smart-fuzzer/cache/w111-depreciation-20260929";out.mkdir(exist_ok=True)
bits=lambda n:"0x%016x"%struct.unpack("<Q",struct.pack("<d",float(n)))[0]
rows={fn:{} for fn in ("DB","DDB","VDB")}
def add(fn,args,tag):rows[fn].setdefault(tuple(map(bits,args)),tag)
for life,period,month in itertools.product([0,.5,1,1.5,2,5.5,10],[0,.25,.5,.99,1,1.1,1.99,2,2.25,5.5,10,10.99,11],[0,.5,1,1.5,6,11.99,12,12.1]):
    add("DB",[1000,100,life,period,month],"fractional-life-period-month-admission")
for life,period,factor in itertools.product([0,.5,1,1.5,2,5.5,10],[0,.25,.5,.99,1,1.1,1.99,2,2.25,5.5,10,10.99,11],[0,.5,1,2,3]):
    add("DDB",[1000,100,life,period,factor],"fractional-period-exponent-and-rate-limit")
for life,start,end,factor,switch in itertools.product([.5,1,2.5,10],[0,.5,1,1.5,2,9,10],[0,.5,1,2,2.5,5,10,11],[.5,2],[0,1]):
    add("VDB",[1000,100,life,start,end,factor,switch],"interval-life-admission-and-switch")
for cost,salvage in itertools.product([-1000,-1,0,1,1000],[0,-1,1,1000,2000]):
    for life,period in [(1,.5),(10,1),(10,2),(10,10)]:
        add("DB",[cost,salvage,life,period,12],"cost-salvage-domain")
        add("DDB",[cost,salvage,life,period,2],"cost-salvage-domain")
        add("VDB",[cost,salvage,life,0,period,2,0],"cost-salvage-domain")
for fn,data in rows.items():
    doc={"function":fn,"probes":[{"probe":{"id":f"w111dep-{fn}-{i:05d}","args":list(args)},"probe_region":tag} for i,(args,tag) in enumerate(data.items())]}
    path=out/f"discriminator-{fn.lower()}.json";path.write_text(json.dumps(doc,indent=2)+"\n",encoding="utf-8")
    print(json.dumps({"path":str(path),"rows":len(data)}))
g.TRANCHE="w111-depreciation-typed-20260929"
for fn,args in {"DB":[g.n(1000),g.n(100),g.n(10),g.n(2)],"DDB":[g.n(1000),g.n(100),g.n(10),g.n(2)],"VDB":[g.n(1000),g.n(100),g.n(10),g.n(1),g.n(2)]}.items():
    g.variants(fn,args,positions=range(len(args)))
    for pos in range(len(args)):
        changed=args.copy();changed[pos]=g.missing();g.emit(fn,f"missing-{pos}",changed)
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111dep-')
doc={"schema_version":"oxfunc.smart_fuzzer.scenario_seed_case_set.v0","tranche_id":g.TRANCHE,"generator_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),"cases":g.cases,"tranches":[{"tranche_id":g.TRANCHE,"case_ids":[c['case_id'] for c in g.cases]}]}
path=out/"typed.json";path.write_text(json.dumps(doc,indent=2)+"\n",encoding="utf-8");path.with_suffix('.jsonl').write_text(''.join(json.dumps(c)+'\n' for c in g.cases),encoding='utf-8')
print(json.dumps({"path":str(path),"rows":len(g.cases)}))
