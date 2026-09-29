"""Independent count-family origin validation after candidate freeze."""
import hashlib,json,random
from pathlib import Path
import gen_broad_typed_20260929 as g

SEED=202609292011
def main():
    rng=random.Random(SEED);g.TRANCHE='w111-count-family-heldout-20260929'
    def value(blank=True):
        return rng.choice([g.n(rng.randrange(-100000,100001)),g.n(0),g.b(rng.choice([True,False])),
            g.t(rng.choice(['',' ','x','TRUE','false','2',' 2 ','(2)','200%','% - 200','-.5',str(rng.randrange(10000))])),
            g.e(rng.choice(list(g.ERR)))]+([g.blank()] if blank else []))
    for fn in ['COUNT','COUNTA','COUNTBLANK']:
        for i in range(400):
            height,width=rng.randrange(1,8),rng.randrange(1,6)
            cells=[[value() for _ in range(width)] for _ in range(height)]
            target=f'B2:{chr(66+width-1)}{height+1}' if height*width>1 else 'B2'
            fixture=g.a(cells) if height*width>1 else cells[0][0]
            g.emit(fn,f'range-{i}',[g.r(target)],[g.fix(target,fixture)],axis='count_family_independent_ranges')
    for fn in ['COUNT','COUNTA']:
        for i in range(160):
            args=[];fixtures=[]
            for j in range(rng.randrange(2,11)):
                v=rng.choice([value(),g.missing()])
                if v['kind']=='empty_cell' or (v['kind']!='missing_arg' and rng.choice([True,False])):
                    target=f'B{j+1}';fixtures.append(g.fix(target,v));v=g.r(target)
                args.append(v)
            g.emit(fn,f'fold-{i}',args,fixtures,axis='count_family_independent_mixed_origins')
    for i in range(96):
        cells=[[value(False) for _ in range(rng.randrange(2,6))]]
        g.emit('COUNTBLANK',f'array-{i}',[g.a(cells)],axis='countblank_independent_array_errors')
    for i in range(48):
        v=value(False)
        g.emit('COUNTBLANK',f'computed-{i}',[v],axis='countblank_independent_computed_scalar')
        case=g.cases[-1]
        body=case['formula_text'][len('=COUNTBLANK('):-1]
        case['formula_text']=f'=COUNTBLANK(IF(FALSE,{body},{body}))'
    for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111countholdout-')
    out=Path('smart-fuzzer/runs/w111-count-family-heldout-20260929');out.mkdir(parents=True,exist_ok=True)
    packet=out/'typed.json';packet.write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
    paths=['crates/oxfunc_core/src/functions/aggregate_common.rs','crates/oxfunc_core/src/functions/count.rs','crates/oxfunc_core/src/functions/counta.rs','crates/oxfunc_core/src/functions/countblank_fn.rs',__file__]
    freeze={'seed':SEED,'rows':len(g.cases),'phase':'before_independent_oracle_capture','source_sha256':{p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in paths},'input_sha256':hashlib.sha256(packet.read_bytes()).hexdigest()}
    Path('docs/function-lane/evidence/w111-broad-20260929/count-family/candidate-freeze.json').write_text(json.dumps(freeze,indent=2),encoding='utf-8')
    print(len(g.cases),packet)

if __name__=='__main__':main()
