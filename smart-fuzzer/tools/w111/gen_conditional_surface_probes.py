"""IF/IFERROR/IFNA prepared-value coercion, selection and shape discriminators.

Arguments are already evaluated in local replay: these observations do not
establish evaluator short-circuit behavior or unselected expression effects.
"""
import itertools
import json
from pathlib import Path
import sys
import gen_broad_typed_20260929 as t

t.TRANCHE = 'w111-conditional-surface-20260929'
texts = ['TRUE', 'FALSE', 'true', 'false', 'TrUe', 'FaLsE', '', '0', '1', '-1',
         '2.5', '1e3', '10%', '(1)', 'TRUEFALSE', 'ＴＲＵＥ']
for word in ('TRUE', 'FALSE'):
    for space in (' ', '\t', '\n', '\u00a0'):
        texts += [space + word, word + space]
conditions = [t.t(x) for x in texts] + [t.n(x) for x in (0, 1, -1, 0.125)]
conditions += [t.b(False), t.b(True)] + [t.e(x) for x in t.ERR]
for value in conditions:
    for label, arg, fixtures in [
        ('direct', value, []),
        ('reference', t.r('A1'), [t.fix('A1', value)]),
        ('row', t.a([[value, t.b(True)]]), []),
        ('column', t.a([[t.b(False)], [value]]), []),
        ('reference-row', t.r('A1:B1'), [t.fix('A1:B1', t.a([[value, t.b(True)]]))]),
    ]:
        t.emit('IF', 'condition-' + label, [arg, t.n(7), t.n(9)], fixtures)

shape_conditions = [t.b(True), t.b(False), t.a([[t.b(True), t.b(False)]]),
                    t.a([[t.b(False)], [t.b(True)]]),
                    t.a([[t.b(True), t.b(False)], [t.b(False), t.e('NA')]])]
branches = [t.n(7), t.t('x'), t.e('Div0'),
            t.a([[t.n(11), t.n(12), t.n(13)]]),
            t.a([[t.n(21)], [t.n(22)]]),
            t.a([[t.n(31), t.n(32)], [t.n(33), t.n(34)]])]
for condition, yes, no in itertools.product(shape_conditions, branches, branches):
    t.emit('IF', 'shape', [condition, yes, no], axis='conditional_array_shape')
for condition in [t.b(True), t.b(False), t.missing(), t.blank(), t.t('TRUE')]:
    for yes, no in [(t.missing(),t.n(9)), (t.n(7),t.missing()),
                    (t.blank(),t.blank()), (t.t(''),t.t(''))]:
        t.emit('IF', 'missing-blank-branch', [condition,yes,no])
    t.emit('IF', 'omitted-false', [condition,t.n(7)])

primaries = [t.n(2), t.t(''), t.b(False), t.e('NA'), t.e('Div0'),
             t.a([[t.e('NA'),t.e('Value')]]),
             t.a([[t.n(1),t.e('NA')]]),
             t.a([[t.e('NA')],[t.n(2)]]),
             t.a([[t.n(1),t.e('NA')],[t.e('Div0'),t.t('x')]])]
fallbacks = [t.n(9),t.t('alt'),t.e('Num'),t.missing(),t.blank(),
             t.a([[t.n(11),t.n(12),t.n(13)]]),
             t.a([[t.n(21)],[t.n(22)],[t.n(23)]]),
             t.a([[t.n(31),t.n(32)],[t.n(33),t.n(34)]])]
for fn in ['IFERROR','IFNA']:
    for primary, fallback in itertools.product(primaries, fallbacks):
        t.emit(fn,'shape',[primary,fallback],axis='conditional_array_shape')
    for error, fallback in itertools.product(t.ERR,t.ERR):
        t.emit(fn,'error-pair',[t.e(error),t.e(fallback)])
    for primary in [t.missing(),t.blank(),t.t('TRUE'),t.t('2')]:
        for fallback in [t.n(9),t.missing(),t.blank()]:
            t.emit(fn,'blank-missing',[primary,fallback])
    for primary in primaries:
        if primary['kind']=='array':
            rows=primary['rows']; target=f"A1:{chr(64+len(rows[0]))}{len(rows)}"
        else:
            target='A1'
        t.emit(fn,'primary-reference',[t.r(target),t.n(9)],[t.fix(target,primary)])
    t.emit(fn,'blank-reference-array',[t.r('A1:C1'),t.n(9)],
           [t.fix('A1:C1',t.a([[t.blank(),t.e(),t.t('')]]))])

out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    authority='non_semantic_exploration_input',generator=Path(__file__).name,
    tranche_id=t.TRANCHE,comparison_policy='exact_typed_bit_match_no_tolerance',
    evaluation_scope='prepared_values_only_no_evaluator_laziness_claim',
    cases=t.cases,tranches=[],summary=dict(case_count=len(t.cases),surfaces_covered=3)),indent=2),encoding='utf-8')
print(len(t.cases),out)
