"""Typed preparation and positional broadcast controls for the four rounders."""
import itertools
import json
from collections import Counter
from pathlib import Path

import gen_broad_typed_20260929 as g


def main():
    run = Path('smart-fuzzer/runs/w111-rounding-prepared-controls-20260929')
    run.mkdir(parents=True, exist_ok=True)
    g.TRANCHE = run.name
    pool = [g.n(1.25), g.n(-2.375), g.n(0), g.b(True), g.b(False),
            g.t('1.25'), g.t(' % 125 '), g.t('(2.5)'), g.t(''), g.t('x'),
            g.t('TRUE'), g.t(' 2 '), g.e('NA'), g.e('Value'), g.e('Div0'),
            g.e('Num'), g.blank(), g.missing()]
    for function in ['ROUND', 'ROUNDDOWN', 'ROUNDUP', 'TRUNC']:
        for position in range(2):
            for index, value in enumerate(pool):
                args = [g.n(12.375), g.n(1)]
                args[position] = value
                g.emit(function, f'arg{position}-direct-{index}', args, axis='typed_origins')
                if value['kind'] != 'missing_arg':
                    args[position] = g.r('B200')
                    g.emit(function, f'arg{position}-reference-{index}', args,
                           [g.fix('B200', value)], axis='typed_origins')
                if value['kind'] not in ['missing_arg', 'empty_cell']:
                    args[position] = g.a([[value, g.n(1.25)]])
                    g.emit(function, f'arg{position}-array-{index}', args, axis='typed_origins')
        # Both coercion order and padding's argument position are visible.
        errors = [g.e('NA'), g.e('Value'), g.e('Div0'), g.e('Num'), g.t('x'), g.missing()]
        for index, (left, right) in enumerate(itertools.product(errors, repeat=2)):
            g.emit(function, f'pair-order-{index}', [left, right], axis='error_precedence')
        shapes = [
            g.a([[g.n(1.25), g.e('Div0'), g.t('x')]]),
            g.a([[g.e('Value')], [g.n(2.5)]]),
            g.a([[g.n(1.25), g.e('Num')], [g.e('Ref'), g.t('2.5')]]),
            g.a([[g.n(1.25), g.n(2.5)]]),
            g.a([[g.n(1)], [g.n(2)], [g.e('NA')]]),
        ]
        for index, (left, right) in enumerate(itertools.product(shapes, repeat=2)):
            g.emit(function, f'array-shape-{index}', [left, right], axis='broadcast_padding_order')
        mixed = g.a([[g.n(1.25), g.blank(), g.t('2.5')], [g.b(True), g.e('Div0'), g.t('')]])
        for position in range(2):
            args = [g.n(12.375), g.n(1)]
            args[position] = g.r('B200:D201')
            g.emit(function, f'reference-mixed-{position}', args,
                   [g.fix('B200:D201', mixed)], axis='reference_array')
            args[1 - position] = shapes[0]
            g.emit(function, f'reference-array-cross-{position}', args,
                   [g.fix('B200:D201', mixed)], axis='reference_array')
        if function == 'TRUNC':
            for index, value in enumerate(pool):
                g.emit(function, f'omitted-count-{index}', [value], axis='optional_count')
            for index, value in enumerate(shapes):
                g.emit(function, f'omitted-array-count-{index}', [value], axis='optional_count')
    for case in g.cases:
        case['case_id'] = case['case_id'].replace('w111typed-', 'w111roundprepared-')
    packet = {'schema_version': 'oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
              'tranche_id': g.TRANCHE, 'cases': g.cases,
              'tranches': [{'tranche_id': g.TRANCHE, 'case_ids': [c['case_id'] for c in g.cases]}]}
    (run / 'typed.json').write_text(json.dumps(packet, separators=(',', ':')), encoding='utf-8')
    design = {'phase': 'prepared_semantics_discovery', 'rows': len(g.cases),
              'by_function': dict(Counter(c['canonical_surface_name'] for c in g.cases)),
              'no_new_runtime_policy': True,
              'purpose': 'Distinguish required missing/optional omission, typed origins, positional coercion errors and singleton/mismatched-shape array lifting before evaluator declaration assessment.'}
    (run / 'design.json').write_text(json.dumps(design, indent=2), encoding='utf-8')
    print(design)


if __name__ == '__main__':
    main()
