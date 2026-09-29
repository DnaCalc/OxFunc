"""Classify observed text-coercion lanes without changing runtime or oracle data."""
import hashlib
import json
from collections import Counter, defaultdict
from pathlib import Path

run = Path('smart-fuzzer/runs/w111-numeric-text-20260929')
out = Path('docs/function-lane/evidence/w111-broad-20260929/numeric-text-analysis')
out.mkdir(parents=True, exist_ok=True)
def rows(path):
    return [json.loads(line) for line in path.read_text(encoding='utf-8-sig').splitlines()]
cases = rows(run/'cases/cases.jsonl')
excel = {r['case_id']:r for r in rows(run/'outcomes/excel.jsonl')}
local = {r['case_id']:r for r in rows(run/'outcomes/local.jsonl')}
aggregates = {'FUNC.SUM','FUNC.AVERAGE','FUNC.MIN','FUNC.MAX','FUNC.PRODUCT'}
groups = defaultdict(list)
counts = Counter()
by_function = Counter()
by_text = Counter()
reproductions = []
for case in cases:
    key = case['case_id']
    observed = excel[key]
    candidate = local[key]
    assert observed['execution_status'] == candidate['execution_status'] == 'ok'
    expected = observed['outcome']
    actual = candidate['outcome']
    text = case['args'][0].get('value')
    if text is None: text = case['cell_fixture'][0]['value']['value']
    origin = case['case_tag'].split('-')[0]
    fn = case['function_id']
    different = expected['digest_payload'] != actual['digest_payload']
    if not different: category = 'exact_match'
    elif fn == 'FUNC.NOT': category = 'logical_argument_policy'
    elif expected['kind'] == actual['kind'] == 'number': category = 'numeric_kernel'
    else: category = 'implicit_numeric_text_parser'
    counts[category] += 1
    if different:
        by_function[fn] += 1
        by_text[text] += 1
        reproductions.append({'case_id':key,'function':fn,'text':text,'origin':origin,
                              'category':category,'excel':expected,'local':actual})
    groups[fn].append({'text':text,'origin':origin,'excel':expected['digest_payload'],
                       'category':category})

numeric = sorted(fn for fn in groups if fn not in aggregates | {'FUNC.NOT'})
summary = {'schema_version':'w111.numeric_text_classification.v1','rows':len(cases),
           'oracle_source':str(run),'counts':dict(counts),
           'classes':{'numeric_scalar_direct_and_reference':numeric,
                      'aggregate_direct_numeric_reference_text_ignored':sorted(aggregates),
                      'logical_text_only':['FUNC.NOT']},
           'mismatches_by_function':dict(by_function), 'mismatches_by_text':dict(by_text),
           'source_sha256':{str(p):hashlib.sha256(p.read_bytes()).hexdigest()
                            for p in [run/'cases/cases.jsonl',run/'outcomes/excel.jsonl',
                                      run/'outcomes/local.jsonl',run/'manifest.json']},
           'function_observations':dict(groups),'mismatches':reproductions}
(out/'classification.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2),encoding='utf-8')
print(dict(counts))
