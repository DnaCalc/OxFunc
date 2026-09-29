"""Classify the retained aggregate error-origin packet before production edits."""
import json
import shutil
from collections import Counter
from pathlib import Path


def main():
    run = Path('smart-fuzzer/runs/w111-aggregate-error-origin-typed-20260929')
    out = Path('docs/function-lane/evidence/w111-broad-20260929/elementary-publication/aggregate-error-origin')
    out.mkdir(parents=True, exist_ok=True)
    for filename in ['cases/cases.jsonl', 'outcomes/excel.jsonl', 'outcomes/local.jsonl', 'manifest.json', 'rollup.json']:
        shutil.copyfile(run / filename, out / filename.replace('/', '-'))
    cases = [json.loads(line) for line in (run/'cases/cases.jsonl').read_text(encoding='utf-8-sig').splitlines()]
    answers = {r['case_id']: r for r in map(json.loads, (run/'outcomes/excel.jsonl').read_text(encoding='utf-8-sig').splitlines())}
    counts = Counter()
    failures = {}
    for case in cases:
        fixtures = {f['target']: f['value'] for f in case['cell_fixture']}
        def items(value, direct=True):
            kind = value['kind']
            if kind == 'reference':
                return items(fixtures[value['target']], False)
            if kind == 'array':
                return [entry for row in value['rows'] for cell in row for entry in items(cell, False)]
            return [(value, direct)]
        def error(value, direct):
            if value['kind'] == 'error':
                return 'error:' + value['code']
            if value['kind'] == 'text' and direct:
                assert value['value'] == 'x'
                return 'error:Value'
            return None
        flattened = [entry for arg in case['args'] for entry in items(arg)]
        for mode in ['flattened_left_to_right', 'direct_scalar_pass_then_collections']:
            sequence = flattened if mode.startswith('flattened') else [p for p in flattened if p[1]] + [p for p in flattened if not p[1]]
            predicted = next((e for value, direct in sequence if (e := error(value, direct))), None)
            actual = answers[case['case_id']]['outcome']['digest_payload']
            observed = actual if actual.startswith('error:') else None
            key = case['canonical_surface_name'] + ':' + mode
            counts[key] += predicted != observed
            if predicted != observed:
                failures.setdefault(key, []).append({'case': case, 'expected': actual, 'predicted_error': predicted})
    report = {'research_only': True, 'rows': len(cases), 'model_error_mismatches': dict(counts), 'misses': failures,
              'no_numerical_value_claim_from_error_model': True}
    (out/'model-analysis.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(dict(counts))


if __name__ == '__main__':
    main()
