"""Retain immutable count/width captures and replay exercised production paths."""
import hashlib,json,shutil
from pathlib import Path

OUT=Path('docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries')
RUNS=Path('smart-fuzzer/runs')
def main():
    OUT.mkdir(parents=True,exist_ok=True);manifest={};total=0
    for fn in ['TRUNC','ROUNDDOWN','ROUNDUP','ROUND']:
        witnesses=[];captures=[]
        for name in ['integer-hyperbolic-heldout','rounding-count-refinement','integer-thresholds','integer-refinement','trunc-hyperbolic']:
            run=RUNS/f'w111-{name}-20260929';source=run/'answers'/f'answers-{fn.lower()}.json'
            if not source.exists():source=run/f'answers-{fn.lower()}.json'
            if not source.exists():continue
            data=json.loads(source.read_text(encoding='utf-8-sig'));witnesses+=data['witnesses'];captures.append({'source':source.as_posix(),'sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'rows':len(data['witnesses']),'capture_provenance':data.get('capture_provenance')})
        assert len({r['id'] for r in witnesses})==len(witnesses)
        document={'function':fn,'witnesses':witnesses,'captures':captures,'phase':'refinement_not_independent_validation_of_current_candidate'}
        (OUT/f'numeric-{fn.lower()}.json').write_text(json.dumps(document,separators=(',',':')),encoding='utf-8');manifest[fn]={'rows':len(witnesses),'sources':captures};total+=len(witnesses)
    freeze=RUNS/'w111-integer-hyperbolic-heldout-20260929'
    (OUT/'original-frozen-candidate').mkdir(exist_ok=True)
    shutil.copyfile(freeze/'candidate-freeze.json',OUT/'original-frozen-candidate'/'freeze.json')
    for name in ['round_fn.rs','rounddown_fn.rs','roundup_fn.rs','trunc_fn.rs']:
        shutil.copyfile(freeze/'candidate'/name,OUT/'original-frozen-candidate'/name)
    source=Path('crates/oxfunc_core/tests/w111_quotient_live_replay.rs').read_text()
    source=source[:source.index('\n#[test]')].replace('//! QUOTIENT exact-bit observations, with Value2 input exclusions explicit.','//! Rounding count/width/assembly observations; exceptional initial15 lanes remain open.').replace('"FUNC.QUOTIENT"','&format!("FUNC.{}", evidence["function"].as_str().unwrap())')
    for fn in ['TRUNC','ROUNDDOWN','ROUNDUP','ROUND']:
        source+=f'\n#[test]\nfn {fn.lower()}_count_and_scale_refinement() {{ replay(include_str!("../../../{OUT.as_posix()}/numeric-{fn.lower()}.json")); }}\n'
    test_path=Path('crates/oxfunc_core/tests/w111_rounding_boundary_live_replay.rs')
    # Later endpoint/residual replay additions are maintained in this test.
    # Re-retaining historical inputs must not discard those exercised lanes.
    if not test_path.exists():test_path.write_text(source)
    (OUT/'retention.json').write_text(json.dumps({'total_rows':total,'functions':manifest},indent=2),encoding='utf-8');print(total,'retained rows')
if __name__=='__main__':main()
