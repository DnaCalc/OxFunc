"""Per-surface discovery for untouched normal and discrete prepared evaluators.

No family-level coercion rule is assumed. Numeric residuals remain separately
visible; this packet is intended to expose argument-type and shape behavior.
"""
import json,sys
from pathlib import Path
import gen_broad_typed_20260929 as g

g.TRANCHE='w111-distribution-adjacent-20260929'
specs={
 'CONFIDENCE':([.05,1,4],False),'CONFIDENCE.NORM':([.05,1,4],False),
 'NORM.INV':([.5,0,1],False),'NORMINV':([.5,0,1],False),
 'NORM.S.INV':([.5],False),'NORMSINV':([.5],False),'NORMSDIST':([0],False),
 'LOGNORM.INV':([.5,0,1],False),'LOGNORM.DIST':([1,0,1,False],True),'LOGNORMDIST':([1,0,1],False),
 'BINOM.DIST':([1,2,.5,False],True),'BINOMDIST':([1,2,.5,False],True),
 'BINOM.DIST.RANGE':([2,.5,1,1],False),'BINOM.INV':([2,.5,.5],False),'CRITBINOM':([2,.5,.5],False),
 'POISSON':([0,1,False],True),'POISSON.DIST':([0,1,False],True),
 'HYPGEOM.DIST':([1,2,2,4,False],True),'HYPGEOMDIST':([1,2,2,4],False),
 'NEGBINOM.DIST':([0,1,.5,False],True),'NEGBINOMDIST':([0,1,.5],False),
}
for fn,(values,flag) in specs.items():
    args=[g.b(v) if type(v)==bool else g.n(v) for v in values]
    g.emit(fn,'base',args)
    for axis,base in enumerate(args):
        probes=[g.b(False),g.b(True),g.t('0'),g.t('1'),g.t('50%'),g.t('TRUE'),g.t('FALSE'),g.t(''),g.missing(),g.e('Ref')]
        if flag and axis==len(args)-1:
            probes.extend(g.t(t) for t in ['TrUe','fAlSe',' TRUE','TRUE ','\tFALSE','FALSE\u00a0','-2','1e0'])
        for v in probes:
            a=list(args);a[axis]=v;g.emit(fn,f'axis-{axis}-direct',a)
            if v['kind']!='missing_arg':
                a[axis]=g.r('D6');g.emit(fn,f'axis-{axis}-reference',a,[g.fix('D6',v)])
        a=list(args);a[axis]=g.r('D6');g.emit(fn,f'axis-{axis}-blank',a,[g.fix('D6',g.blank())])
        a=list(args);a[axis]=g.a([[base,g.e('NA')]]);g.emit(fn,f'axis-{axis}-row-array',a)
        a[axis]=g.r('D6:D7');g.emit(fn,f'axis-{axis}-range',a,[g.fix('D6:D7',g.a([[base],[g.e('Ref')]]))])
    if len(args)>1:
        a=list(args);a[0]=g.a([[args[0]],[args[0]],[g.e('Ref')]]);a[-1]=g.a([[args[-1],args[-1]],[args[-1],args[-1]]]);g.emit(fn,'error-before-padding',a)
        a[0],a[-1]=g.a([[args[0],args[0]],[args[0],args[0]]]),g.a([[args[-1]],[args[-1]],[g.e('Num')]]);g.emit(fn,'padding-before-error',a)
    g.emit(fn,'all-missing',[g.missing() for _ in args])
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',authority='unmodified_adjacent_prepared_evaluator_discovery',
 generator=Path(__file__).name,tranche_id=g.TRANCHE,cases=g.cases,tranches=[],summary=dict(case_count=len(g.cases),surfaces_covered=len(specs))),indent=2))
print(len(g.cases),out)
