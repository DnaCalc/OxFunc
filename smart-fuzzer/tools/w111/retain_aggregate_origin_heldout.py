"""Retain the serialized aggregate repetition and compare it with the provisional run."""
import hashlib
import json
import shutil
import sys
from collections import Counter
from pathlib import Path


def rows(path):
    return {r['case_id']: r for r in map(json.loads, path.read_text(encoding='utf-8-sig').splitlines())}


def main():
    repeat = Path(sys.argv[1])
    initial = Path('smart-fuzzer/runs/w111-aggregate-origin-heldout-typed-20260929')
    assert repeat != initial, 'Require the distinct serialized repetition.'
    base = Path('docs/function-lane/evidence/w111-broad-20260929/elementary-publication')
    retained = base / 'aggregate-origin-heldout'
    target = retained / 'serialized-repeat'
    target.mkdir(parents=True, exist_ok=True)
    for name in ['cases/cases.jsonl', 'outcomes/excel.jsonl', 'outcomes/local.jsonl', 'manifest.json', 'rollup.json']:
        shutil.copyfile(repeat / name, target / name.replace('/', '-'))
    cases = rows(repeat / 'cases/cases.jsonl')
    excel = rows(repeat / 'outcomes/excel.jsonl')
    local = rows(repeat / 'outcomes/local.jsonl')
    earlier = rows(initial / 'outcomes/excel.jsonl')
    initial_cases = rows(initial / 'cases/cases.jsonl')
    assert cases.keys() == excel.keys() == local.keys() == earlier.keys() == initial_cases.keys()
    for case_id, case in cases.items():
        assert all(case[key] == initial_cases[case_id][key]
                   for key in ['args', 'cell_fixture', 'formula_text', 'function_id'])
    differences = []
    nonexecuted = []
    local_misses = []
    counts = Counter()
    exact = Counter()
    for case_id, case in cases.items():
        observed = excel[case_id]
        old = earlier[case_id]
        if observed['execution_status'] != 'ok' or local[case_id]['execution_status'] != 'ok':
            nonexecuted.append(case_id)
        if (observed['execution_status'], observed.get('outcome')) != (old['execution_status'], old.get('outcome')):
            differences.append({'case_id': case_id, 'provisional': old, 'serialized': observed})
        function = case['canonical_surface_name']
        counts[function] += 1
        if observed.get('outcome', {}).get('digest_payload') == local[case_id].get('outcome', {}).get('digest_payload'):
            exact[function] += 1
        else:
            local_misses.append({'case': case, 'excel': observed, 'local': local[case_id]})
    freeze = json.loads((retained / 'candidate-freeze.json').read_text())
    changed = [name for name, sha in freeze['source_sha256'].items()
               if hashlib.sha256(Path(name).read_bytes()).hexdigest() != sha]
    report = {'status': 'in_progress', 'scope_completeness': 'scope_partial',
              'target_completeness': 'target_partial', 'integration_completeness': 'partial',
              'open_lanes': ['contextual_numeric_text', 'numeric_primitive_platform_alignment',
                             'HO_FN_030_receiving_acknowledgment'],
              'serialized_run': str(repeat), 'provisional_run': str(initial),
              'rows': len(cases), 'by_function': dict(counts), 'exact_by_function': dict(exact),
              'serial_vs_provisional_observable_differences': differences,
              'nonexecuted': nonexecuted, 'local_misses': local_misses,
              'frozen_source_count': len(freeze['source_sha256']), 'changed_frozen_sources': changed,
              'counted_independent_observations': len(cases) if not nonexecuted and not changed else 0,
              'repetition_policy': 'Only serialized observations count. Provisional overlap capture remains retained and is compared, not added to evidence totals.'}
    (retained / 'validation.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    witnesses = [{'case': case, 'excel': excel[case_id]} for case_id, case in cases.items()]
    (base / 'aggregate-origin-heldout.json').write_text(json.dumps({'witnesses': witnesses}, separators=(',', ':')), encoding='utf-8')
    print(json.dumps({key: value for key, value in report.items() if key != 'local_misses'}, indent=2))
    if nonexecuted or changed or differences or local_misses:
        raise SystemExit('Retained nonpassing result; inspect validation.json before claiming validation.')


if __name__ == '__main__':
    main()
