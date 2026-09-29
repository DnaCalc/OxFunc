"""Distinguish direct scalar errors from array/reference aggregate errors."""
import itertools
import json
from collections import Counter
from pathlib import Path

import gen_broad_typed_20260929 as g


def main():
    run = Path('smart-fuzzer/runs/w111-aggregate-error-origin-20260929')
    if (run / 'design.json').exists():
        raise SystemExit('Immutable discovery packet exists; choose another path.')
    run.mkdir(parents=True, exist_ok=True)
    g.TRANCHE = run.name
    origins = ['direct', 'unit_array', 'array', 'cell_reference', 'area_reference']
    values = [g.e('Div0'), g.e('Num'), g.t('x')]
    def carried(value, origin, position):
        if origin == 'direct':
            return value, []
        if origin == 'unit_array':
            return g.a([[value]]), []
        if origin == 'array':
            return g.a([[value, g.n(2)]]), []
        row = 200 + 10 * position
        if origin == 'cell_reference':
            target = f'B{row}'
            return g.r(target), [g.fix(target, value)]
        target = f'B{row}:C{row}'
        return g.r(target), [g.fix(target, g.a([[value, g.n(2)]]))]
    choices = list(itertools.product(origins, range(len(values))))
    for function in ['MEDIAN', 'HARMEAN', 'DEVSQ']:
        for index, (left, right) in enumerate(itertools.product(choices, repeat=2)):
            lhs, lf = carried(values[left[1]], left[0], 0)
            rhs, rf = carried(values[right[1]], right[0], 1)
            g.emit(function, f'origin-order-{index}-{left[0]}-{right[0]}',
                   [lhs, rhs, g.n(8)], lf + rf, axis='direct_vs_collection_error_order')
    for case in g.cases:
        case['case_id'] = case['case_id'].replace('w111typed-', 'w111aggregateerrororigin-')
    packet = {'schema_version': 'oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
              'tranche_id': g.TRANCHE, 'cases': g.cases,
              'tranches': [{'tranche_id': g.TRANCHE, 'case_ids': [c['case_id'] for c in g.cases]}]}
    (run / 'typed.json').write_text(json.dumps(packet, separators=(',', ':')), encoding='utf-8')
    design = {'phase': 'aggregate_error_origin_discovery', 'rows': len(g.cases),
              'by_function': dict(Counter(c['canonical_surface_name'] for c in g.cases)),
              'no_new_production_rule': True,
              'purpose': 'Distinguish left-to-right flattened errors from an earlier direct-scalar coercion pass, including unit arrays and both reference extents; compare three aggregate functions without presuming a shared rule.'}
    (run / 'design.json').write_text(json.dumps(design, indent=2), encoding='utf-8')
    print(design)


if __name__ == '__main__':
    main()
