"""Separate defaults/coercion, broadcast, and BMP text discovery controls.

Supplementary UTF-16 slicing is captured separately without JSON string loss.
No outcomes are inferred by this generator.
"""
import copy,json,math,random
from collections import Counter
from pathlib import Path
import gen_broad_typed_20260929 as g

SEED=202609292101

def main():
    rng=random.Random(SEED);g.TRANCHE='w111-text-slice-discovery-20260929'
    counts=[g.missing(),g.blank(),g.n(0),g.n(1),g.n(2),g.n(-1),g.n(-0.5),g.n(-0.9999999999999999),
        g.n(math.nextafter(1,0)),g.n(math.nextafter(2,0)),g.n(1.9),g.n(2.9),g.n(32767),g.n(1e20),
        g.b(True),g.b(False),g.t(''),g.t(' '),g.t('2'),g.t(' 2 '),g.t('200%'),g.t('(2)'),g.t('x')]+[g.e(e) for e in g.ERR]
    sources=[g.t(''),g.t('Ab\u00e9\u6f22e\u0301'),g.n(123),g.b(True),g.blank(),g.missing(),g.e('NA'),g.e('Div0')]
    for fn in ['LEFT','RIGHT','LEFTB','RIGHTB']:
        for i,source in enumerate(sources):
            g.emit(fn,f'absent-{i}',[source],axis='text_absent_count')
            for j,count in enumerate(counts):
                g.emit(fn,f'default-{i}-{j}',[source,count],axis='text_count_kind_and_error_order')
        for i in range(24):
            left=rng.choice(list(g.ERR));right=rng.choice(list(g.ERR))
            g.emit(fn,f'errors-{i}',[g.e(left),g.e(right)],axis='text_explicit_error_order')
    for fn in ['MID','MIDB']:
        for i,start in enumerate(counts):
            for j,count in enumerate([g.missing(),g.n(0),g.n(-0.5),g.n(2),g.e('NA')]):
                g.emit(fn,f'position-{i}-{j}',[g.t('Ab\u00e9\u6f22e\u0301'),start,count],axis='text_mid_start_count')
    # Each shape pair has scalar, row, column, rectangular, mismatching and
    # reference forms. Array values include distinct errors for precedence.
    texts=[g.t('abc'),g.t(''),g.t('e\u0301'),g.b(False),g.n(123),g.e('NA'),g.e('Div0')]
    nums=[g.n(0),g.n(1),g.n(2),g.n(-0.5),g.t('2'),g.e('Num'),g.e('Ref')]
    shapes=[(1,1),(1,3),(3,1),(2,2),(2,3),(3,2)]
    def arr(pool,shape):return g.a([[copy.deepcopy(rng.choice(pool)) for _ in range(shape[1])] for _ in range(shape[0])])
    for fn in ['LEFT','RIGHT','LEFTB','RIGHTB','MID','MIDB','EXACT']:
        for i,shape1 in enumerate(shapes):
            for j,shape2 in enumerate(shapes):
                args=[arr(texts,shape1),arr(texts if fn=='EXACT' else nums,shape2)]
                if fn in ['MID','MIDB']:args.append(arr(nums,shapes[(i+j)%len(shapes)]))
                g.emit(fn,f'broadcast-{i}-{j}',args,axis='text_multiple_array_broadcast')
                if (i+j)%3==0:
                    fixtures=[]
                    for pos,arg in enumerate(args):
                        rows=arg['rows'];h,w=len(rows),len(rows[0]);col=chr(66+pos*4)
                        target=f'{col}1:{chr(ord(col)+w-1)}{h}' if h*w>1 else f'{col}1'
                        fixtures.append(g.fix(target,arg if h*w>1 else rows[0][0]));args[pos]=g.r(target)
                    g.emit(fn,f'ref-broadcast-{i}-{j}',args,fixtures,axis='text_reference_broadcast')
    for fn in ['LEN','LENB','ENCODEURL']:
        for i,v in enumerate(sources+[g.t('\U0001f600'),g.t('A\U0001f600B'),g.t('\u2764\ufe0f'),g.t('\U0001f469\u200d\U0001f4bb')]):
            g.emit(fn,f'scalar-{i}',[v],axis='text_unary_scalar_control')
        for i in range(48):
            value=arr(texts+([g.t('\U0001f600')] if fn!='ENCODEURL' else []),rng.choice(shapes))
            g.emit(fn,f'array-{i}',[value],axis='text_unary_array')
        for i in range(16):
            value=arr(texts+[g.blank()],(2,3))
            g.emit(fn,f'reference-{i}',[g.r('B1:D2')],[g.fix('B1:D2',value)],axis='text_unary_reference_array')
    for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111textdiscovery-')
    out=Path('smart-fuzzer/runs/w111-text-slice-discovery-20260929');out.mkdir(parents=True,exist_ok=True)
    (out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
    report={'seed':SEED,'rows':len(g.cases),'functions':dict(Counter(c['canonical_surface_name'] for c in g.cases)),'axes':dict(Counter(c['axis'] for c in g.cases))}
    (out/'design.json').write_text(json.dumps(report,indent=2),encoding='utf-8');print(json.dumps(report))

if __name__=='__main__':main()
