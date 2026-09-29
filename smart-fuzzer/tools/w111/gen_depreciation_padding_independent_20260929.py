"""Independent financial padding, error-order, and one-cell-array controls."""
import copy,hashlib,json
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/cache/w111-depreciation-20260929';g.TRANCHE='w111-depreciation-padding-independent-20260929'
specs={'SLN':[1375.,125.,9.75],'SYD':[1375.,125.,9.75,3.125], 'DB':[1375.,125.,9.75,3.125,4.5], 'DDB':[1375.,125.,9.75,3.125,1.5], 'VDB':[1375.,125.,9.75,1.125,3.375,1.5]}
for fn,values in specs.items():
 base=list(map(g.n,values))
 if fn=='VDB':base.append(g.b(True))
 n=len(base)
 for earlier,later in sorted({(0,n-1),(1,n-1),(n-2,n-1)}):
  unit=next(i for i in range(n) if i not in [earlier,later])
  for vertical in [False,True]:
   for tag,value in [('null',g.e('Null')),('name',g.e('Name')),('ref',g.e('Ref')),('badtext',g.t('invalid')),('domain',g.n(-.375))]:
    for reversed_shapes in [False,True]:
     for unit_array in [False,True]:
      args=copy.deepcopy(base)
      wide=[[base[earlier],base[earlier],value],[base[earlier],base[earlier],value]]
      narrow=[[base[later],base[later]],[base[later],base[later]]]
      if reversed_shapes:
       wide=[[base[later],base[later],value],[base[later],base[later],value]]
       narrow=[[base[earlier],base[earlier]],[base[earlier],base[earlier]]]
      if vertical:wide=list(map(list,zip(*wide)))
      args[earlier],args[later]=(g.a(narrow),g.a(wide)) if reversed_shapes else (g.a(wide),g.a(narrow))
      if unit_array:args[unit]=g.a([[base[unit]]])
      g.emit(fn,f'independent-padding-{earlier}-{later}-{vertical}-{tag}-{reversed_shapes}-{unit_array}',args,axis='array_lift')
 for pos in range(n):
  args=copy.deepcopy(base);args[pos]=g.a([[base[pos]]]);g.emit(fn,f'unit-only-{pos}',args,axis='array_lift')
for c in g.cases:c['case_id']=c['case_id'].replace('w111typed-','w111dep-padding-independent-')
path=out/'typed-depreciation-padding-independent.json';doc=dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',tranche_id=g.TRANCHE,cases=g.cases,tranches=[dict(tranche_id=g.TRANCHE,case_ids=[c['case_id'] for c in g.cases])])
path.write_text(json.dumps(doc,indent=2)+'\n',encoding='utf-8');path.with_suffix('.jsonl').write_text(''.join(json.dumps(c,ensure_ascii=False)+'\n' for c in g.cases),encoding='utf-8')
(out/'typed-depreciation-padding-independent-manifest.json').write_text(json.dumps(dict(cases=len(g.cases),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),family_source_sha256=hashlib.sha256((g.ROOT/'crates/oxfunc_core/src/functions/depreciation_family.rs').read_bytes()).hexdigest(),helper_source_sha256=hashlib.sha256((g.ROOT/'crates/oxfunc_core/src/functions/distribution_common.rs').read_bytes()).hexdigest(),scope='Independent financial positional-padding controls: both error orders, horizontal/vertical shape conflicts, optional final positions, one-cell-array vs scalar controls, domain-vs-coercion distinction.'),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),cases=len(g.cases))))
