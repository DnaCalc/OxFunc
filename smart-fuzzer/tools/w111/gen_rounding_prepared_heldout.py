"""Independent prepared-origin, Missing and positional-padding validation."""
import hashlib
import json
import random
import shutil
from collections import Counter
from pathlib import Path

import gen_broad_typed_20260929 as g


def main():
    seed = 202609293034
    rng = random.Random(seed)
    run = Path('smart-fuzzer/runs/w111-rounding-prepared-heldout-20260929')
    if (run / 'candidate-freeze.json').exists():
        raise SystemExit('An immutable freeze exists; use a new run path for another candidate.')
    run.mkdir(parents=True, exist_ok=True)
    g.TRANCHE = run.name
    pool = [g.b(True), g.b(False), g.t('3.125'), g.t('25%'), g.t('(1.5)'),
            g.t(''), g.t('bad'), g.e('Div0'), g.e('Ref'), g.e('NA'), g.e('Value')]

    def scalar(count=False):
        if rng.randrange(3):
            return g.n(rng.randrange(-40 if count else -8000, 41 if count else 8001) / rng.choice([1, 2, 4, 8]))
        return rng.choice(pool)

    for function in ['ROUND', 'ROUNDDOWN', 'ROUNDUP', 'TRUNC']:
        for index in range(224):
            args, fixtures = [], []
            for position in range(2):
                kind = rng.randrange(6)
                if kind == 0:
                    arg = g.missing()
                elif kind == 1:
                    arg = scalar(position == 1)
                elif kind == 2:
                    arg = g.blank()
                elif kind == 3:
                    target = 'B200' if position == 0 else 'G200'
                    arg = g.r(target)
                    fixtures.append(g.fix(target, scalar(position == 1)))
                else:
                    rows, cols = rng.choice([(1, 2), (1, 4), (2, 1), (3, 1), (2, 3), (3, 2)])
                    cells = [[scalar(position == 1) for _ in range(cols)] for _ in range(rows)]
                    if kind == 4:
                        arg = g.a(cells)
                    else:
                        first = 1 if position == 0 else 6
                        target = f'{chr(65+first)}200:{chr(65+first+cols-1)}{199+rows}'
                        if index % 3 == 0:
                            cells[rng.randrange(rows)][rng.randrange(cols)] = g.blank()
                        arg = g.r(target)
                        fixtures.append(g.fix(target, g.a(cells)))
                args.append(arg)
            g.emit(function, f'fresh-{index}', args, fixtures, axis='independent_prepared_origins_and_padding')
        if function == 'TRUNC':
            for index in range(32):
                arg = scalar() if index % 2 else g.a([[scalar(), scalar()], [scalar(), scalar()]])
                g.emit(function, f'fresh-omitted-{index}', [arg], axis='independent_optional_count')
    for case in g.cases:
        case['case_id'] = case['case_id'].replace('w111typed-', 'w111roundpreparedheldout-')
    packet = {'schema_version': 'oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
              'tranche_id': g.TRANCHE, 'cases': g.cases,
              'tranches': [{'tranche_id': g.TRANCHE, 'case_ids': [c['case_id'] for c in g.cases]}]}
    (run / 'typed.json').write_text(json.dumps(packet, separators=(',', ':')), encoding='utf-8')
    sources = ['crates/oxfunc_core/src/functions/' + p for p in
               ['round_fn.rs', 'rounddown_fn.rs', 'roundup_fn.rs', 'trunc_fn.rs']]
    sources += ['formal/lean/OxFunc/DecimalRounding.lean', 'formal/lean/OxFunc/Functions/Round.lean']
    hashes = {}
    for source in sources:
        path = Path(source)
        destination = run / 'candidate' / path
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, destination)
        hashes[source] = hashlib.sha256(path.read_bytes()).hexdigest()
    design = {'phase': 'independent_prepared_validation', 'seed': seed, 'rows': len(g.cases),
              'by_function': dict(Counter(c['canonical_surface_name'] for c in g.cases)),
              'source_sha256': hashes, 'open_lanes': ['initial15_precision', 'ROUNDUP_tiny_positive_residual',
                  'raw_overflow_numeric_publication_HO029', 'contextual_numeric_text_HO022', 'receiving_acknowledgement_HO028']}
    (run / 'candidate-freeze.json').write_text(json.dumps(design, indent=2), encoding='utf-8')
    print(len(g.cases), design['by_function'])


if __name__ == '__main__':
    main()
