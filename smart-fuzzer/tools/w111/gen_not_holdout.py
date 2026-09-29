"""Independent mixed-shape holdout for NOT's origin-independent logical-text rule."""
import hashlib
import json
from pathlib import Path
import random
import sys
import gen_broad_typed_20260929 as typed

SEED=2026092910
rng=random.Random(SEED)
typed.TRANCHE='w111-not-heldout-20260929'
values=[typed.t(s) for s in ['tRuE','FAlsE','true','FALSE','TRUE\n','\u00a0FALSE','false ',
    '0','-2','0.0','1e0','%1','(1)','','fAlſe','TRÜE','TRUE\u200b']]
values += [typed.n(n) for n in [0,1,-1,1e-100,1e100]]
values += [typed.b(False),typed.b(True)]+[typed.e(e) for e in typed.ERR]
for index in range(60):
    height,width=rng.choice([(2,3),(3,2),(1,4),(4,1)])
    matrix=typed.a([[rng.choice(values) for _ in range(width)] for _ in range(height)])
    typed.emit('NOT','mixed-shape',[matrix])
    target=f'A1:{chr(64+width)}{height}'
    typed.emit('NOT','mixed-reference-shape',[typed.r(target)],[typed.fix(target,matrix)])
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    authority='non_semantic_exploration_input',generator=Path(__file__).name,seed=SEED,
    tranche_id=typed.TRANCHE,comparison_policy='exact_typed_bit_match_no_tolerance',
    candidate_sha256=hashlib.sha256(Path('crates/oxfunc_core/src/functions/not_fn.rs').read_bytes()).hexdigest(),
    cases=typed.cases,tranches=[],summary=dict(case_count=len(typed.cases),surfaces_covered=1)),indent=2),encoding='utf-8')
print(len(typed.cases),out)
