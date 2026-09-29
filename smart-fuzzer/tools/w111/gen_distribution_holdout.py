"""Fresh prepared distribution grammar, origin and padding/error interactions."""
import hashlib,itertools,json,random,sys
from pathlib import Path
import gen_broad_typed_20260929 as g

SEED=2026092925;rng=random.Random(SEED)
g.TRANCHE='w111-distribution-heldout-20260929'
specs={'NORM.S.DIST':[g.n(0),g.b(False)],'NORM.DIST':[g.n(0),g.n(0),g.n(1),g.b(False)],
 'NORMDIST':[g.n(0),g.n(0),g.n(1),g.b(False)],'EXPON.DIST':[g.n(0),g.n(1),g.b(False)],'EXPONDIST':[g.n(0),g.n(1),g.b(False)]}
flags=[]
for word in ['TRUE','FALSE']:
    variants=[''.join(chars) for chars in itertools.product(*[(c,c.lower()) for c in word])]
    rng.shuffle(variants);flags.extend(g.t(s) for s in variants[:12])
    for ws in ['\v','\f','\r','\n','\u2003','\u202f']:
        flags.extend([g.t(ws+word),g.t(word+ws)])
flags += [g.t(s) for s in ['00','+0','-0','1.0','0%','(0)','+TRUE','FALſE','ＴＲＵＥ','truefalse']]
flags += [g.n(n) for n in [.5,-.5,2,-2]]+[g.e(e) for e in g.ERR]
for fn,args in specs.items():
    for v in flags:
        a=list(args);a[-1]=v;g.emit(fn,'fresh-flag-direct',a)
        a[-1]=g.r('C4');g.emit(fn,'fresh-flag-reference',a,[g.fix('C4',v)])
        a[-1]=g.a([[g.b(True)],[v]]);g.emit(fn,'fresh-flag-column',a)
    for axis in range(len(args)-1):
        for v in [g.t('0'),g.t('100%'),g.t('(1)'),g.t('TRUE'),g.t(' 1 '),g.t('\t1'),g.b(False),g.missing()]:
            a=list(args);a[axis]=v;g.emit(fn,f'fresh-number-{axis}',a)
        for pair in itertools.permutations(['Ref','Div0','NA'],2):
            a=list(args);a[axis]=g.e(pair[0]);a[-1]=g.e(pair[1]);g.emit(fn,'fresh-error-order',a)
    g.emit(fn,'all-explicit-missing',[g.missing() for _ in args])
    # The first argument has a present error where a later argument is padded.
    for err in ['Ref','Div0','Num']:
        a=list(args);a[0]=g.a([[g.n(0)],[g.n(0)],[g.e(err)]]);a[-1]=g.a([[g.b(False),g.b(True)],[g.b(True),g.b(False)]]);g.emit(fn,'earlier-error-later-padding',a)
        a=list(args);a[0]=g.a([[g.n(0),g.n(0)],[g.n(0),g.n(0)]]);a[-1]=g.a([[g.b(False)],[g.b(True)],[g.e(err)]]);g.emit(fn,'earlier-padding-later-error',a)
        if len(args)>2:
            a=list(args);a[0]=g.e(err);a[1]=g.a([[g.n(1),g.n(1)],[g.n(1),g.n(1)]]);a[-1]=g.a([[g.b(False)],[g.b(True)],[g.b(False)]]);g.emit(fn,'scalar-error-later-padding',a)
    for _ in range(12):
        a=list(args);a[-1]=g.a([[rng.choice([g.t('trUE'),g.t('FALSE'),g.t('2'),g.e('NA'),g.b(False)]) for _ in range(rng.choice([1,2,3]))]])
        g.emit(fn,'fresh-flag-row',a)
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',authority='independent_after_prepared_distribution_candidate',
 generator=Path(__file__).name,seed=SEED,tranche_id=g.TRANCHE,cases=g.cases,tranches=[],summary=dict(case_count=len(g.cases),surfaces_covered=len(specs))),indent=2))
print(len(g.cases),out)
