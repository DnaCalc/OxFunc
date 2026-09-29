"""Independent frozen-candidate GCD/LCM controls: numeric bits and typed origins."""
import hashlib,itertools,json,math,random,struct
from datetime import datetime,timezone
from pathlib import Path
import gen_broad_typed_20260929 as g

SEED=202609292701
OUT=Path('smart-fuzzer/runs/w111-gcd-lcm-heldout-20260929')
EVIDENCE=Path('docs/function-lane/evidence/w111-broad-20260929/gcd-lcm')
def bits(n):return '0x'+struct.pack('>d',float(n)).hex()
def main():
    rng=random.Random(SEED);limit=2**53;rows=[]
    # Draw complete significands, including fractions, negatives and huge finite
    # normals, rather than relying on uniform()'s limited relative precision.
    for _ in range(1200):
        args=[]
        for _ in range(rng.randrange(1,9)):
            exponent=rng.choice([rng.randrange(1,1023),rng.randrange(1023,1077),rng.randrange(1077,2047)])
            raw=(rng.getrandbits(52)|(exponent<<52)|(rng.randrange(12)==0)<<63)
            args.append(struct.unpack('>d',raw.to_bytes(8,'big'))[0])
        rows.append(args)
    for _ in range(800):
        mode=rng.randrange(4);count=rng.randrange(1,15)
        if mode==0:
            common=rng.randrange(1,2**28);args=[common*rng.randrange(1,2**18) for _ in range(count)]
        elif mode==1:
            args=[rng.randrange(1,2**rng.randrange(1,54)) for _ in range(count)]
        elif mode==2:
            args=[rng.randrange(1,10000)+rng.random() for _ in range(count)]
        else:
            args=[2**rng.randrange(0,53) for _ in range(count)]
        if rng.randrange(5)==0:args.insert(rng.randrange(len(args)+1),0)
        rows.append(args)
    # New factor centers near the rounded product boundary, with repeat/order
    # controls and zero cancellation. Each center is fresh after candidate freeze.
    for factor in rng.sample(list(range(5,300,2)),45):
        base=limit//factor
        for delta in [-2,-1,0,1,2]:
            b=base+delta
            rows.extend([[factor,b],[factor,factor,b],[b,factor,factor],[2,factor,b],[factor,b,0],[0,factor,b]])
    for _ in range(100):
        n=rng.randrange(1,2**52)
        rows.extend([[math.nextafter(float(n),-math.inf),rng.randrange(1,19)],[math.nextafter(float(n),math.inf),rng.randrange(1,19)]])
    rows.extend([[1]*255,[0]+[2**53-1]*254,[2**53]*255,[-.01]+[0]*254])
    unique={tuple(map(bits,row)):None for row in rows};OUT.mkdir(parents=True,exist_ok=True);(OUT/'batches').mkdir(exist_ok=True)
    for fn in ['GCD','LCM']:
        (OUT/'batches'/f'batch-{fn.lower()}.json').write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'w111gcdlcmheldout-{fn}-{i:05d}','args':list(row)}} for i,row in enumerate(unique)]},separators=(',',':')),encoding='utf-8')
    g.TRANCHE=OUT.name
    pool=[g.n(0),g.n(6.75),g.n(-.25),g.n(limit),g.n(limit+2),g.t(' 125% '),g.t('12.999'),g.t('x'),g.t(''),g.b(True),g.b(False),g.e('NA'),g.e('Div0'),g.e('Value'),g.e('Ref')]
    for fn in ['GCD','LCM']:
        for i in range(160):
            count=rng.randrange(1,9);cells=[rng.choice(pool) if rng.randrange(3)==0 else g.n(rng.randrange(1,2**24)) for _ in range(count)]
            if i%4==0:cells[rng.randrange(count)]=g.n(limit//rng.choice([3,5,7,11,13]))
            # Paired scalar, computed array and referenced array with identical
            # ordered content makes the origin and reduction contract falsifiable.
            g.emit(fn,f'scalar-{i}',cells,axis='heldout_order_and_coercion')
            matrix=[cells] if i%2==0 else [[v] for v in cells]
            g.emit(fn,f'array-{i}',[g.a(matrix)],axis='heldout_order_and_coercion')
            target=f'B200:{chr(ord("B")+count-1)}200' if i%2==0 else f'B200:B{199+count}'
            g.emit(fn,f'reference-{i}',[g.r(target)],[g.fix(target,g.a(matrix))],axis='heldout_order_and_coercion')
        for i in range(70):
            count=rng.randrange(1,8);cells=[g.blank() if rng.randrange(3)==0 else rng.choice(pool) for _ in range(count)]
            target=f'B200:{chr(ord("B")+count-1)}200'
            args=[g.r(target),g.n(rng.randrange(1,30))]
            if i%3==0:args.reverse()
            if i%5==0:args.insert(rng.randrange(len(args)+1),g.missing())
            g.emit(fn,f'blank-missing-{i}',args,[g.fix(target,g.a([cells]))],axis='heldout_blank_shape_and_missing')
        for i in range(30):
            count=rng.randrange(2,8);cells=[g.n(limit//rng.choice([3,5,7,11,13])),g.n(rng.choice([2,3,5,7,11]))]*count
            split=rng.randrange(1,len(cells));args=[g.a([cells[:split]]),g.a([cells[split:]])]
            if i%3==0:args.insert(1,g.missing())
            g.emit(fn,f'grouped-{i}',args,axis='heldout_grouped_reverse_reduction')
    for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111gcdlcmheldout-')
    (OUT/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
    files=['crates/oxfunc_core/src/functions/'+n for n in ['gcd_fn.rs','lcm_fn.rs','gcd_lcm_common.rs']]+['formal/lean/OxFunc/GcdLcmReduction.lean','formal/lean/OxFunc/Functions/GcdFn.lean','formal/lean/OxFunc/Functions/LcmFn.lean']
    freeze={'frozen_utc':datetime.now(timezone.utc).isoformat(),'seed':SEED,'source_sha256':{p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in files},'numeric_rows_per_function':len(unique),'typed_rows':len(g.cases),'scope_completeness':'scope_partial','target_completeness':'target_partial','integration_completeness':'partial','open_lanes':['independent capture pending','shared contextual numeric-text grammar','HO-FN-027 acknowledgement'],'design':'Independent full-bit normals, factors/common multiples, normal integer neighbors, 255-argument controls, fresh product-boundary factors and reverse grouped-array reduction; paired typed origins, blank shape and error order.'}
    (OUT/'candidate-freeze.json').write_text(json.dumps(freeze,indent=2),encoding='utf-8');(EVIDENCE/'candidate-freeze.json').write_text(json.dumps(freeze,indent=2),encoding='utf-8');print(len(unique),'numeric per function;',len(g.cases),'typed')
if __name__=='__main__':main()
