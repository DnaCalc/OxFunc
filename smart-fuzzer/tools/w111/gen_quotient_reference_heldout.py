"""Freeze QUOTIENT reference-origin candidate before independent controls."""
import copy
import hashlib
import json
import random
import shutil
from pathlib import Path

import gen_broad_typed_20260929 as g


def main():
    run = Path('smart-fuzzer/runs/w111-quotient-reference-heldout-20260929')
    if (run / 'candidate-freeze.json').exists():
        raise SystemExit('Immutable freeze already exists.')
    run.mkdir(parents=True, exist_ok=True)
    sources = ['crates/oxfunc_core/src/functions/quotient_fn.rs',
               'crates/oxfunc_core/src/functions/adapters.rs',
               'crates/oxfunc_core/src/functions/binary_numeric.rs',
               'crates/oxfunc_core/src/coercion.rs',
               'crates/oxfunc_core/src/coercion_decimal.rs',
               'formal/lean/OxFunc/Functions/QuotientFn.lean']
    hashes = {}
    for name in sources:
        destination = run / 'candidate' / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(name, destination)
        hashes[name] = hashlib.sha256(Path(name).read_bytes()).hexdigest()
    golden = next(line for line in Path('crates/oxfunc_core/tests/fixtures/function_meta_golden.txt')
                  .read_text().splitlines() if line.startswith('FUNC.QUOTIENT =>'))
    seed = 202609293040
    freeze = {'seed': seed, 'phase': 'independent_reference_origin_validation',
              'source_sha256': hashes, 'golden_row': golden,
              'golden_row_sha256': hashlib.sha256(golden.encode()).hexdigest(),
              'numeric_kernel_unchanged': True,
              'open_lanes': ['reference_origin_independent_validation',
                             'multi_area_and_unresolved_references', 'broader_positional_array_padding',
                             'contextual_numeric_text', 'HO_FN_027_receiving_acknowledgment']}
    (run / 'candidate-freeze.json').write_text(json.dumps(freeze, indent=2), encoding='utf-8')
    rng = random.Random(seed)
    g.TRANCHE = run.name
    errors = [g.e(x) for x in ['NA', 'Value', 'Div0', 'Num', 'Ref', 'Name', 'Null']]
    shapes = [(1, 2), (1, 3), (1, 5), (2, 1), (3, 1), (4, 1), (2, 2), (2, 3), (3, 2)]

    def scalar(blank=False):
        values = [g.n(rng.randrange(-99999, 100000) / 32), g.n(0), g.n(2),
                  g.b(bool(rng.getrandbits(1))), g.t(''), g.t('bad'),
                  g.t(str(rng.randrange(-200, 201)))] + errors
        if blank:
            values.append(g.blank())
        return copy.deepcopy(rng.choice(values))

    def reference(rows, position):
        row = 210 + 10 * position
        target = f'B{row}' if (len(rows), len(rows[0])) == (1, 1) else (
            f'B{row}:{chr(ord("B") + len(rows[0]) - 1)}{row + len(rows) - 1}')
        return g.r(target), [g.fix(target, g.a(rows))]

    for index in range(128):
        shape = rng.choice(shapes)
        rows = [[scalar() for _ in range(shape[1])] for _ in range(shape[0])]
        axis = index % 2
        other = scalar() if index % 4 else g.a([[scalar(), scalar()]])
        args = [other, other]
        args[axis], fixtures = reference(rows, axis)
        g.emit('QUOTIENT', f'fresh-reference-{index}', args, fixtures, axis='reference_origin')
        g.cases[-1]['formula_cell'] = rng.choice(['J20', 'J210', 'J211', 'B20', 'C20'])
        args[axis] = g.a(rows)
        g.emit('QUOTIENT', f'fresh-materialized-{index}', args, axis='paired_materialized_array')
    for index in range(128):
        axis = index % 2
        args = [scalar(), scalar()]
        value = scalar(True)
        args[axis], fixtures = reference([[value]], axis)
        if index % 4 == 0:
            args[1 - axis] = g.a([[scalar(), scalar()], [scalar(), scalar()]])
        g.emit('QUOTIENT', f'fresh-unit-reference-{index}', args, fixtures, axis='unit_reference')
    for index in range(128):
        axis = index % 2
        shape = rng.choice(shapes)
        rows = [[scalar(True) for _ in range(shape[1])] for _ in range(shape[0])]
        args = [copy.deepcopy(rng.choice(errors + [g.missing(), g.t('x')])), g.n(2)]
        args[1 - axis] = copy.deepcopy(rng.choice(errors + [g.missing(), g.t('x')]))
        args[axis], fixtures = reference(rows, axis)
        if index % 4 == 0:
            other_shape = rng.choice(shapes)
            other_rows = [[scalar() for _ in range(other_shape[1])] for _ in range(other_shape[0])]
            args[1 - axis], more = reference(other_rows, 1 - axis)
            fixtures += more
        g.emit('QUOTIENT', f'fresh-competing-origin-{index}', args, fixtures, axis='ordered_reference_error')
    for case in g.cases:
        case['case_id'] = case['case_id'].replace('w111typed-', 'w111quotientreferenceheldout-')
    packet = {'schema_version': 'oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
              'tranche_id': g.TRANCHE, 'cases': g.cases,
              'tranches': [{'tranche_id': g.TRANCHE, 'case_ids': [c['case_id'] for c in g.cases]}]}
    (run / 'typed.json').write_text(json.dumps(packet, separators=(',', ':')), encoding='utf-8')
    design = {'seed': seed, 'rows': len(g.cases), 'candidate_frozen_before_generation': True,
              'paired_reference_and_materialized_rows': 256, 'unit_reference_rows': 128,
              'competing_error_origin_rows': 128, 'scope': 'Finite prepared values, direct arrays and single rectangular references; caller positions varied.'}
    (run / 'design.json').write_text(json.dumps(design, indent=2), encoding='utf-8')
    print(json.dumps(design, indent=2))


if __name__ == '__main__':
    main()
