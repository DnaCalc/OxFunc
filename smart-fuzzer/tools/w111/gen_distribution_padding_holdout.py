"""Fresh error/coercion precedence against missing array coordinates."""
import itertools,json,sys
from pathlib import Path
import gen_broad_typed_20260929 as g

g.TRANCHE='w111-distribution-padding-heldout-20260929'
specs={'NORM.S.DIST':[g.n(0),g.b(False)],'NORM.DIST':[g.n(0),g.n(0),g.n(1),g.b(False)],
 'NORMDIST':[g.n(0),g.n(0),g.n(1),g.b(False)],'EXPON.DIST':[g.n(0),g.n(1),g.b(False)],'EXPONDIST':[g.n(0),g.n(1),g.b(False)]}
for fn,args in specs.items():
    for left,right in itertools.permutations(range(len(args)),2):
        for value in [g.e('Value'),g.e('Name'),g.e('NA'),g.t('bad'),g.t('7'),g.n(-1)]:
            for orientation in ['column','row']:
                a=list(args)
                long=[[args[left]],[args[left]],[value]];short=[[args[right],args[right]],[args[right],args[right]]]
                if orientation=='row':long=[list(v) for v in zip(*long)]
                a[left]=g.a(long);a[right]=g.a(short);g.emit(fn,'fresh-padding-'+orientation,a)
                a[left]=value;g.emit(fn,'fresh-scalar-error-padding-'+orientation,a)
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',authority='independent_after_padding_order_refinement',
 generator=Path(__file__).name,tranche_id=g.TRANCHE,cases=g.cases,tranches=[],summary=dict(case_count=len(g.cases),surfaces_covered=len(specs))),indent=2))
print(len(g.cases),out)
