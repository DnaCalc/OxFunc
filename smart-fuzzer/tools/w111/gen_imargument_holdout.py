"""Independent principal-argument quadrants, axes and ratio admission probes."""
import hashlib
import json
from pathlib import Path
import random
import sys
import gen_broad_typed_20260929 as typed

SEED=int(sys.argv[2]) if len(sys.argv)>2 else 2026092913
rng=random.Random(SEED)
typed.TRANCHE='w111-imargument-heldout-20260929'
for index in range(400):
    a=rng.choice([-1,1])*rng.randint(1,9999)
    b=rng.choice([-1,1])*rng.randint(1,9999)
    re=rng.randint(-300,300)
    im=re if index<200 else rng.randint(-300,300)
    typed.emit('IMARGUMENT','independent-quadrant',[typed.t(f'{a}e{re}{b:+}e{im}i')])
for a in ['0','-0','1','-1','1e-300','-1e-300','1e300','-1e300']:
    for b in ['0','-0','1','-1','1e-300','-1e-300','1e300','-1e300']:
        typed.emit('IMARGUMENT','axis-scale-grid',[typed.t(a+('' if b.startswith('-') else '+')+b+'i')])
for value in ['i','-i','j','-j','0','-0','1','-1','1+i','-1-i','1-j','-1+j']:
    typed.emit('IMARGUMENT','implicit-unit-axis',[typed.t(value)])
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
    authority='non_semantic_exploration_input',generator=Path(__file__).name,seed=SEED,
    candidate_sha256=hashlib.sha256(Path('crates/oxfunc_core/src/functions/complex_family.rs').read_bytes()).hexdigest(),
    tranche_id=typed.TRANCHE,comparison_policy='exact_typed_bit_match_no_tolerance',
    cases=typed.cases,tranches=[],summary=dict(case_count=len(typed.cases),surfaces_covered=1)),indent=2),encoding='utf-8')
print(len(typed.cases),out)
