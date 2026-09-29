"""Retain reproducible MROUND graph comparisons without duplicated input rows."""
import hashlib, json, shutil, subprocess
from pathlib import Path

dst=Path('docs/function-lane/evidence/w111-broad-20260929/mround')
for path in dst.glob('*graphs.json'):
    report=json.loads(path.read_text())
    if not isinstance(report,dict):continue
    if report.get('schema')=='w111-mround-graph-profiles-v1':continue
    profiles=[];indices={};models={}
    for name in sorted(set(report['exact'])|set(report['failures'])):
        failures=report['failures'].get(name,[])
        # Input bits and expected typed results are in the retained answer bank.
        compact=[[r['id'],r['actual']] for r in failures]
        key=json.dumps(compact,separators=(',',':'))
        if key not in indices:
            indices[key]=len(profiles);profiles.append(compact)
        models[name]=dict(exact=report['exact'].get(name,0),failure_profile=indices[key])
    path.write_text(json.dumps(dict(schema='w111-mround-graph-profiles-v1',
        rows=report['rows'],models=models,failure_profiles=profiles),separators=(',',':')))
shutil.copy2('.tmp/w111-mround-stores-v3.json',dst/'initial-store-graphs.json')
shutil.copy2('crates/oxfunc_core/examples/w111_mround_stores.rs',dst/'initial-store-probe.rs')
if not (dst/'initial-mround.rs').exists():
    source=subprocess.check_output(['git','show','HEAD:crates/oxfunc_core/src/functions/mround.rs'])
    (dst/'initial-mround.rs').write_bytes(source)
for name in ['gen_mround_half_boundaries.py','gen_mround_fraction_cutoff.py','gen_mround_heldout.py','gen_mround_typed.py','gen_mround_typed_heldout.py','gen_mround_reference_controls.py','gen_mround_reference_heldout.py','classify_reference_models.py']:
    shutil.copy2(Path('smart-fuzzer/tools/w111')/name,dst/name)
shutil.copy2('smart-fuzzer/tools/w111/complex_kernel_probe/src/bin/mround_half_graph_probe.rs',dst/'arithmetic-graph-probe.rs')
shutil.copy2('smart-fuzzer/tools/w111/complex_kernel_probe/src/bin/mround_staged_holdout.rs',dst/'staged-heldout-generator.rs')
for name,folder in [('half-boundary','w111-mround-half-boundaries-20260929'),
                    ('fraction-cutoff','w111-mround-fraction-cutoff-20260929'),
                    ('heldout','w111-mround-cutoff-heldout-20260929'),
                    ('staged-heldout','w111-mround-staged-heldout-20260929')]:
    src=Path('smart-fuzzer/runs')/folder
    shutil.copy2(src/'manifest.json',dst/(name+'-manifest.json'))
    if 'heldout' in name:shutil.copy2(src/'batch-mround.json',dst/(name+'-batch.json'))
shutil.copy2('smart-fuzzer/runs/w111-mround-prepared-typed-20260929/typed.json',dst/'prepared-discovery-input.json')
shutil.copy2('smart-fuzzer/runs/w111-mround-prepared-heldout-typed-20260929/typed.json',dst/'prepared-heldout-input.json')
shutil.copy2('smart-fuzzer/runs/w111-mround-prepared-typed-run-20260929/manifest.json',dst/'prepared-discovery-capture-manifest.json')
shutil.copy2('smart-fuzzer/runs/w111-mround-reference-heldout-typed-20260929/typed.json',dst/'reference-heldout-input.json')
files={p.name:dict(bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest())
       for p in dst.iterdir() if p.is_file() and p.name!='artifact-manifest.json'}
(dst/'artifact-manifest.json').write_text(json.dumps(files,indent=2))
print(json.dumps(dict(files=len(files),bytes=sum(f['bytes'] for f in files.values()))))
