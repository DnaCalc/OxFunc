"""Independent IMSQRT complex-text holdout after the operation graph is frozen.

Simple integer coefficient spellings focus on arithmetic, rather than long
decimal-text parsing. Includes ordinary quadrants, independently scaled
components, squared-term boundaries and signed axes.
"""
import json
from pathlib import Path
import random
import sys
import gen_broad_typed_20260929 as typed

SEED=int(sys.argv[2]) if len(sys.argv)>2 else 2026092908
rng=random.Random(SEED)
typed.TRANCHE='w111-imsqrt-heldout-20260929'

def add(a,ea,b,eb,tag):
    value=f'{a}e{ea}{b:+}e{eb}i'
    typed.emit('IMSQRT',tag,[typed.t(value)])

for _ in range(400):
    exponent=rng.randint(-120,120)
    add(rng.randint(-9999,9999),exponent,rng.randint(-9999,9999),exponent,'ordinary-quadrant')
for _ in range(240):
    add(rng.randint(-9999,9999),rng.randint(-300,300),rng.randint(-9999,9999),rng.randint(-300,300),'independent-scales')
for _ in range(160):
    exponent=rng.choice([-157,151])
    add(rng.randint(-2000,2000),exponent,rng.randint(-2000,2000),exponent,'squared-term-boundary')
for _ in range(160):
    exponent=rng.randint(-300,300);n=rng.randint(-9999,9999)
    add(n if rng.randrange(2) else 0,exponent,0 if n%2 else n,exponent,'signed-axis')

out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    authority='non_semantic_exploration_input',generator=Path(__file__).name,seed=SEED,tranche_id=typed.TRANCHE,
    comparison_policy='exact_typed_bit_match_no_tolerance',cases=typed.cases,
    tranches=[dict(tranche_id=typed.TRANCHE,case_ids=[c['case_id'] for c in typed.cases])],
    summary=dict(case_count=len(typed.cases),surfaces_covered=1)),indent=2),encoding='utf-8')
print(len(typed.cases),out)
