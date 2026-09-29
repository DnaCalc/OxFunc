"""Freeze the QUOTIENT positional refinement before fresh asymmetric controls."""
import hashlib
import json
import random
import shutil
from pathlib import Path

import gen_broad_typed_20260929 as g


def main():
    run = Path('smart-fuzzer/runs/w111-quotient-padding-heldout-20260929')
    if (run / 'candidate-freeze.json').exists():
        raise SystemExit('Immutable freeze already exists.')
    run.mkdir(parents=True, exist_ok=True)
    sources = ['crates/oxfunc_core/src/functions/' + name for name in [
        'quotient_fn.rs', 'adapters.rs', 'binary_numeric.rs', 'elementary_prepared.rs']]
    sources += ['crates/oxfunc_core/src/coercion.rs',
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
    seed = 202609293041
    freeze = {'seed': seed, 'phase': 'independent_after_positional_refinement',
              'source_sha256': hashes, 'golden_row': golden,
              'golden_row_sha256': hashlib.sha256(golden.encode()).hexdigest(),
              'previous_failed_freeze': 'w111-quotient-reference-heldout-20260929/candidate-freeze.json',
              'previous_failed_rows': 5,
              'open_lanes': ['independent_positional_validation', 'contextual_numeric_text',
                             'multi_area_and_unresolved_references', 'HO_FN_027_receiving_acknowledgment']}
    (run / 'candidate-freeze.json').write_text(json.dumps(freeze, indent=2), encoding='utf-8')
    rng = random.Random(seed)
    g.TRANCHE = run.name
    errors = ['NA', 'Value', 'Div0', 'Num', 'Ref', 'Name', 'Null']

    def scalar(blank=False):
        kind = rng.randrange(8)
        if kind < 3:
            return g.e(rng.choice(errors))
        if kind == 3:
            return g.b(bool(rng.getrandbits(1)))
        if kind == 4:
            return g.t(rng.choice(['x', '', ' 3 ', '(4)', '%2', '18.125']))
        if kind == 5:
            return g.n(0)
        if kind == 6 and blank:
            return g.blank()
        return g.n(rng.randrange(-65535, 65536) / 32)

    def array(shape, blank=False):
        return g.a([[scalar(blank) for _ in range(shape[1])] for _ in range(shape[0])])

    pairs = [((1, 5), (1, 2)), ((5, 1), (2, 1)), ((2, 5), (3, 2)),
             ((3, 2), (2, 3)), ((2, 4), (4, 2)), ((1, 3), (2, 2))]
    for pair_index, (left, right) in enumerate(pairs):
        for reverse in [False, True]:
            for index in range(40):
                args = [array(left), array(right)]
                if reverse:
                    args.reverse()
                g.emit('QUOTIENT', f'asymmetric-{pair_index}-{int(reverse)}-{index}', args,
                       axis='independent_positional_padding')
    shapes = [(1, 2), (1, 4), (2, 1), (3, 1), (2, 2), (2, 3), (3, 2)]
    for index in range(144):
        axis = index % 2
        shape = rng.choice(shapes) if index < 96 else (1, 1)
        value = array(shape, True)
        row = 220 + 10 * axis
        target = f'B{row}' if shape == (1, 1) else (
            f'B{row}:{chr(ord("B") + shape[1] - 1)}{row + shape[0] - 1}')
        args = [scalar(), scalar()]
        if index % 3 == 0:
            args[1 - axis] = array(rng.choice(shapes))
        args[axis] = g.r(target)
        g.emit('QUOTIENT', f'origin-control-{index}', args, [g.fix(target, value)],
               axis='preserved_reference_origin')
        g.cases[-1]['formula_cell'] = rng.choice(['J20', 'J220', 'J230', 'B20', 'C20'])
    for case in g.cases:
        case['case_id'] = case['case_id'].replace('w111typed-', 'w111quotientpaddingheldout-')
    packet = {'schema_version': 'oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
              'tranche_id': g.TRANCHE, 'cases': g.cases,
              'tranches': [{'tranche_id': g.TRANCHE, 'case_ids': [c['case_id'] for c in g.cases]}]}
    (run / 'typed.json').write_text(json.dumps(packet, separators=(',', ':')), encoding='utf-8')
    design = {'seed': seed, 'rows': len(g.cases), 'candidate_frozen_before_generation': True,
              'asymmetric_array_rows': 480, 'nonunit_reference_rows': 96,
              'unit_reference_rows': 48, 'both_argument_positions': True,
              'error_codes': errors}
    (run / 'design.json').write_text(json.dumps(design, indent=2), encoding='utf-8')
    print(json.dumps(design, indent=2))


if __name__ == '__main__':
    main()
