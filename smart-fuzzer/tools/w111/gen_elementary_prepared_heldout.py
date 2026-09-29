"""Freeze the elementary prepared candidate, then generate independent probes."""
import hashlib
import json
import random
import shutil
import struct
from collections import Counter
from pathlib import Path

import gen_broad_typed_20260929 as g


def bits(number):
    return '0x' + struct.pack('>d', float(number)).hex()


def main():
    run = Path('smart-fuzzer/runs/w111-elementary-prepared-heldout-20260929')
    if (run / 'candidate-freeze.json').exists():
        raise SystemExit('Immutable candidate freeze exists; use a new run path.')
    run.mkdir(parents=True, exist_ok=True)
    sources = [
        'crates/oxfunc_core/src/functions/' + name for name in [
            'elementary_prepared.rs', 'log_fn.rs', 'log10_fn.rs', 'power_fn.rs', 'median_fn.rs',
            'fisher_fn.rs', 'surface_dispatch_by_index_generated.rs',
            'surface_dispatch_unary_numeric_spec_generator.rs']]
    sources += ['formal/lean/OxFunc/ElementaryPrepared.lean',
                'formal/lean/OxFunc/ElementaryPublication.lean',
                'formal/lean/OxFunc/Functions/LogFn.lean',
                'formal/lean/OxFunc/Functions/PowerFn.lean',
                'formal/lean/OxFunc/Functions/MedianFn.lean',
                'crates/oxfunc_core/tests/fixtures/function_meta_golden.txt']
    hashes = {}
    for name in sources:
        source = Path(name)
        destination = run / 'candidate' / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
        hashes[name] = hashlib.sha256(source.read_bytes()).hexdigest()
    seed = 202609293038
    rng = random.Random(seed)
    g.TRANCHE = run.name
    freeze = {'phase': 'independent_after_elementary_prepared_refinement', 'seed': seed,
              'source_sha256': hashes, 'open_lanes': ['contextual_numeric_text',
                  'broader_numeric_primitive_alignment', 'receiving_HO_FN_030_acknowledgment']}
    (run / 'candidate-freeze.json').write_text(json.dumps(freeze, indent=2), encoding='utf-8')

    def scalar(allow_missing=True, allow_blank=True):
        values = [g.n(rng.randrange(-2000, 4001) / 16), g.n(0), g.n(1),
                  g.b(bool(rng.getrandbits(1))), g.t(str(rng.randrange(-99, 100) / 8)),
                  g.t('x'), g.t(''), g.e(rng.choice(['NA', 'Value', 'Num', 'Div0', 'Ref']))]
        if allow_missing:
            values.append(g.missing())
        if allow_blank:
            values.append(g.blank())
        return rng.choice(values)

    def array(shape, blanks=False):
        return g.a([[scalar(False, blanks) for _ in range(shape[1])] for _ in range(shape[0])])

    shapes = [(1, 1), (1, 2), (1, 4), (2, 1), (3, 1), (2, 2), (2, 3), (3, 2)]
    for function in ['LOG', 'POWER']:
        for index in range(96):
            g.emit(function, f'fresh-scalars-{index}', [scalar(), scalar()], axis='fresh_scalar_pairs')
        for index in range(96):
            g.emit(function, f'fresh-arrays-{index}', [array(rng.choice(shapes)), array(rng.choice(shapes))], axis='fresh_shape_pairs')
        for index in range(48):
            shape = rng.choice(shapes)
            end = chr(ord('B') + shape[1] - 1) + str(200 + shape[0] - 1)
            target = 'B200' if shape == (1, 1) else 'B200:' + end
            args = [scalar(), scalar()]
            args[index % 2] = g.r(target)
            g.emit(function, f'fresh-reference-{index}', args,
                   [g.fix(target, array(shape, True))], axis='fresh_reference_elements')
        if function == 'LOG':
            for index in range(64):
                value = scalar(False) if index % 2 else array(rng.choice(shapes))
                g.emit(function, f'fresh-omitted-{index}', [value], axis='fresh_optional_base')
    for index in range(192):
        args, fixtures = [], []
        for position in range(rng.randrange(2, 7)):
            origin = rng.randrange(3)
            if origin == 0:
                args.append(scalar())
            elif origin == 1:
                args.append(array(rng.choice(shapes)))
            else:
                target = f'B{200 + position}'
                args.append(g.r(target))
                fixtures.append(g.fix(target, scalar(False)))
        g.emit('MEDIAN', f'fresh-aggregate-{index}', args, fixtures, axis='fresh_aggregate_origins')
    for index in range(96):
        value = scalar(False) if index % 2 else array(rng.choice(shapes))
        g.emit('FISHER', f'fresh-control-{index}', [value], axis='unchanged_unary_controls')
    for case in g.cases:
        case['case_id'] = case['case_id'].replace('w111typed-', 'w111elementaryheldout-')
    packet = {'schema_version': 'oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
              'tranche_id': g.TRANCHE, 'cases': g.cases,
              'tranches': [{'tranche_id': g.TRANCHE, 'case_ids': [c['case_id'] for c in g.cases]}]}
    (run / 'typed.json').write_text(json.dumps(packet, separators=(',', ':')), encoding='utf-8')
    values = []
    for _ in range(1024):
        raw = (rng.randrange(1, 2047) << 52) | rng.getrandbits(52)
        values.append(struct.unpack('>d', raw.to_bytes(8, 'big'))[0])
    (run / 'batches').mkdir(exist_ok=True)
    for function in ['LOG', 'LOG10']:
        probes = []
        for index, number in enumerate(values):
            probes.append({'probe': {'id': f'w111logomissionheldout-{function}-{index:05d}-omitted', 'args': [bits(number)]}})
            if function == 'LOG':
                probes.append({'probe': {'id': f'w111logomissionheldout-{function}-{index:05d}-explicit', 'args': [bits(number), bits(10.)]}})
        (run / 'batches' / f'batch-{function.lower()}.json').write_text(json.dumps({'function': function, 'probes': probes}, separators=(',', ':')), encoding='utf-8')
    design = {'seed': seed, 'typed_rows': len(g.cases),
              'typed_functions': dict(Counter(c['canonical_surface_name'] for c in g.cases)),
              'numeric_rows': 3072, 'numeric_distinct_positive_normals': len(set(values)),
              'candidate_frozen_before_generation': True}
    (run / 'design.json').write_text(json.dumps(design, indent=2), encoding='utf-8')
    print(design)


if __name__ == '__main__':
    main()
