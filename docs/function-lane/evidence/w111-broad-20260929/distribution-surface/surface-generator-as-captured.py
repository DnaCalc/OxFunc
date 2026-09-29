"""Prepared NORM/EXPON argument origins, cumulative grammar and shape controls."""
import itertools,json,sys
from pathlib import Path
import gen_broad_typed_20260929 as g

g.TRANCHE='w111-distribution-surface-20260929'
specs={'NORM.S.DIST':[g.n(0),g.b(False)],'NORM.DIST':[g.n(0),g.n(0),g.n(1),g.b(False)],
       'NORMDIST':[g.n(0),g.n(0),g.n(1),g.b(False)],
       'EXPON.DIST':[g.n(0),g.n(1),g.b(False)],'EXPONDIST':[g.n(0),g.n(1),g.b(False)]}
texts=['TRUE','FALSE','true','fAlSe',' TRUE','TRUE ','\tTRUE','TRUE\u00a0','','0','1','-1','2.5','1e2','10%','bad']
flags=[g.t(s) for s in texts]+[g.n(n) for n in [0,1,-1,.125]]+[g.b(False),g.b(True)]+[g.e(e) for e in g.ERR]
numeric=[g.n(0),g.n(1),g.n(-1),g.t('0'),g.t('1'),g.t('TRUE'),g.t(''),g.b(False),g.b(True),g.missing()]+[g.e(e) for e in ['Ref','Num','Div0']]
for fn,args in specs.items():
    for value in flags:
        a=list(args);a[-1]=value;g.emit(fn,'cumulative-direct',a)
        a[-1]=g.r('B2');g.emit(fn,'cumulative-reference',a,[g.fix('B2',value)])
        a[-1]=g.a([[value,g.b(False)]]);g.emit(fn,'cumulative-row-array',a)
        a[-1]=g.r('B2:B3');g.emit(fn,'cumulative-reference-array',a,[g.fix('B2:B3',g.a([[value],[g.b(True)]]))])
    for axis in range(len(args)):
        values=numeric if axis<len(args)-1 else [g.missing()]
        for value in values:
            a=list(args);a[axis]=value;g.emit(fn,f'axis-{axis}-direct',a)
            if value['kind']!='missing':
                a[axis]=g.r('B2');g.emit(fn,f'axis-{axis}-reference',a,[g.fix('B2',value)])
        a=list(args);a[axis]=g.r('B2');g.emit(fn,f'axis-{axis}-blank-reference',a,[g.fix('B2',g.blank())])
    for axis in range(len(args)-1):
        for left,right in itertools.product([g.e('Ref'),g.e('Div0'),g.t('bad'),g.n(-1)],[g.e('Num'),g.t('bad'),g.t('1'),g.t('TRUE')]):
            a=list(args);a[axis]=left;a[-1]=right;g.emit(fn,f'precedence-axis-{axis}',a)
    for shape1,shape2 in [([[0,1]],[[False],[True]]),([[0,1],[1,0]],[[False],[True],[False]])]:
        a=list(args);a[0]=g.a([[g.n(n) for n in row] for row in shape1]);a[-1]=g.a([[g.b(n) for n in row] for row in shape2]);g.emit(fn,'shape-broadcast',a)
    a=list(args);a[0]=g.a([[g.t('0')]]);a[-1]=g.a([[g.t('TRUE')]]);g.emit(fn,'direct-unit-arrays',a)
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
 authority='non_semantic_prepared_argument_exploration',tranche_id=g.TRANCHE,
 cases=g.cases,tranches=[],summary=dict(case_count=len(g.cases),surfaces_covered=len(specs))),indent=2))
print(len(g.cases),out)
