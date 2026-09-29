"""Compare frozen public-dispatch engines on retained numeric consumers.

Usage: audit_power_consumers.py OLD_EXE NEW_EXE OUT BANK...
No Excel capture or production mutation is performed. Every mismatch is
requested from both engines so equal aggregate counts cannot hide regressions.
"""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

old, new, out = map(Path, sys.argv[1:4])
out.mkdir(parents=True, exist_ok=True)
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
summary = {'old_executable': str(old), 'old_sha256': sha(old),
           'new_executable': str(new), 'new_sha256': sha(new), 'banks': []}
for i, name in enumerate(sys.argv[4:]):
    path = Path(name)
    source = json.loads(path.read_text(encoding='utf-8-sig'))
    reports = []
    for tag, exe in [('old',old), ('new',new)]:
        result = subprocess.run([str(exe.resolve()), 'judge-witnesses', str(path),
            '--max-misses', str(len(source['witnesses'])), '--json'],
            capture_output=True, text=True, check=True, timeout=120)
        report = json.loads(result.stdout)[0]
        (out/f'{i:03d}-{tag}.json').write_text(json.dumps(report, indent=2))
        reports.append(report)
    before, after = reports
    old_m = {r['id']:r for r in before['misses']}
    new_m = {r['id']:r for r in after['misses']}
    gained = sorted(old_m.keys()-new_m.keys())
    lost = sorted(new_m.keys()-old_m.keys())
    changed = [r for r in sorted(old_m.keys()&new_m.keys()) if old_m[r]['actual'] != new_m[r]['actual']]
    record = dict(path=str(path), sha256=sha(path), function=source['function'],
        rows=before['rows_judged'], old_exact=before['rows_agree'], new_exact=after['rows_agree'],
        newly_exact=[old_m[r] for r in gained], regressions=[new_m[r] for r in lost],
        changed_misses=[dict(id=r,before=old_m[r],after=new_m[r]) for r in changed])
    summary['banks'].append(record)
    (out/'summary.json').write_text(json.dumps(summary, indent=2))
    print(f'{source["function"]} {path.name}: {before["rows_agree"]}->{after["rows_agree"]}/{before["rows_judged"]}; +{len(gained)} -{len(lost)} changed misses {len(changed)}',flush=True)
