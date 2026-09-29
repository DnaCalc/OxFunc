"""Qualify raw numeric judgements by measured Value2 and reference-origin limits.

No expected or actual result is changed. Signed zero/subnormal source-input rows
are withheld from function verdicts because Excel stores a different input.
ISREF requires a reference-aware local route and is withheld in this scalar lane.
Usage: summarize_numeric_campaign.py ANSWERS_DIR JUDGE_JSON OUT_JSON
"""
import json
import sys
from pathlib import Path

answers,judge,out=map(Path,sys.argv[1:4])
reports=json.loads(judge.read_text(encoding='utf-8-sig'))
result={'comparison_policy':'exact_typed_bit_match_no_tolerance',
        'input_limit_evidence':'numeric-ingress.json', 'functions':[]}
for report in reports:
    fn=report['function_id'].removeprefix('FUNC.')
    ws=json.loads((answers/('answers-'+fn.lower()+'.json')).read_text(encoding='utf-8-sig'))
    excluded={}
    for w in ws['witnesses']:
        reason=None
        if fn=='ISREF': reason='reference_argument_compared_with_local_value'
        elif any((int(a,16)&0x7ff0000000000000)==0 and int(a,16)!=0 for a in w['args']):
            reason='Value2_changes_signed_zero_or_subnormal_input'
        if reason: excluded[w['id']]=reason
    misses=[m for m in report['misses'] if m['id'] not in excluded]
    if len(report['misses'])!=report['rows_judged']-report['rows_agree']:
        raise SystemExit(f'Truncated miss list for {fn}; rejudge with --max-misses 10000')
    counts={}
    for m in misses: counts[m['verdict']['severity']]=counts.get(m['verdict']['severity'],0)+1
    severity=next((s for s in ['structural','gross','numeric','last_bit'] if s in counts),None)
    judged=report['rows_judged']-len(excluded)
    result['functions'].append(dict(function_id=report['function_id'],raw_rows=report['rows_judged'],
        admitted_rows=judged,admitted_agree=judged-len(misses),severity=severity,
        misses_by_severity=counts,withheld_rows=len(excluded),withheld_ids=excluded,
        misses=misses))
result['totals']={k:sum(x[k] for x in result['functions']) for k in
                  ['raw_rows','admitted_rows','admitted_agree','withheld_rows']}
out.write_text(json.dumps(result,separators=(',',':')),encoding='utf-8')
print(result['totals'])
print('Functions with admitted mismatches:',sum(bool(x['misses']) for x in result['functions']))
