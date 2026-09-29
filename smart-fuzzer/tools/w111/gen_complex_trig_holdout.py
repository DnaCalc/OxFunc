"""Independent complex-axis sine/cosine holdout after discovery candidate freeze."""
import hashlib
import json
from pathlib import Path
import random
import sys
import gen_broad_typed_20260929 as typed

SEED=int(sys.argv[2]) if len(sys.argv)>2 else 2026092911
rng=random.Random(SEED)
typed.TRANCHE='w111-complex-trig-heldout-20260929'
values=[]
for _ in range(320):
    real=rng.uniform(-10000,10000)
    imag=rng.uniform(-700,700) if len(values)%2 else rng.uniform(-30,30)
    values.append(f'{real:.17g}{imag:+.17g}i')
for real in ['0','-0','1','-1','1.5707963267948966','3.141592653589793','134217727','134217728']:
    for imag in ['1e-300','-1e-300','1e-16','-1e-16','1e-8','-1e-8','709','710']:
        values.append(real+('' if imag.startswith('-') else '+')+imag+'i')
for fn in ['IMCOS','IMSIN']:
    for value in values: typed.emit(fn,'independent-complex-axis',[typed.t(value)])
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    authority='non_semantic_exploration_input',generator=Path(__file__).name,seed=SEED,
    candidate_sha256=hashlib.sha256(Path('crates/oxfunc_core/src/functions/complex_family.rs').read_bytes()).hexdigest(),
    tranche_id=typed.TRANCHE,comparison_policy='exact_typed_bit_match_no_tolerance',
    cases=typed.cases,tranches=[],summary=dict(case_count=len(typed.cases),surfaces_covered=2)),indent=2),encoding='utf-8')
print(len(typed.cases),out)
