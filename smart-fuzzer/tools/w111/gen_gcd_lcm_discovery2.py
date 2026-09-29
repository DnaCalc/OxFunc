"""Focused integer bounds, aggregate origins and validation order; no Excel calls."""
import itertools,json,math,struct
from collections import Counter
from pathlib import Path
import gen_broad_typed_20260929 as g

def bits(n):return '0x'+struct.pack('>d',float(n)).hex()
def main():
    out=Path('smart-fuzzer/runs/w111-gcd-lcm-discovery2-20260929');out.mkdir(parents=True,exist_ok=True)
    (out/'batches').mkdir(exist_ok=True);numeric=[];limit=2**53
    for center in [0,1,2,2**31,2**32,limit]:
        for direction in [-math.inf,math.inf]:
            n=float(center)
            for _ in range(5):
                if n==0 or abs(n)>=2**-1022:
                    for other in [0,1,3,7,limit]:numeric.extend([(n,other),(other,n),(n,other,0),(0,n,other)])
                n=math.nextafter(n,direction)
    for total in range(limit-31,limit+32):
        for factor in range(2,100):
            if total%factor==0 and math.gcd(factor,total//factor)==1:
                a,b=factor,total//factor;numeric.extend([(a,b),(b,a),(a,b,0),(0,a,b)]);break
    for n in [-1e-300,-.5,-1,limit+2,1e100]:
        for a in [(n,0),(0,n),(n,1),(1,n),(n,0,1),(1,0,n)]:numeric.append(a)
    numeric=list(dict.fromkeys(numeric))
    for fn in ['GCD','LCM']:
        packet={'function':fn,'probes':[{'probe':{'id':f'w111gcdlcm2-{fn}-{i:05d}','args':list(map(bits,row))}} for i,row in enumerate(numeric)]}
        (out/'batches'/f'batch-{fn.lower()}.json').write_text(json.dumps(packet,separators=(',',':')),encoding='utf-8')
    g.TRANCHE=out.name
    pool=[('six',g.n(6)),('zero',g.n(0)),('negative',g.n(-.25)),('too-large',g.n(limit+2)),('numeric-text',g.t('2')),('bad-text',g.t('x')),('empty-text',g.t('')),('true',g.b(True)),('false',g.b(False)),('blank',g.blank()),('missing',g.missing())]+[(e,g.e(e)) for e in ['NA','Value','Div0','Num','Ref']]
    for fn in ['GCD','LCM']:
        for tag,value in pool:
            origins=[('direct',value,[])]
            if value['kind']!='missing_arg':
                origins += [('reference',g.r('B200'),[g.fix('B200',value)]),('range',g.r('B200:C200'),[g.fix('B200:C200',g.a([[value,g.n(6)]]))])]
                if value['kind']!='empty_cell':origins += [('array',g.a([[value,g.n(6)]]),[])]
            for origin,v,fixtures in origins:
                for pos,args in [('only',[v]),('first',[v,g.n(8)]),('last',[g.n(8),v]),('middle',[g.n(6),v,g.n(8)])]:
                    g.emit(fn,f'{tag}-{origin}-{pos}',args,fixtures,axis='aggregate_origin_and_missing')
        for left,right in itertools.product([g.n(-1),g.n(limit+2),g.n(0),g.t('x'),g.t(''),g.b(True),g.missing(),g.e('NA'),g.e('Div0')],repeat=2):
            g.emit(fn,f'order-{len(g.cases)}',[left,right],axis='coercion_domain_error_order')
        for values in [[g.missing(),g.missing()],[g.n(6),g.missing(),g.missing()],[g.missing(),g.n(6),g.missing()],[g.missing(),g.missing(),g.n(6)],[g.blank(),g.blank()],[g.t(''),g.t('')]]:
            g.emit(fn,f'empties-{len(g.cases)}',values,axis='empty_reduction_and_required_slot')
    for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111gcdlcm2-')
    (out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
    design={'numeric_per_function':len(numeric),'typed_rows':len(g.cases),'typed_functions':dict(Counter(c['canonical_surface_name'] for c in g.cases)),'purpose':'Inclusive2^53 input/result limits; floating vs exact LC M products; zero cancellation; requiredslot and aggregate origin/error precedence.'}
    (out/'design.json').write_text(json.dumps(design,indent=2),encoding='utf-8');print(design)
if __name__=='__main__':main()
