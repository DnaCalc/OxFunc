"""Compact HARMEAN/DEVSQ aggregate origins, no-value and error-order probes."""
import itertools
import json
from collections import Counter
from pathlib import Path

import gen_broad_typed_20260929 as g


def main():
    run = Path('smart-fuzzer/runs/w111-aggregate-prepared-audit-20260929')
    if (run / 'design.json').exists():
        raise SystemExit('Immutable packet exists; use a new path.')
    run.mkdir(parents=True, exist_ok=True)
    g.TRANCHE = run.name
    pool = [g.n(0), g.n(-1), g.n(2), g.b(True), g.b(False), g.t('2'), g.t(''),
            g.t('x'), g.e('NA'), g.e('Div0'), g.e('Num'), g.blank(), g.missing()]
    for function in ['HARMEAN', 'DEVSQ']:
        for position in [0, 1]:
            for index, value in enumerate(pool):
                args = [g.n(4), g.n(8)]
                args[position] = value
                g.emit(function, f'arg{position}-direct-{index}', args, axis='aggregate_direct')
                if value['kind'] != 'missing_arg':
                    args[position] = g.r('B200')
                    g.emit(function, f'arg{position}-reference-{index}', args,
                           [g.fix('B200', value)], axis='aggregate_reference')
                if value['kind'] not in ['missing_arg', 'empty_cell']:
                    args[position] = g.a([[value, g.n(2)]])
                    g.emit(function, f'arg{position}-array-{index}', args, axis='aggregate_array')
        errors = [g.n(0), g.n(-1), g.t('x'), g.e('Num'), g.e('Div0'), g.missing()]
        for index, args in enumerate(itertools.product(errors, repeat=2)):
            g.emit(function, f'pair-order-{index}', list(args), axis='domain_coercion_error_order')
        for index, value in enumerate([g.n(2), g.b(True), g.t('2'), g.t('x'), g.e('Div0')]):
            g.emit(function, f'unit-array-{index}', [g.a([[value]])], axis='unit_array_origin')
        for index, row in enumerate([[g.t('2'), g.b(True), g.n(8)],
                                     [g.t(''), g.t('x'), g.b(False)],
                                     [g.e('Div0'), g.e('NA'), g.n(8)],
                                     [g.n(-1), g.e('Div0'), g.n(8)]]):
            g.emit(function, f'mixed-array-{index}', [g.a([row])], axis='array_order')
            g.emit(function, f'mixed-reference-{index}', [g.r('B200:D200')],
                   [g.fix('B200:D200', g.a([row]))], axis='reference_order')
        for index, cells in enumerate([[g.blank(), g.blank()], [g.blank(), g.n(8)],
                                       [g.blank(), g.e('Div0')]]):
            g.emit(function, f'blank-area-{index}', [g.r('B200:C200')],
                   [g.fix('B200:C200', g.a([cells]))], axis='blank_reference_aggregate')
    for case in g.cases:
        case['case_id'] = case['case_id'].replace('w111typed-', 'w111aggregateprepared-')
    packet = {'schema_version': 'oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
              'tranche_id': g.TRANCHE, 'cases': g.cases,
              'tranches': [{'tranche_id': g.TRANCHE, 'case_ids': [c['case_id'] for c in g.cases]}]}
    (run / 'typed.json').write_text(json.dumps(packet, separators=(',', ':')), encoding='utf-8')
    design = {'phase': 'aggregate_prepared_discovery', 'rows': len(g.cases),
              'by_function': dict(Counter(c['canonical_surface_name'] for c in g.cases)),
              'no_production_change': True, 'purpose': 'Distinguish direct, array and reference origins; empty vs missing; no counted values; coercion, explicit-error and domain ordering.'}
    (run / 'design.json').write_text(json.dumps(design, indent=2), encoding='utf-8')
    print(design)


if __name__ == '__main__':
    main()
