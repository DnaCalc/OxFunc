"""Serially capture generated ProbeBatches through the existing live Value2 oracle.

Retains individual oracle logs and a source/profile/hash manifest. Never retries a
failed capture automatically and never treats an existing answer as a fresh run.
Usage: python run_numeric_campaign.py RUN_DIRECTORY
"""
import hashlib
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parents[3]
run = Path(sys.argv[1]).resolve()
answers, logs = run / 'answers', run / 'logs'
answers.mkdir(parents=True, exist_ok=True)
logs.mkdir(parents=True, exist_ok=True)
manifest = json.loads((run / 'batches/manifest.json').read_text(encoding='utf-8'))
manifest.update(started_utc=datetime.now(timezone.utc).isoformat(),
                source_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),
                source_status=subprocess.check_output(['git','status','--short'],cwd=root,text=True),
                runner='smart-fuzzer/tools/Run-W109BulkBatch.ps1', cache='disabled', captures=[])
def save():
    (run / 'capture_manifest.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
save()
for item in manifest['batches']:
    fn = item['function']
    answer = answers / ('answers-'+fn.lower()+'.json')
    if answer.exists():
        raise SystemExit(f'Refusing to overwrite capture: {answer}')
    log = logs / (fn.lower()+'.log')
    with log.open('w',encoding='utf-8') as stream:
        result = subprocess.run(['pwsh','-NoProfile','-File',str(root / manifest['runner']),
             '-Batch',str(run / 'batches' / item['path']),'-Out',str(answer),'-NoCache'],
             cwd=root,stdout=stream,stderr=subprocess.STDOUT)
    if result.returncode:
        manifest['failure']={'function':fn,'exit_code':result.returncode,'log':str(log)}
        save()
        raise SystemExit(f'Capture failed: {fn}; see {log}')
    data=json.loads(answer.read_text(encoding='utf-8-sig'))
    if len(data['witnesses']) != item['rows']:
        raise SystemExit(f'Wrong row count for {fn}')
    manifest['captures'].append(dict(function=fn,rows=len(data['witnesses']),
        sha256=hashlib.sha256(answer.read_bytes()).hexdigest(),
        provenance=data['capture_provenance']))
    save()
    print(f'{len(manifest["captures"])}/{len(manifest["batches"])} {fn}: {len(data["witnesses"])} live answers',flush=True)
manifest['finished_utc']=datetime.now(timezone.utc).isoformat()
save()
