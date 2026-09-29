"""Independent prepared distribution contexts after the production freeze."""
import copy,itertools,json,random,sys
from pathlib import Path
import gen_broad_typed_20260929 as g

SEED=2026092931;rng=random.Random(SEED);g.TRANCHE='w111-distribution-prepared-heldout-20260929'
source=Path(__file__).resolve().parents[3]/'docs/function-lane/evidence/w111-broad-20260929/distribution-surface/adjacent-cases.json'
specs={c['canonical_surface_name']:c['args'] for c in json.loads(source.read_text())['cases'] if c['case_tag']=='base'}
specs.update({'NORM.S.DIST':[g.n(0),g.b(False)],'NORM.DIST':[g.n(0),g.n(0),g.n(1),g.b(False)],
 'NORMDIST':[g.n(0),g.n(0),g.n(1),g.b(False)],'EXPON.DIST':[g.n(0),g.n(1),g.b(False)],'EXPONDIST':[g.n(0),g.n(1),g.b(False)]})
for fn,args in specs.items():
    for axis,base in enumerate(args):
        if base['kind']=='logical':
            probes=[g.t(x) for x in ['trUE','FAlse','tRuE','FaLSE','\rTRUE','FALSE\n','\u202fTRUE','FALSE\u2003','00','+1','(1)','100%']]
        else:
            spelling=str(base['value']);probes=[g.t(' '+spelling+' '),g.t('\t'+spelling),g.t('('+spelling+')'),g.b(True),g.b(False)]
            if len(args)>1:probes.append(g.missing())
        probes.extend(g.e(code) for code in ['Name','Value','Div0'])
        for v in probes:
            a=copy.deepcopy(args);a[axis]=v;g.emit(fn,f'fresh-axis-{axis}-direct',a)
            if v['kind']!='missing_arg':
                a[axis]=g.r('F8');g.emit(fn,f'fresh-axis-{axis}-reference',a,[g.fix('F8',v)])
                a[axis]=g.a([[base],[v],[g.e('Ref')]]);g.emit(fn,f'fresh-axis-{axis}-array',a)
        a=copy.deepcopy(args);a[axis]=g.r('F8');g.emit(fn,f'fresh-axis-{axis}-blank',a,[g.fix('F8',g.blank())])
    if len(args)>1:
        for left,right in itertools.permutations(range(len(args)),2):
            a=copy.deepcopy(args);a[left]=g.a([[args[left]],[args[left]],[g.e('Name')]]);a[right]=g.a([[args[right],args[right]],[args[right],args[right]]]);g.emit(fn,'fresh-positional-padding',a)
    if fn=='BINOM.DIST.RANGE':
        g.emit(fn,'fresh-omitted-upper',args[:3])
        a=copy.deepcopy(args);a[3]=g.missing();g.emit(fn,'fresh-explicit-missing-upper',a)
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',authority='independent_after_production_distribution_freeze',
 generator=Path(__file__).name,seed=SEED,tranche_id=g.TRANCHE,cases=g.cases,tranches=[],summary=dict(case_count=len(g.cases),surfaces_covered=len(specs))),indent=2))
print(len(g.cases),out)
