"""Fresh unit-array/reference and positional-padding metadata validation."""
import itertools,json,sys
from pathlib import Path
import gen_broad_typed_20260929 as g

g.TRANCHE='w111-distribution-unit-heldout-20260929'
source=Path(__file__).resolve().parents[3]/'docs/function-lane/evidence/w111-broad-20260929/distribution-surface/adjacent-cases.json'
specs={c['canonical_surface_name']:c['args'] for c in json.loads(source.read_text())['cases'] if c['case_tag']=='base' and len(c['args'])>=3}
specs.update({'NORM.DIST':[g.n(0),g.n(0),g.n(1),g.b(False)],'NORMDIST':[g.n(0),g.n(0),g.n(1),g.b(False)],'EXPON.DIST':[g.n(0),g.n(1),g.b(False)],'EXPONDIST':[g.n(0),g.n(1),g.b(False)]})
for fn,args in specs.items():
    for unit_axis in range(1,len(args)-1):
        for error,reverse,orientation,reference in itertools.product(['Value','NA','Null'],[False,True],['row','column'],[False,True]):
            left,right=(len(args)-1,0) if reverse else (0,len(args)-1)
            long=[[args[left]],[args[left]],[g.e(error)]]
            if orientation=='row':long=[list(v) for v in zip(*long)]
            a=list(args);a[left]=g.a(long);a[right]=g.a([[args[right],args[right]],[args[right],args[right]]])
            fixture=[]
            if reference:a[unit_axis]=g.r('G7');fixture=[g.fix('G7',args[unit_axis])]
            else:a[unit_axis]=g.a([[args[unit_axis]]])
            g.emit(fn,f'fresh-unit-{unit_axis}-padding-{orientation}',a,fixture)
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',authority='independent_after_legacy_native_lifting_metadata_correction',
 generator=Path(__file__).name,tranche_id=g.TRANCHE,cases=g.cases,tranches=[],summary=dict(case_count=len(g.cases),surfaces_covered=len(specs))),indent=2))
print(len(g.cases),out)
