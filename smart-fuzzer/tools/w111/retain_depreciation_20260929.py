"""Retain lossless depreciation oracle data and candidate lineage, never transcripts."""
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parents[3]
cache=root/"smart-fuzzer/cache/w111-depreciation-20260929"
dest=root/"docs/function-lane/evidence/w111-broad-20260929/depreciation"
dest.mkdir(exist_ok=True)
manifest=[]
def retain(src,name):
    if not src.exists():return
    data=src.read_bytes()
    if src.suffix=='.jsonl':payload=[json.loads(line) for line in data.decode('utf-8-sig').split('\n') if line.strip()]
    else:payload=json.loads(data.decode('utf-8-sig'))
    target=dest/name
    target.write_text(json.dumps(payload,ensure_ascii=False,separators=(',',':'))+'\n',encoding='utf-8')
    manifest.append(dict(source=str(src.relative_to(root)).replace('\\','/'),source_sha256=hashlib.sha256(data).hexdigest(),retained=name,retained_sha256=hashlib.sha256(target.read_bytes()).hexdigest()))
for fn in ['db','ddb','vdb']:
    retain(root/f'smart-fuzzer/runs/w111-extended-numeric-20260929/answers/answers-{fn}.json',f'primary-{fn}.json')
    for phase in ['answers-','heldout-','refinement-','second-heldout-','refinement2-','refinement3-','refinement4-','refinement5-','refinement6-','refinement7-','refinement8-','refinement9-','third-heldout-','fourth-heldout-','fifth-heldout-','sixth-heldout-','seventh-heldout-','eighth-heldout-']:
        name=phase+fn+('.json' if phase=='answers-' else '-answers.json')
        retain(cache/name,name)
for name in ['freeze-db-ddb-v1.json','freeze-db-ddb-v2.json','freeze-db-ddb-v3.json',
             'db-v7-discovery-judgement.json','db-v7-full-refined-judgement.json','db-large-exponent-judgement.json','db-large-exponent-refined-judgement.json','ddb-large-exponent-judgement.json','ddb-large-exponent-refined-judgement.json','depreciation-bounded-power-refined-judgement.json','freeze-db-v7.json','freeze-ddb-v5.json','oracle-only-vdb-large-index.json','oracle-only-vdb-large-index-answers.json','latest-independent-power-judgement.json','observed-vdb-large-index-safe-prefix.json','observed-vdb-large-index-safe-prefix-judgement.json','oracle-only-vdb-owned-process-stop.json','freeze-db-v8.json','freeze-ddb-v6.json','final-stage-discriminators-before.json','final-stage-refined-all-judgement.json','typed-independent.json','typed-independent-judgement.json','dbv8-ddbv6-independent-judgement.json','switched-research-candidate.json','vdb-shift-negative-residual.json','vdb-switched-bounded-discovery.json','refinement3-vdb-switched.json','refinement3-vdb-switched-answers.json','candidate-v1-judgement.json','heldout-v1-full-judgement.json',
             'refined-db-judgement.json','refined-ddb-judgement.json','second-heldout-v2-judgement.json',
             'final-numeric-judgement.json','third-heldout-judgement.json',
             'db-v3-judgement.json','db-v4-judgement.json','db-v5-judgement.json','ddb-v4-judgement.json',
             'vdb-noswitch-v2-judgement.json','latest-heldout-judgement.json',
             'freeze-db-v4.json','freeze-ddb-v4.json','freeze-db-v5.json','freeze-db-v6.json','freeze-vdb-noswitch-v2.json',
             'fifth-heldout-db-judgement.json','sixth-heldout-db-judgement.json',
             'db-v6-discovery-judgement.json','db-store-discriminator-judgement.json','db-v7-initial-judgement.json','db-v7-discriminator-judgement.json',
             'refinement2-vdb-switched-answers.json','switched-vdb-counterexamples.json','fourth-heldout-db-judgement.json','refinement3-ddb-before.json','freeze-vdb-noswitch-v1.json',
             'vdb-noswitch-production-v1.json','heldout-vdb-noswitch-answers.json',
             'heldout-vdb-noswitch-judgement.json','refinement2-vdb-noswitch-answers.json','vdb-noswitch-refined-production.json','second-heldout-vdb-noswitch-answers.json',
             'vdb-noswitch-discovery.json','refinement-latest-judgement.json']:
    retain(cache/name,name)
for fn in ['db','ddb','vdb']:
    for phase in ['discriminator','heldout','refinement','second-heldout','refinement2','refinement3','refinement4','refinement5','refinement6','refinement7','refinement8','refinement9','third-heldout','fourth-heldout','fifth-heldout','sixth-heldout','seventh-heldout','eighth-heldout']:
        p=cache/f'{phase}-{fn}.json'
        retain(p,f'inputs-{p.name}')
for name in ['heldout-vdb-noswitch.json','refinement2-vdb-noswitch.json','second-heldout-vdb-noswitch.json','refinement2-vdb-switched.json']:
    retain(cache/name,'inputs-'+name)
for name in ['third-heldout-vdb-noswitch.json','fourth-heldout-vdb-noswitch.json','refinement3-vdb-noswitch.json','refinement4-vdb-switched.json','discriminator-syd.json']:
    retain(cache/name,'inputs-'+name)
for name in ['third-heldout-vdb-noswitch-answers.json','third-heldout-vdb-noswitch-judgement.json','fourth-heldout-vdb-noswitch-answers.json','fourth-heldout-vdb-noswitch-judgement.json','refinement3-vdb-noswitch-answers.json','vdb-noswitch-v4-discovery-judgement.json','vdb-noswitch-v4-earlier-judgement.json','freeze-vdb-noswitch-v3.json','freeze-vdb-noswitch-v4.json','refinement4-vdb-switched-answers.json','vdb-stage-trace-judgement.json','vdb-transition-trace-judgement.json','discriminator-syd-answers.json','syd-discriminator-before.json']:
    retain(cache/name,name)
retain(cache/'vdb-partial-store-discriminator.jsonl','vdb-partial-store-discriminator.json')
for name in ['fifth-heldout-vdb-noswitch.json','refinement4-vdb-noswitch.json','second-heldout-syd.json','heldout-syd.json','refinement-syd.json','refinement2-syd.json','typed-syd.json','typed-independent-syd.json']:
    retain(cache/name,'inputs-'+name)
for name in ['fifth-heldout-vdb-noswitch-answers.json','fifth-heldout-vdb-noswitch-judgement.json','refinement4-vdb-noswitch-answers.json','vdb-noswitch-v5-prior-judgement.json','vdb-noswitch-v5-discriminator-judgement.json','freeze-vdb-noswitch-v5.json','freeze-syd-v1.json','freeze-syd-v2.json','syd-v1-discovery-judgement.json','syd-v2-discovery-judgement.json','heldout-syd-answers.json','heldout-syd-v1-judgement.json','second-heldout-syd-answers.json','second-heldout-syd-judgement.json','refinement-syd-answers.json','refinement2-syd-answers.json','typed-syd-before.json']:
    retain(cache/name,name)
retain(root/'smart-fuzzer/runs/w111-broad-20260929/answers/answers-syd.json','primary-syd.json')
retain(root/'smart-fuzzer/cache/w111-typed-syd-v2-replay.json','typed-syd-v2-replay.json')
for name in ['third-heldout-syd.json','refinement3-syd.json','typed-depreciation-padding.json','typed-depreciation-padding-independent.json','heldout-vdb-switched-research.json']:
    retain(cache/name,'inputs-'+name)
for name in ['freeze-syd-v3.json','third-heldout-syd-answers.json','third-heldout-syd-judgement.json','refinement3-syd-answers.json','syd-extreme-basis-judgement.json','syd-v3-extreme-basis-judgement.json','typed-independent-syd-judgement.json','typed-depreciation-padding-before.json','heldout-vdb-switched-research-answers.json','heldout-vdb-switched-research-judgement.json','switched-research-independent-freeze.json','vdb-unswitched-clipping-heldout-replay.json','vdb-explicit-clipping-heldout-replay.json','frozen-function-body-check.json']:
    retain(cache/name,name)
for tag in ['syd-v3','depreciation-v7','depreciation-padding-v1','depreciation-padding-independent-v1','vdb-switched-production-v1']:
    retain(root/f'smart-fuzzer/cache/w111-typed-{tag}-replay.json',f'typed-{tag}-replay.json')
for pattern in ['vdb-clipping-replay-*.json','vdb-duration-*.json','vdb-inherited-sl-duration-*.json','switched-*-store-axis-search.json','vdb-first-step-sl-duration-*.json','vdb-x87-shifted-end-*.json','vdb-absolute-period-end-*.json','vdb-absolute-life-*.json','vdb-interval-*.json','vdb-cursor-loop-*.json','vdb-negative-basis-general-*.json','vdb-initial-store-*-judgement.json','vdb-direct-width-count-*.json','vdb-width-and-cursor-*.json','vdb-final-remaining-period-*.json','refinement5-vdb-switched-mode*-judgement.json','typed-vdb-switched-recheck-research-*.json']:
    for p in sorted(cache.glob(pattern)):retain(p,p.name)
for name in ['switched-research-endpoint-freeze.json','refinement5-vdb-switched.json','refinement5-vdb-switched-answers.json','switched-research-eighteen-residuals.json','switched-research-absolute-end-freeze.json','typed-vdb-switched-recheck.json','vdb-switched-ten-local-stage-traces.json','vdb-switched-eight-local-stage-traces.json','vdb-final-take-operation-analysis.json','vdb-absolute-end-ten-residuals.json','vdb-absolute-end-ten-axis-search.json','vdb-absolute-period-equivalence.json','switched-research-cursor-freeze.json','second-heldout-vdb-switched-research.json','second-heldout-vdb-switched-research-answers.json']:
    retain(cache/name,name)
for name in ['second-heldout-vdb-switched-research-judgement.json','switched-research-general-schedule-freeze.json','third-heldout-vdb-switched-research.json','third-heldout-vdb-switched-research-answers.json','third-heldout-vdb-switched-research-judgement.json']:
    retain(cache/name,name)
for name in ['switched-research-final-period-freeze.json','fourth-heldout-vdb-switched-research.json','fourth-heldout-vdb-switched-research-answers.json','fourth-heldout-vdb-switched-research-judgement.json','discovery-vdb-switched-publication.json','discovery-vdb-switched-publication-answers.json']:
    retain(cache/name,name)
for name in ['discovery-vdb-switched-publication-judgement.json','switched-research-publication-freeze.json','heldout-vdb-switched-publication.json','heldout-vdb-switched-publication-answers.json','heldout-vdb-switched-publication-judgement.json','vdb-publication-prior-discovery-replay.json']:
    retain(cache/name,name)
for pattern in ['vdb-publication-trial-*.json','vdb-published-stages-*.json','vdb-publication-stage-discriminator*.json','vdb-published-comparison-*.json','refinement6-vdb-switched-publication*.json','refinement7-vdb-switch-comparison*.json','vdb-comparison-min16-*.json','vdb-dd-ordinary-*.json','vdb-rate-ordinary-*.json','vdb-tiny-book-store-*.json','vdb-tiny-advance-store-all-replay.json','vdb-publication-v2-prior-discovery-replay.json','switched-research-publication-v2-freeze.json','second-heldout-vdb-switched-publication*.json','switched-production-v1-*.json','heldout-vdb-switched-dispatch*.json','typed-vdb-switched-independent*.json','formal-recovery-validation.json','vdb-translation-advance-audit-*.json','progress-safe-*-controls.json','progress-safe-*-answers.json','progress-safe-controls-judgement.json']:
    for p in sorted(cache.glob(pattern)):retain(p,p.name)
for p in sorted(cache.glob('*-manifest.json')):retain(p,p.name)
for tag,run in [('typed','w111-depreciation-typed-20260929'),('typed-options','w111-depreciation-options-20260929'),('typed-heldout','w111-depreciation-typed-heldout-20260929'),('typed-independent','w111-depreciation-typed-independent-20260929'),('typed-syd','w111-syd-typed-20260929'),('typed-independent-syd','w111-syd-typed-independent-20260929'),('typed-padding','w111-depreciation-padding-20260929'),('typed-padding-independent','w111-depreciation-padding-independent-20260929'),('typed-vdb-switched-recheck','w111-vdb-switched-recheck-20260929'),('typed-vdb-switched-independent','w111-vdb-switched-typed-independent-20260929')]:
    run=root/'smart-fuzzer/runs'/run
    for part in ['cases/cases.jsonl','outcomes/excel.jsonl','manifest.json']:
        retain(run/part,tag+'-'+Path(part).name.replace('.jsonl','.json'))
    retain(run/'outcomes/local.jsonl',tag+'-captured-local.json')
    retain(cache/f'{tag}-local-v2.jsonl',tag+'-candidate-v2.json')
    retain(run/'outcomes/local-depreciation-v4.jsonl',tag+'-candidate-v4.json')
    retain(run/'outcomes/local-depreciation-v5.jsonl',tag+'-candidate-v5.json')
    retain(run/'outcomes/local-syd-v2.jsonl',tag+'-candidate-syd-v2.json')
    for version in ['syd-v3','depreciation-v7','depreciation-padding-v1','depreciation-padding-independent-v1','vdb-switched-production-v1']:
        retain(run/f'outcomes/local-{version}.jsonl',tag+'-candidate-'+version+'.json')
retain(root/'smart-fuzzer/cache/w111-typed-depreciation-v4-replay.json','typed-v4-replay-report.json')
retain(root/'smart-fuzzer/cache/w111-typed-depreciation-v5-replay.json','typed-v5-replay-report.json')
for name in ['formal-recovery-equivalence.lean','candidate-depreciation-formal-current.lean']:
    p=cache/name
    if p.exists():
        target=dest/name;target.write_bytes(p.read_bytes())
        manifest.append(dict(source=str(p.relative_to(root)).replace('\\','/'),source_sha256=hashlib.sha256(p.read_bytes()).hexdigest(),retained=name,retained_sha256=hashlib.sha256(target.read_bytes()).hexdigest()))
snapshots=[]
for name in ['candidate-db-ddb-v1-recovered.rs','candidate-db-ddb-v2.rs','candidate-db-ddb-v3.rs','candidate-db-v4.rs','candidate-ddb-v4.rs','candidate-db-v5.rs','candidate-db-v6.rs','candidate-db-v7.rs','candidate-db-v8.rs','candidate-ddb-v6.rs','candidate-ddb-v5.rs','candidate-vdb-noswitch-v1.rs','candidate-vdb-noswitch-v2.rs','candidate-vdb-noswitch-v3.rs','candidate-vdb-noswitch-v4.rs']:
    p=cache/name
    if p.exists():snapshots.append(dict(source=name,sha256=hashlib.sha256(p.read_bytes()).hexdigest(),utf8_source=p.read_bytes().decode('utf-8')))
for name in ['candidate-vdb-noswitch-v5.rs','candidate-syd-v1.rs','candidate-syd-v2.rs','candidate-syd-v3.rs']:
    p=cache/name
    if p.exists():snapshots.append(dict(source=name,sha256=hashlib.sha256(p.read_bytes()).hexdigest(),utf8_source=p.read_bytes().decode('utf-8')))
(dest/'candidate-source-snapshots.json').write_text(json.dumps(snapshots,ensure_ascii=False,separators=(',',':'))+'\n',encoding='utf-8')
(dest/'retention-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(artifacts=len(manifest),source_snapshots=len(snapshots),path=str(dest))))
