"""Fresh heldout after formatter freeze; all admission is normal binary64 or +0."""
import decimal,hashlib,json,math,random,struct,sys
from collections import Counter
from pathlib import Path
import gen_broad_typed_20260929 as g

SEED=202609292307
def bits(n):return struct.pack('>d',n).hex()
def from_bits(b):return struct.unpack('>d',bytes.fromhex(b))[0]
def main():
    g.TRANCHE='w111-numeric-to-text-heldout-20260929';rng=random.Random(SEED);values={}
    old=json.loads(Path('smart-fuzzer/runs/w111-numeric-to-text-discovery-20260929/design.json').read_text(encoding='utf-8'))
    previous={row['bits'] for row in old['inputs']}
    def add(n,origin):
        if math.isfinite(n) and (n==0 or abs(n)>=sys.float_info.min) and bits(n) not in previous:
            values.setdefault(bits(n),{'value':n,'origin':origin})
    def neighborhood(n,origin):
        if not math.isfinite(n):return
        for z in [math.nextafter(n,0),n,math.nextafter(n,math.inf)]:
            add(z,origin);add(-z,origin)
    decimal.getcontext().prec=400
    for exponent in [-308,-307,-100,-99,-98,-22,-21,-20,-19,-8,-7,-6,-5,-4,0,13,14,15,18,19,20,21,97,98,99,100,307]:
        for coefficient in ['1.000000000000005','9.999999999999945','9.99999999999995','9.999999999999995','1.234567890123445','1.234567890123455']:
            neighborhood(float(decimal.Decimal(coefficient).scaleb(exponent)),'width_exponent_carry_and_tie_neighbors')
    for _ in range(100):
        head=rng.randrange(10**14,10**15)
        exponent=rng.choice([-300,-101,-100,-99,-98,-20,-10,-1,0,1,4,5,6,7,84,85,86,100,280])
        neighborhood(float((decimal.Decimal(head)+decimal.Decimal('0.5')).scaleb(exponent)),'fresh_decimal_midpoint_neighbors')
    for _ in range(800):
        add(from_bits(f'{rng.randrange(0x0010000000000000,0x7ff0000000000000)|(rng.randrange(2)<<63):016x}'),'fresh_uniform_normal_bits')
    rows=list(values.values())
    for i,row in enumerate(rows):
        n=g.n(row['value'])
        g.emit('LEFT',f'value-{i}',[n,g.n(32767)],axis='fresh_number_to_text')
        g.emit('LEN',f'length-{i}',[n],axis='fresh_number_to_text')
        if i%4==0:
            g.emit('EXACT',f'candidate-{i}',[n,g.t(format(row['value'],'.15g').replace('e','E'))],axis='fresh_equality_control')
        if i%11==0:
            for fn,args in [('LEFTB',[n,g.n(32767)]),('RIGHTB',[n,g.n(32767)]),('MIDB',[n,g.n(1),g.n(32767)]),('LENB',[n]),('CONCATENATE',[n]),('UNICODE',[n]),('CODE',[n]),('FIND',[g.t('E'),n]),('SEARCH',[g.t('E'),n]),('REPLACE',[n,g.n(1),g.n(0),g.t('')])]:
                g.emit(fn,f'consumer-{i}',args,axis='additional_formatter_consumer')
        if i%17==0:
            for fn,extra in [('LEFT',[g.n(32767)]),('CONCAT',[]),('EXACT',[g.t(format(row['value'],'.15g').replace('e','E'))])]:
                g.emit(fn,f'reference-{i}',[g.r('B200')]+extra,[g.fix('B200',n)],axis='number_reference_origin')
                g.emit(fn,f'array-{i}',[g.a([[n,n],[g.n(0),n]])]+extra,axis='number_array_origin')
        if i%29==0:
            g.emit('ADDRESS',f'address-control-{i}',[g.n(1),g.n(1),g.n(1),g.b(True),n],axis='address_formatter_binding')
    for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111numbertextholdout-')
    out=Path('smart-fuzzer/runs/w111-numeric-to-text-heldout-20260929');out.mkdir(parents=True,exist_ok=True)
    (out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
    sources=[Path('crates/oxfunc_core/src/functions')/f for f in ['numeric_text.rs','adapters.rs','reference_metadata_family.rs','round_fn.rs']]
    design={'seed':SEED,'numbers':len(rows),'rows':len(g.cases),'functions':dict(Counter(c['canonical_surface_name'] for c in g.cases)),'inputs':[{'bits':b,**r} for b,r in values.items()],'excluded_all_discovery_bits':True,'candidate_sha256':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in sources},'open_lanes':['unobservable nonfinite/subnormal carriers','locale/context','unexercised consumers','upstream acknowledgement']}
    (out/'design.json').write_text(json.dumps(design,indent=2),encoding='utf-8');print(len(rows),len(g.cases),design['functions'])
if __name__=='__main__':main()
