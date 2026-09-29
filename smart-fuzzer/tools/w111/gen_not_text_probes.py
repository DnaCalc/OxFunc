"""Independent NOT logical-text controls across direct, reference, and array origins."""
import itertools
import json
from pathlib import Path
import sys
import gen_broad_typed_20260929 as typed

typed.TRANCHE = 'w111-not-text-20260929'
texts = [''.join(chars) for word in ('TRUE', 'FALSE')
         for chars in itertools.product(*[(c.lower(), c) for c in word])]
texts += ['','0','1','-1','2.5','1e3','1,000','10%','TRUEFALSE','ＴＲＵＥ']
for word in ('TRUE', 'FALSE'):
    for whitespace in (' ', '\t', '\r', '\n', '\u00a0', '\u2003'):
        texts += [whitespace + word, word + whitespace, whitespace + word + whitespace]
values = [typed.t(s) for s in texts] + [typed.n(n) for n in (0,-1,1,0.125,-0.125)]
values += [typed.b(False), typed.b(True)] + [typed.e(e) for e in typed.ERR]
for value in values:
    typed.emit('NOT','direct',[value])
    typed.emit('NOT','reference',[typed.r('A1')],[typed.fix('A1',value)])
    typed.emit('NOT','row-array',[typed.a([[value,typed.b(True)]])])
    typed.emit('NOT','column-array',[typed.a([[typed.b(False)],[value]])])
    typed.emit('NOT','reference-array',[typed.r('A1:B1')],
               [typed.fix('A1:B1',typed.a([[value,typed.b(True)]]))])
typed.emit('NOT','blank-reference',[typed.r('A1')],[typed.fix('A1',typed.blank())])
typed.emit('NOT','blank-reference-array',[typed.r('A1:B1')],
           [typed.fix('A1:B1',typed.a([[typed.blank(),typed.t('')]]))])
typed.emit('NOT','missing',[typed.missing()])
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    authority='non_semantic_exploration_input',generator=Path(__file__).name,
    tranche_id=typed.TRANCHE,comparison_policy='exact_typed_bit_match_no_tolerance',
    cases=typed.cases,tranches=[],summary=dict(case_count=len(typed.cases),surfaces_covered=1)),
    indent=2),encoding='utf-8')
print(len(typed.cases),out)
