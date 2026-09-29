"""Retain exact root-owned packets without overwriting earlier differing data."""
import hashlib,json,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3];E=ROOT/'docs/function-lane/evidence/w111-broad-20260929'
def copy(p,q):
 q.parent.mkdir(parents=True,exist_ok=True)
 if q.exists():assert q.read_bytes()==p.read_bytes(),f'Immutable artifact differs: {q}'
 else:shutil.copyfile(p,q)
jobs=[
 ('integer-thresholds','integer-publication/thresholds',['int','permutationa','trunc','rounddown','roundup','round']),
 ('integer-hyperbolic-heldout','integer-publication/later-heldout',['int','trunc','rounddown','roundup','round']),
 ('trunc-hyperbolic','hyperbolic/discovery',['sinh','cosh','csch','tanh','coth']),
 ('integer-hyperbolic-heldout','hyperbolic/heldout',['sinh','cosh','csch','tanh','coth']),
 ('trunc-hyperbolic','integer-publication/digit-discovery',['trunc','rounddown'])]
for run,dest,names in jobs:
 source=ROOT/f'smart-fuzzer/runs/w111-{run}-20260929';target=E/dest
 manifest=json.loads((source/'capture_manifest.json').read_text());assert 'finished_utc' in manifest
 for name in names:
  for folder,filename in [('answers','answers-'+name+'.json'),('batches','batch-'+name+'.json'),('logs',name+'.log')]:copy(source/folder/filename,target/folder/filename)
 for name in ['capture_manifest.json','candidate-freeze.json','manifest.json']:
  if (source/name).exists():copy(source/name,target/name)
 if (source/'candidate').exists():
  for p in (source/'candidate').rglob('*'):
   if p.is_file():copy(p,target/'candidate'/p.relative_to(source/'candidate'))
 for p in [ROOT/'.tmp/w111-int-threshold-validation.json',ROOT/'.tmp/w111-integer-digit-candidates.json',ROOT/'.tmp/w111-integer-hyperbolic-heldout-judgement.json']:
  copy(p,E/'integer-publication/validation'/p.name)
for name in ['integer-publication','hyperbolic']:
 target=E/name;rows=[]
 for p in sorted(target.rglob('*')):
  if p.is_file() and p.name!='artifact-manifest.json':rows.append({'path':p.relative_to(target).as_posix(),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
 (target/'artifact-manifest.json').write_text(json.dumps({'schema_version':'w111.retained_boundary_artifacts.v1','artifacts':rows},indent=2))
 print(name,len(rows),'artifacts')
