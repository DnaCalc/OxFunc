"""Fresh typed/numeric text-function validation after the retained candidate."""
import copy,hashlib,json,math,random
from pathlib import Path
import gen_broad_typed_20260929 as g

SEED=202609292147

def main():
    rng=random.Random(SEED);g.TRANCHE='w111-text-slice-heldout-20260929'
    def text(midb=False):
        alphabet=['A','b','0',' ','\u00e9','\u0301','\u6f22','\ufe0f','%','/','\n']
        if not midb:alphabet+=['\U0001f600','\U0001f469','\U0001f4bb','\u200d']
        return g.t(''.join(rng.choice(alphabet) for _ in range(rng.randrange(0,30))))
    def source(midb=False):return rng.choice([text(midb),text(midb),g.n(rng.randrange(-9999,9999)),g.b(rng.choice([True,False])),g.e(rng.choice(list(g.ERR)))])
    def count():
        center=rng.randrange(1,130);near=center-(2**-22+2**-33)
        return rng.choice([g.n(rng.uniform(-5,90)),g.n(math.nextafter(near,rng.choice([-math.inf,math.inf]))),
            g.n(rng.choice([0,1,2,32767,2**31-1,2**31,1e30])),g.t(rng.choice(['2','200%','(2)','x',''])),g.b(rng.choice([True,False])),g.e(rng.choice(list(g.ERR))),g.missing()])
    def array(cell):
        h,w=rng.randrange(1,5),rng.randrange(1,5)
        rows=[]
        for _ in range(h):
            row=[]
            for _ in range(w):
                v=cell()
                if v['kind']=='missing_arg':v=g.n(0)
                row.append(v)
            rows.append(row)
        return g.a(rows)
    fns=['LEFT','RIGHT','LEFTB','RIGHTB','MID','MIDB','LEN','LENB','EXACT','ENCODEURL']
    for fn in fns:
        for i in range(180):
            make_source=lambda:source(fn=='MIDB')
            args=[array(make_source) if i%3 else make_source()]
            if fn=='EXACT':args.append(array(make_source) if i%2 else make_source())
            elif fn in ['MID','MIDB']:args += [array(count) if i%4 else count(),array(count) if i%5 else count()]
            elif fn in ['LEFT','RIGHT','LEFTB','RIGHTB'] and i%13:args += [array(count) if i%4 else count()]
            fixtures=[]
            if i%2:
                for pos,value in enumerate(args):
                    if value['kind']=='array':
                        rows=value['rows'];h,w=len(rows),len(rows[0]);col=chr(66+pos*4)
                        target=f'{col}1:{chr(ord(col)+w-1)}{h}' if h*w>1 else f'{col}1'
                        fixtures.append(g.fix(target,value if h*w>1 else rows[0][0]));args[pos]=g.r(target)
            g.emit(fn,f'fresh-{i}',args,fixtures,axis='text_independent_typed_broadcast')
    # Fresh integer centers and raw binary64 neighbors, with long enough text
    # to expose each changed count rather than merely clipping to a short text.
    source_text=''.join(chr(33+i%90) for i in range(192))
    for i in range(300):
        center=rng.randrange(1,180);distance=rng.choice([2**-22+2**-33,2**-22,1e-12,rng.random()])
        n=math.nextafter(center-distance,rng.choice([-math.inf,math.inf]));fn=rng.choice(fns[:6])
        args=[g.t(source_text),g.n(n)]
        if fn in ['MID','MIDB']:
            args=([g.t(source_text),g.n(n),g.n(3)] if i%2 else [g.t(source_text),g.n(2),g.n(n)])
        g.emit(fn,f'numeric-{i}',args,axis='text_independent_numeric_boundaries')
    for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111textholdout-')
    out=Path('smart-fuzzer/runs/w111-text-slice-heldout-20260929');out.mkdir(parents=True,exist_ok=True)
    packet=out/'typed.json';packet.write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
    paths=['crates/oxfunc_core/src/functions/'+x for x in ['text_slice_family.rs','text_b_compat_family.rs','exact_fn.rs','web_text_xml_family.rs','date_fn.rs']]
    report={'seed':SEED,'rows':len(g.cases),'phase':'before_fresh_oracle_capture','input_sha256':hashlib.sha256(packet.read_bytes()).hexdigest(),'source_sha256':{p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in paths}}
    Path('docs/function-lane/evidence/w111-broad-20260929/text-slice/candidate-freeze.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(len(g.cases),packet)
if __name__=='__main__':main()
