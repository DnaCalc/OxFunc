"""Freeze raw observed GCD/LCM rows and test them through production dispatch."""
import json,shutil
from pathlib import Path

OUT=Path('docs/function-lane/evidence/w111-broad-20260929/gcd-lcm')
RUNS=Path('smart-fuzzer/runs')
def main():
    OUT.mkdir(parents=True,exist_ok=True)
    for tag,run in [('discovery2','w111-gcd-lcm-discovery2-20260929'),('refinement','w111-gcd-lcm-refinement-20260929'),('heldout','w111-gcd-lcm-heldout-20260929')]:
        for fn in ['gcd','lcm']:shutil.copyfile(RUNS/run/f'answers-{fn}.json',OUT/f'numeric-{tag}-{fn}.json')
    for tag,run in [('discovery2','w111-gcd-lcm-discovery2-typed-20260929'),('refinement','w111-gcd-lcm-refinement-typed-20260929'),('array-order','w111-lcm-array-order-typed-20260929'),('heldout','w111-gcd-lcm-heldout-typed-20260929')]:
        cases=[json.loads(s) for s in (RUNS/run/'cases/cases.jsonl').read_text(encoding='utf-8-sig').splitlines()]
        excel={r['case_id']:r for r in map(json.loads,(RUNS/run/'outcomes/excel.jsonl').read_text(encoding='utf-8-sig').splitlines())}
        witnesses=[];withheld=[]
        for case in cases:
            row={'case':case,'excel':excel[case['case_id']]}
            (witnesses if row['excel']['execution_status']=='ok' else withheld).append(row)
        (OUT/f'typed-{tag}.json').write_text(json.dumps({'source_run':run,'witnesses':witnesses,'withheld':withheld},separators=(',',':')),encoding='utf-8')
        print(tag,len(witnesses),'admitted;',len(withheld),'withheld')
    source=Path('.tmp/w111-oracle-thresholds-1645.log')
    if source.exists():shutil.copyfile(source,OUT/'original-local-overflow.log')
    base=Path('crates/oxfunc_core/tests/w111_numeric_to_text_live_replay.rs').read_text()
    base=base[:base.index('\n#[test]')].replace('//! Generic Number-to-Text observations through production surface dispatch.\n//! COMPLEX controls are kept separate; contextual scope remains partial.','//! GCD/LCM retained typed observations through production dispatch.').replace('fn replay(source: &str)','fn replay_typed(source: &str)')
    numeric=Path('crates/oxfunc_core/tests/w111_quotient_live_replay.rs').read_text()
    numeric=numeric[numeric.index('fn replay(source: &str)'):numeric.index('\n#[test]')].replace('fn replay(source: &str)','fn replay_numeric(source: &str)').replace('"FUNC.QUOTIENT"','&format!("FUNC.{}", evidence["function"].as_str().unwrap())').replace('&NULL_REFERENCE_SYSTEM_PROVIDER','&oxfunc_core::resolver::NULL_REFERENCE_SYSTEM_PROVIDER')
    base+='\n'+numeric
    for name in ['initial','discovery2','refinement','array-order','heldout']:
        base+=f'\n#[test]\nfn gcd_lcm_typed_{name.replace("-","_")}() {{ replay_typed(include_str!("../../../{OUT.as_posix()}/typed-{name}.json")); }}\n'
    for name in ['','discovery2-','refinement-','heldout-']:
        for fn in ['gcd','lcm']:
            base+=f'\n#[test]\nfn {fn}_numeric_{name.replace("-","_") or "initial"}() {{ replay_numeric(include_str!("../../../{OUT.as_posix()}/numeric-{name}{fn}.json")); }}\n'
    Path('crates/oxfunc_core/tests/w111_gcd_lcm_live_replay.rs').write_text(base)

if __name__=='__main__':main()
