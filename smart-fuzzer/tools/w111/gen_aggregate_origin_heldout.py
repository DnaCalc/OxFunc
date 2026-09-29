"""Freeze three aggregate candidates, then generate fresh origin-order controls."""
import copy
import hashlib
import itertools
import json
import random
import shutil
from collections import Counter
from pathlib import Path

import gen_broad_typed_20260929 as g


def main():
    run = Path('smart-fuzzer/runs/w111-aggregate-origin-heldout-20260929')
    if (run / 'candidate-freeze.json').exists():
        raise SystemExit('Immutable freeze exists; choose a new run path.')
    run.mkdir(parents=True, exist_ok=True)
    sources = ['crates/oxfunc_core/src/functions/' + name for name in [
        'median_fn.rs', 'harmean_fn.rs', 'devsq_fn.rs', 'adapters.rs',
        'aggregate_common.rs', 'elementary_prepared.rs']]
    sources += ['formal/lean/OxFunc/ElementaryPrepared.lean',
                'formal/lean/OxFunc/AggregatePublication.lean',
                'formal/lean/OxFunc/Functions/MedianFn.lean',
                'formal/lean/OxFunc/Functions/HarMeanFn.lean',
                'formal/lean/OxFunc/Functions/DevSqFn.lean']
    hashes = {}
    for name in sources:
        source = Path(name)
        destination = run / 'candidate' / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
        hashes[name] = hashlib.sha256(source.read_bytes()).hexdigest()
    seed = 202609293039
    freeze = {'phase': 'independent_aggregate_origin_refinement', 'seed': seed,
              'source_sha256': hashes,
              'scope_completeness': 'scope_partial', 'target_completeness': 'target_partial',
              'integration_completeness': 'partial',
              'open_lanes': ['independent_origin_validation', 'contextual_numeric_text',
                             'HO_FN_030_receiving_acknowledgment']}
    (run / 'candidate-freeze.json').write_text(json.dumps(freeze, indent=2), encoding='utf-8')
    rng = random.Random(seed)
    g.TRANCHE = run.name
    origins = ['direct', 'unit_array', 'array', 'cell_reference', 'area_reference']
    errors = [g.e(code) for code in ['NA', 'Value', 'Div0', 'Num', 'Ref', 'Name', 'Null']]
    failing = errors + [g.t('x'), g.t(''), g.t('bad number')]
    shapes = [(1, 2), (1, 4), (2, 1), (3, 1), (2, 2), (2, 3), (3, 2)]

    def scalar(no_errors=False, in_collection=False):
        values = [g.n(rng.randrange(1, 2049) / 16), g.n(0), g.n(-2),
                  g.b(bool(rng.getrandbits(1))), g.t(str(rng.randrange(1, 100))),
                  g.t(''), g.t('x')]
        if not no_errors:
            values += errors
        if not in_collection:
            values += [g.missing(), g.blank()]
        return copy.deepcopy(rng.choice(values))

    def carried(value, origin, position, random_fill=False, no_errors=False):
        if origin == 'direct':
            return value, []
        if origin == 'unit_array':
            return g.a([[value]]), []
        row = 200 + position * 10
        if origin == 'cell_reference':
            target = f'B{row}'
            return g.r(target), [g.fix(target, value)]
        shape = rng.choice(shapes)
        cells = [scalar(no_errors, True) if random_fill else g.n(2 + i)
                 for i in range(shape[0] * shape[1])]
        cells[rng.randrange(len(cells))] = value
        rows = [cells[i * shape[1]:(i + 1) * shape[1]] for i in range(shape[0])]
        array = g.a(rows)
        if origin == 'array':
            return array, []
        target = f'B{row}:{chr(ord("B") + shape[1] - 1)}{row + shape[0] - 1}'
        if rng.getrandbits(1):
            fixtures = [g.fix(f'{chr(ord("B") + c)}{row + r}', rows[r][c])
                        for r in range(shape[0]) for c in range(shape[1])]
        else:
            fixtures = [g.fix(target, array)]
        return g.r(target), fixtures

    specifications = []
    for index, triple in enumerate(itertools.product(origins, repeat=3)):
        values = rng.sample(failing, 3)
        for reverse in [False, True]:
            pairings = list(zip(triple, values))
            if reverse:
                pairings.reverse()
            specifications.append((f'triple-{index}-reverse-{int(reverse)}', pairings, False, False))
    for index in range(384):
        pairings = []
        for _ in range(rng.randrange(3, 8)):
            origin = rng.choice(origins)
            pairings.append((origin, scalar(in_collection=origin != 'direct')))
        specifications.append((f'mixed-{index}', pairings, True, False))
    for index in range(96):
        pairings = [(rng.choice(origins), g.n(rng.randrange(1, 65537) / 16))
                    for _ in range(rng.randrange(3, 8))]
        specifications.append((f'value-order-{index}', pairings, True, True))
    # Materialize exactly the same argument/fixture packet for each family.
    for index, (tag, pairings, random_fill, no_errors) in enumerate(specifications):
        args, fixtures = [], []
        for position, (origin, value) in enumerate(pairings):
            arg, more = carried(copy.deepcopy(value), origin, position, random_fill, no_errors)
            args.append(arg)
            fixtures.extend(more)
        args.append(g.n(8))
        for function in ['MEDIAN', 'HARMEAN', 'DEVSQ']:
            g.emit(function, tag, args, fixtures,
                   axis='independent_cross_origin_precedence' if index < 634 else 'original_value_order')
    for case in g.cases:
        case['case_id'] = case['case_id'].replace('w111typed-', 'w111aggregateoriginheldout-')
    packet = {'schema_version': 'oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
              'tranche_id': g.TRANCHE, 'cases': g.cases,
              'tranches': [{'tranche_id': g.TRANCHE, 'case_ids': [c['case_id'] for c in g.cases]}]}
    (run / 'typed.json').write_text(json.dumps(packet, separators=(',', ':')), encoding='utf-8')
    design = {'seed': seed, 'rows': len(g.cases),
              'by_function': dict(Counter(c['canonical_surface_name'] for c in g.cases)),
              'candidate_frozen_before_generation': True,
              'all_origin_triples_and_reversals_per_function': 250,
              'random_mixed_origin_per_function': 384, 'value_order_controls_per_function': 96,
              'seven_error_codes': True, 'unit_arrays_preserve_collection_origin': True,
              'reference_fixture_forms': ['area', 'separate_cells'],
              'new_rule_target': 'Direct scalar coercion errors first; then original collection order.',
              'qualification': 'Public prepared-value observations; resolver failure before preparation and evaluator laziness are not claimed.'}
    (run / 'design.json').write_text(json.dumps(design, indent=2), encoding='utf-8')
    print(json.dumps(design, indent=2))


if __name__ == '__main__':
    main()
