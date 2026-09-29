"""Prepared-value discovery prompted by a read-only numerical repair review."""
import itertools
import json
from collections import Counter
from pathlib import Path

import gen_broad_typed_20260929 as g


def main():
    run = Path('smart-fuzzer/runs/w111-elementary-prepared-audit-20260929-002')
    if (run / 'design.json').exists():
        raise SystemExit('Existing audit packet is immutable; choose another path.')
    run.mkdir(parents=True, exist_ok=True)
    g.TRANCHE = run.name
    pool = [g.n(0), g.n(.5), g.n(-.5), g.n(2), g.b(True), g.b(False),
            g.t('0.5'), g.t('2'), g.t(''), g.t('x'), g.e('NA'),
            g.e('Div0'), g.e('Num'), g.blank(), g.missing()]
    defaults = {'LOG': [g.n(8), g.n(2)], 'POWER': [g.n(8), g.n(2)],
                'FISHER': [g.n(.5)], 'MEDIAN': [g.n(8), g.n(4)]}
    for function, ordinary in defaults.items():
        for position in range(len(ordinary)):
            for index, value in enumerate(pool):
                if function == 'FISHER' and value['kind'] == 'missing_arg':
                    # A lone missing slot renders as FISHER(), which the
                    # current literal/arity guard rejects before evaluation.
                    continue
                args = list(ordinary)
                args[position] = value
                g.emit(function, f'arg{position}-direct-{index}', args, axis='scalar_origin')
                if value['kind'] != 'missing_arg':
                    args[position] = g.r('B200')
                    g.emit(function, f'arg{position}-reference-{index}', args,
                           [g.fix('B200', value)], axis='reference_origin')
                if value['kind'] not in ['missing_arg', 'empty_cell']:
                    args[position] = g.a([[value, g.n(.5)]])
                    g.emit(function, f'arg{position}-array-{index}', args, axis='array_origin')
            for index, value in enumerate([g.n(.5), g.t('2'), g.b(True), g.e('Div0')]):
                args = list(ordinary)
                args[position] = g.a([[value]])
                g.emit(function, f'arg{position}-unit-array-{index}', args, axis='unit_array_origin')
            mixed = g.a([[g.n(.5), g.blank(), g.t('2')],
                         [g.b(True), g.e('Div0'), g.t('x')]])
            args = list(ordinary)
            args[position] = g.r('B200:D201')
            g.emit(function, f'arg{position}-mixed-reference-area', args,
                   [g.fix('B200:D201', mixed)], axis='mixed_reference_elements')
        if function == 'LOG':
            for index, value in enumerate(pool):
                if value['kind'] != 'missing_arg':
                    g.emit(function, f'omitted-base-{index}', [value], axis='omitted_default')
        if function in ['LOG', 'POWER']:
            failures = [g.e('NA'), g.e('Value'), g.e('Div0'), g.t('x'), g.missing()]
            for index, args in enumerate(itertools.product(failures, repeat=2)):
                g.emit(function, f'ordered-errors-{index}', list(args), axis='ordered_coercion')
            shapes = [g.a([[g.n(8), g.e('Div0'), g.t('x')]]),
                      g.a([[g.e('Value')], [g.n(2)]]),
                      g.a([[g.n(8), g.e('Num')], [g.e('Ref'), g.t('2')]]),
                      g.a([[g.n(2), g.n(4)]])]
            for index, args in enumerate(itertools.product(shapes, repeat=2)):
                g.emit(function, f'broadcast-{index}', list(args), axis='broadcast_padding_order')
        if function == 'MEDIAN':
            for index, row in enumerate([[g.t('2'), g.b(True), g.n(8)],
                                         [g.e('Div0'), g.e('NA'), g.n(8)],
                                         [g.t(''), g.t('x'), g.b(False)]]):
                g.emit(function, f'aggregate-direct-{index}', row, axis='aggregate_origin')
                g.emit(function, f'aggregate-array-{index}', [g.a([row])], axis='aggregate_origin')
                g.emit(function, f'aggregate-reference-{index}', [g.r('B200:D200')],
                       [g.fix('B200:D200', g.a([row]))], axis='aggregate_origin')
    for case in g.cases:
        case['case_id'] = case['case_id'].replace('w111typed-', 'w111elementaryprepared-')
    packet = {'schema_version': 'oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
              'tranche_id': g.TRANCHE, 'cases': g.cases,
              'tranches': [{'tranche_id': g.TRANCHE, 'case_ids': [c['case_id'] for c in g.cases]}]}
    (run / 'typed.json').write_text(json.dumps(packet, separators=(',', ':')), encoding='utf-8')
    design = {'phase': 'prepared_semantics_discovery', 'rows': len(g.cases),
              'by_function': dict(Counter(c['canonical_surface_name'] for c in g.cases)),
              'family_source_unchanged': True,
              'open_lanes': ['sole_missing_unary_arity_guard', 'receiving_metadata_acknowledgment'],
              'purpose': 'Test the remaining declarations and prepared semantics independently of repaired numeric arithmetic: origins, defaults, positional errors, LOG base arrays, binary padding and MEDIAN aggregates.'}
    (run / 'design.json').write_text(json.dumps(design, indent=2), encoding='utf-8')
    print(design)


if __name__ == '__main__':
    main()
