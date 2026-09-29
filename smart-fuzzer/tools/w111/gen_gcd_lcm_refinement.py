"""Resolve rounded LCM accumulator behavior and scalar versus area blanks."""
import itertools,json,math,struct
from pathlib import Path
import gen_broad_typed_20260929 as g

def main():
    out=Path('smart-fuzzer/runs/w111-gcd-lcm-refinement-20260929');out.mkdir(parents=True,exist_ok=True);(out/'batches').mkdir(exist_ok=True)
    big=3002399751580331;rows=[]
    for tail in [0,1,2,3,5,7,big,2**53]:
        rows.extend(itertools.permutations([3,big,tail]))
    rows.extend([(3,big,3,big),(3,big,0,3,big),(3,big,1,1),(3,big,2,0),(3,big,0,2)])
    primes=[];n=1000000001
    while len(primes)<50:
        if all(n%d for d in range(3,math.isqrt(n)+1,2)):primes.append(n)
        n+=2
    for count in [5,20,35,40,50]:
        values=primes[:count];rows.extend([tuple(values),tuple([0]+values),tuple(values+[0]),tuple(values[:count//2]+[0]+values[count//2:])])
    rows=list(dict.fromkeys(rows))
    for fn in ['GCD','LCM']:
        (out/'batches'/f'batch-{fn.lower()}.json').write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'w111gcdlcmrefine-{fn}-{i:04d}','args':['0x'+struct.pack('>d',float(n)).hex() for n in values]}} for i,values in enumerate(rows)]},separators=(',',':')),encoding='utf-8')
    g.TRANCHE=out.name
    for fn in ['GCD','LCM']:
        for target,matrix in [('B200',[[g.blank()]]),('B200:B200',[[g.blank()]]),('B200:C200',[[g.blank(),g.blank()]]),('B200:B201',[[g.blank()],[g.blank()]]),('B200:C201',[[g.blank(),g.blank()],[g.blank(),g.blank()]])]:
            val=matrix[0][0] if ':' not in target else g.a(matrix)
            for extra in [[],[g.n(6)],[g.n(-1)],[g.e('Div0')]]:
                g.emit(fn,f'blank-shape-{len(g.cases)}',[g.r(target)]+extra,[g.fix(target,val)],axis='blank_reference_shape')
                if extra:g.emit(fn,f'blank-shape-reverse-{len(g.cases)}',extra+[g.r(target)],[g.fix(target,val)],axis='blank_reference_shape')
        for matrix in [[[g.n(-1),g.e('Div0')]],[[g.e('Div0'),g.n(-1)]],[[g.n(-1),g.t('x')]],[[g.n(-1),g.n(2)],[g.e('NA'),g.e('Div0')]],[[g.n(1),g.e('Div0')],[g.e('NA'),g.n(2)]]]:
            target='B200:C200' if len(matrix)==1 else 'B200:C201'
            for kind in ['array','reference']:
                arg=g.a(matrix) if kind=='array' else g.r(target);fixtures=[] if kind=='array' else [g.fix(target,g.a(matrix))]
                g.emit(fn,f'array-error-{len(g.cases)}',[arg],fixtures,axis='array_coercion_before_domain')
    for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111gcdlcmrefine-')
    (out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
    print('numeric perfn',len(rows),'typed',len(g.cases))
if __name__=='__main__':main()
