"""Uniform-selector shape controls supplement the initial conditional packet."""
import itertools,json
from pathlib import Path
import sys
import gen_broad_typed_20260929 as t
t.TRANCHE='w111-conditional-shape-controls-20260929'
branches=[t.n(7),t.a([[t.n(1),t.n(2),t.n(3)]]),t.a([[t.n(1)],[t.n(2)],[t.n(3)]]),
          t.a([[t.n(1),t.n(2)],[t.n(3),t.n(4)]])]
for cell in [t.b(True),t.b(False),t.e('Div0'),t.t('bad')]:
    condition=t.a([[cell,cell]])
    for yes,no in itertools.product(branches,branches):
        t.emit('IF','uniform-condition',[condition,yes,no])
for fn in ['IFERROR','IFNA']:
    for primary in [t.a([[t.n(1),t.n(2)]]),t.a([[t.e('Div0'),t.e('Div0')]]),
                    t.a([[t.e('NA'),t.e('NA')]]),t.a([[t.t(''),t.b(False)]])]:
        for fallback in branches:
            t.emit(fn,'uniform-primary',[primary,fallback])
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
 authority='non_semantic_exploration_input',generator=Path(__file__).name,tranche_id=t.TRANCHE,
 evaluation_scope='prepared_values_only_no_evaluator_laziness_claim',cases=t.cases,tranches=[],
 summary=dict(case_count=len(t.cases),surfaces_covered=3)),indent=2),encoding='utf-8')
print(len(t.cases),out)
