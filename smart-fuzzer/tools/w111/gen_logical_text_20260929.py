"""Deterministic direct/array logical-text probes; no numeric literals."""
import json
import sys
from pathlib import Path

cases = []
texts = ["TRUE", "true", "TrUe", "FALSE", "FaLsE", " TRUE", "TRUE ", " TRUE ",
         "\tTRUE", "TRUE\t", "\nTRUE", "TRUE\r\n", "\u00a0TRUE", "TRUE\u00a0",
         "1", "0", "", "x", "  FALSE ", "\tFALSE\t"]

def tok(a):
    if a['kind'] == 'text': return '"' + a['value'].replace('"', '""') + '"'
    if a['kind'] == 'logical': return str(a['value']).upper()
    if a['kind'] == 'error': return {'NA':'#N/A', 'Div0':'#DIV/0!'}[a['code']]
    if a['kind'] == 'array': return '{' + ';'.join(','.join(tok(x) for x in row) for row in a['rows']) + '}'
    raise ValueError(a)

def add(fn, tag, args):
    cases.append(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case.v0',
        run_id='assigned_by_runner', tranche_id='w111-logical-text-20260929',
        case_id=f'logical-{fn.lower()}-{len(cases):04d}', function_id=f'FUNC.{fn}',
        canonical_surface_name=fn, case_tag=tag, axis='logical_text_origin',
        expected_probe_class='typed_value_call', formula_text=f'={fn}('+','.join(map(tok,args))+')',
        args=args, cell_fixture=[], formula_cell=None, category='Logical functions',
        blocked_or_deferred_lanes=[], known_deviation_tags=[]))

for fn in ['AND','OR','XOR']:
    neutral = dict(kind='logical', value=fn=='AND')
    for text in texts:
        t = dict(kind='text',value=text)
        for tag, args in [('alone',[t]),('neutral-first',[neutral,t]),('text-first',[t,neutral]),
                          ('array-alone',[dict(kind='array',rows=[[t]])]),
                          ('array-neutral',[neutral,dict(kind='array',rows=[[t]])])]:
            add(fn,tag,args)
        for code in ['NA','Div0']:
            e=dict(kind='error',code=code)
            add(fn,'text-error',[t,e]); add(fn,'error-text',[e,t])

out = Path(sys.argv[1]); out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    authority='non_semantic_exploration_input', generator=Path(__file__).name,
    tranche_id='w111-logical-text-20260929',comparison_policy='exact_typed_bit_match_no_tolerance',
    cases=cases,tranches=[],skipped=[],summary=dict(case_count=len(cases),surfaces_covered=3)),
    ensure_ascii=False,indent=2),encoding='utf-8')
print(f'{len(cases)} probes -> {out}')
