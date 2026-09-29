"""Retain the bitwise promotion review against later current-baseline evidence."""
import hashlib
import json
from collections import Counter
from pathlib import Path

root = Path(__file__).resolve().parents[3]
out = root / 'docs/function-lane/evidence/w111-broad-20260929/bitwise'
functions = ['BITAND', 'BITOR', 'BITXOR', 'BITLSHIFT', 'BITRSHIFT']
load = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
def lines(path):
    return {v['case_id']: v for v in map(json.loads, filter(str.strip, path.read_text(encoding='utf-8-sig').split('\n')))}
def changed(bits):
    n = int(bits, 16)
    return n == 0x8000000000000000 or (n & 0x7ff0000000000000 == 0 and n & 0xfffffffffffff != 0)
audit = {'scope_completeness':'scope_partial', 'target_completeness':'target_partial',
         'integration_completeness':'partial', 'recommendation':'retain Divergent/Structural',
         'open_lanes':['current-baseline shared numeric text context', 'remaining array precedence coverage',
                       'canonical contract and completion review', 'other baseline/locale/platform phases'],
         'functions':{}}
for fn in functions:
    counts = {}
    for phase in ['discovery','discriminator','heldout']:
        data = load(out / phase / f'answers-{fn.lower()}.json')['witnesses']
        excluded = sum(any(changed(arg) for arg in v['args']) for v in data)
        counts[phase] = {'raw':len(data), 'qualified':len(data)-excluded, 'ingress_excluded':excluded}
    for phase in ['typed-discovery','typed-heldout']:
        data = lines(out / phase / 'cases/cases.jsonl')
        counts[phase] = sum(v['function_id'] == 'FUNC.'+fn for v in data.values())
    audit['functions'][fn] = counts
freeze = load(out / 'freeze-v2.json')
audit['frozen_source_checks'] = []
for name, expected in freeze['source_sha256'].items():
    current = hashlib.sha256((root/name).read_bytes()).hexdigest()
    audit['frozen_source_checks'].append({'path':name,'frozen_sha256':expected,'current_sha256':current,'equal':current==expected})
run = root / 'smart-fuzzer/runs/w111-numeric-text-20260929'
sources = [run/'cases/cases.jsonl', run/'outcomes/excel.jsonl', run/'outcomes/local-bitwise-promotion-audit-v1.jsonl',
           root/'smart-fuzzer/cache/w111-typed-bitwise-promotion-audit-v1-replay.json']
cases, excel, local = [lines(p) for p in sources[:3]]
rows = []
for cid, case in cases.items():
    if case['function_id'][5:] not in functions: continue
    actual, expected = local[cid], excel[cid]
    admitted = actual['execution_status'] == expected['execution_status'] == 'ok'
    exact = actual['outcome']['digest_payload'] == expected['outcome']['digest_payload']
    rows.append({'case':case,'excel':expected,'local':actual,'admitted':admitted,'exact':exact})
audit['current_text_rows'] = len(rows)
audit['current_text_exact'] = sum(row['exact'] for row in rows)
audit['current_text_admitted'] = sum(row['admitted'] for row in rows)
audit['current_text_misses_by_function'] = dict(Counter(row['case']['function_id'] for row in rows if row['admitted'] and not row['exact']))
audit['source_manifest'] = [{'path':str(p.relative_to(root)).replace('\\','/'),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in sources]
(out/'promotion-audit-current-text.json').write_text(json.dumps(rows,ensure_ascii=False,separators=(',',':'))+'\n',encoding='utf-8')
(out/'promotion-audit.json').write_text(json.dumps(audit,indent=2)+'\n',encoding='utf-8')
print(json.dumps(audit,indent=2))
