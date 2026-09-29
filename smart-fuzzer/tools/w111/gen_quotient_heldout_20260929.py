"""Independent QUOTIENT signed-zero, overflow and integer-boundary probes."""
import hashlib,json,math,random,struct
from pathlib import Path
import gen_broad_typed_20260929 as g

seed=202609291629
rng=random.Random(seed)
def bits(n):return f'0x{struct.unpack("<Q",struct.pack("<d",float(n)))[0]:016x}'
pairs=[]
for _ in range(1200):
    values=[]
    for _ in range(2):
        raw=(rng.getrandbits(1)<<63)|(rng.randrange(1,2047)<<52)|rng.getrandbits(52)
        values.append(struct.unpack('<d',struct.pack('<Q',raw))[0])
    pairs.append(values)
for _ in range(160):
    denominator=rng.uniform(.5,2)*10**rng.randrange(-20,21)*rng.choice([-1,1])
    integer=rng.randrange(-1000,1001)
    center=integer*denominator
    for numerator in [math.nextafter(center,-math.inf),center,math.nextafter(center,math.inf)]:
        pairs.append([numerator,denominator])
for x in [0,.125,-.125,.999,-.999,1,-1,math.nextafter(1,0),math.nextafter(-1,0),
          2.2250738585072014e-308,-2.2250738585072014e-308,1.7976931348623157e308,-1.7976931348623157e308]:
    for y in [0,.125,-.125,1,-1,2,-2,2.2250738585072014e-308,1.7976931348623157e308]:pairs.append([x,y])
probes=[{'probe':{'id':f'quotient-heldout-{seed}-{i:04d}','args':[bits(x),bits(y)]}} for i,(x,y) in enumerate(pairs)]
out=Path('smart-fuzzer/runs/w111-broad-20260929/quotient-heldout.json')
out.write_text(json.dumps({'function':'QUOTIENT','probes':probes},separators=(',',':')),encoding='utf-8')
g.TRANCHE='w111-quotient-typed-heldout-20260929'
for axis in [0,1]:
    for i,val in enumerate([g.n(0),g.n(-.125),g.n(.125),g.n(2),g.b(False),g.b(True),
                           g.t('2'),g.t('200%'),g.t('(2)'),g.t(' - .5 '),g.t('x'),
                           {'kind':'empty_cell'},{'kind':'missing_arg'},
                           {'kind':'error','code':'NA'},{'kind':'error','code':'Div0'}]):
        args=[g.n(-2),g.n(3)];args[axis]=val
        g.emit('QUOTIENT',f'axis{axis}-value{i}',args,axis='quotient_typed_heldout')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111quotienttyped-')
typed=out.with_name('quotient-typed-heldout.json')
typed.write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    'tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},
    ensure_ascii=False,separators=(',',':')),encoding='utf-8')
freeze={'seed':seed,'numeric_rows':len(probes),'typed_rows':len(g.cases),
    'phase':'before_independent_oracle_capture','source_sha256':{p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
        for p in ['crates/oxfunc_core/src/functions/quotient_fn.rs','crates/oxfunc_core/src/coercion.rs',__file__]},
    'input_sha256':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [out,typed]}}
Path('docs/function-lane/evidence/w111-broad-20260929/quotient/candidate-freeze.json').write_text(json.dumps(freeze,indent=2),encoding='utf-8')
print(len(probes),'numeric',len(g.cases),'typed',out)
