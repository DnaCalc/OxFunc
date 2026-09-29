"""Independent complex-text division holdout after scaled-division candidate freeze."""
import json
from pathlib import Path
import random
import sys
import gen_broad_typed_20260929 as typed

SEED=2026092907
rng=random.Random(SEED)
typed.TRANCHE='w111-imdiv-heldout-20260929'

def value():
    scale=rng.randint(-250,250)
    real=rng.randint(-9999,9999)
    imag=rng.randint(-9999,9999)
    if rng.randrange(8)==0:real=0
    if rng.randrange(8)==0:imag=0
    return f'{real}e{scale}{imag:+}e{scale}i'

for index in range(512):
    left,right=value(),value()
    if index%8==0:right=left
    typed.emit('IMDIV','independent-scaled-components',[typed.t(left),typed.t(right)])

out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    authority='non_semantic_exploration_input',generator=Path(__file__).name,seed=SEED,tranche_id=typed.TRANCHE,
    comparison_policy='exact_typed_bit_match_no_tolerance',cases=typed.cases,
    tranches=[dict(tranche_id=typed.TRANCHE,case_ids=[c['case_id'] for c in typed.cases])],
    summary=dict(case_count=len(typed.cases),surfaces_covered=1)),indent=2),encoding='utf-8')
print(len(typed.cases),out)
