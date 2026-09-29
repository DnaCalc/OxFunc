"""Generic numeric-to-text black-box controls; no formatter is assumed shared."""
import json,math,random,struct,sys
from collections import Counter
from pathlib import Path
import gen_broad_typed_20260929 as g

SEED=202609292209

def to_bits(n):return struct.pack('>d',n).hex()
def from_bits(bits):return struct.unpack('>d',bytes.fromhex(bits.removeprefix('0x')))[0]

def main():
    g.TRANCHE='w111-numeric-to-text-discovery-20260929';rng=random.Random(SEED);values={}
    def add(n,origin):
        if math.isfinite(n) and (n==0 or abs(n)>=sys.float_info.min):
            values.setdefault(to_bits(n),{'value':n,'origin':origin})
    for n in [0.0,sys.float_info.min,math.nextafter(sys.float_info.min,math.inf),sys.float_info.max,math.nextafter(sys.float_info.max,0),1.2345678901234567,100000000000000.5,123456789012344.5,999999999999998.5]:
        add(n,'exact_boundary_or_decimal_tie');
        if n:add(-n,'exact_boundary_or_decimal_tie')
    for exponent in [-308,-307,-100,-99,-98,-22,-21,-20,-16,-15,-14,-12,-11,-10,-9,-8,-7,-6,-5,-4,-3,-2,-1,0,1,2,3,5,6,9,10,11,12,13,14,15,16,19,20,21,22,97,98,99,100,307,308]:
        for coefficient in [1,1.23456789012345,9.99999999999999]:
            try:n=coefficient*10.0**exponent
            except OverflowError:continue
            if not math.isfinite(n):continue
            for adjacent in [math.nextafter(n,0),n,math.nextafter(n,math.inf)]:
                add(adjacent,'exponent_boundary');add(-adjacent,'exponent_boundary')
    for path,index in [('smart-fuzzer/runs/w111-complex-scaling-heldout-20260929/batches/batch-complex.json',0),
                       ('smart-fuzzer/runs/w111-broad-20260929/address-numeric-format-discriminators.json',-1)]:
        d=json.loads(Path(path).read_text(encoding='utf-8-sig'));pool=[]
        for row in d['probes']:
            n=from_bits(row['probe']['args'][index])
            if n and n not in pool:pool.append(n)
        for n in rng.sample(pool,min(120,len(pool))):add(n,Path(path).name)
    for _ in range(400):
        bits=rng.randrange(0x0010000000000000,0x7ff0000000000000)|(rng.randrange(2)<<63)
        add(from_bits(f'{bits:016x}'),'fresh_uniform_normal_bits')
    rows=list(values.values())
    for i,row in enumerate(rows):
        n=row['value'];args=[g.n(n)]
        g.emit('LEFT',f'value-{i}',args+[g.n(32767)],axis='generic_numeric_text_exact_string')
        g.emit('LEN',f'length-{i}',args,axis='generic_numeric_text_length')
        candidate=format(n,'.15g').replace('e','E')
        g.emit('EXACT',f'candidate-15-{i}',args+[g.t(candidate)],axis='generic_numeric_text_candidate_equality')
        if i%7==0:
            g.emit('EXACT',f'candidate-shortest-{i}',args+[g.t(str(n).replace('e','E'))],axis='generic_numeric_text_shortest_control')
        if i%5==0:
            for fn,extra in [('RIGHT',[g.n(32767)]),('MID',[g.n(1),g.n(32767)]),('TRIM',[]),('CONCAT',[]),('ENCODEURL',[])]:
                g.emit(fn,f'cross-{i}',args+extra,axis='generic_numeric_text_cross_function')
        if i%11==0:
            for fn,extra in [('LOWER',[]),('UPPER',[]),('CLEAN',[]),('REPT',[g.n(1)]),('SUBSTITUTE',[g.t('x'),g.t('y')])]:
                g.emit(fn,f'additional-{i}',args+extra,axis='generic_numeric_text_additional_function')
            g.emit('TEXTJOIN',f'join-{i}',[g.t('|'),g.b(False),g.n(n)],axis='generic_numeric_text_additional_function')
        if i%17==0:
            g.emit('COMPLEX',f'formatter-control-{i}',[g.n(n),g.n(0)],axis='function_specific_formatter_control')
            g.emit('ADDRESS',f'formatter-control-{i}',[g.n(1),g.n(1),g.n(1),g.b(True),g.n(n)],axis='function_specific_formatter_control')
    for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111numbertext-')
    out=Path('smart-fuzzer/runs/w111-numeric-to-text-discovery-20260929');out.mkdir(parents=True,exist_ok=True)
    (out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
    design={'seed':SEED,'numbers':len(rows),'rows':len(g.cases),'inputs':[{'bits':b,**r} for b,r in values.items()],'functions':dict(Counter(c['canonical_surface_name'] for c in g.cases)),'exclusions':'No negative zero or subnormals: Value2 cannot preserve them.'}
    (out/'design.json').write_text(json.dumps(design,indent=2),encoding='utf-8');print(len(rows),len(g.cases),design['functions'])
if __name__=='__main__':main()
