"""Distinguish reversal of arguments from reversal of elements within an array."""
import itertools,json
from pathlib import Path
import gen_broad_typed_20260929 as g

def main():
    g.TRANCHE='w111-lcm-array-order-20260929';big=3002399751580331
    triples=list(dict.fromkeys(itertools.chain.from_iterable(itertools.permutations(values) for values in [[3,3,big],[2,3,big],[3,big,big],[2**53,3,big]])))
    for i,nums in enumerate(triples):
        cells=list(map(g.n,nums))
        g.emit('LCM',f'direct-{i}',cells,axis='lcm_reduction_order')
        for name,matrix,target in [('row',[cells],'B200:D200'),('column',[[v] for v in cells],'B200:B202')]:
            g.emit('LCM',f'array-{name}-{i}',[g.a(matrix)],axis='lcm_reduction_order')
            g.emit('LCM',f'reference-{name}-{i}',[g.r(target)],[g.fix(target,g.a(matrix))],axis='lcm_reduction_order')
        for name,args in [('pair-first',[g.a([cells[:2]]),cells[2]]),('pair-last',[cells[0],g.a([cells[1:]])])]:
            g.emit('LCM',f'{name}-{i}',args,axis='lcm_reduction_order')
    for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111lcmorder-')
    out=Path('smart-fuzzer/runs')/g.TRANCHE;out.mkdir(parents=True,exist_ok=True)
    (out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8');print(len(g.cases))
if __name__=='__main__':main()
