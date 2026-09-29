"""Public-function probes separating the two expm1 halves of SINH."""
import hashlib,json,math,struct
from pathlib import Path
root=Path(__file__).resolve().parents[3]
source=root/'smart-fuzzer/runs/w111-tanh-dependencies-20260929/answers-sinh.json'
out=root/'smart-fuzzer/runs/w111-sinh-expm1-dependencies-20260929';out.mkdir(parents=True,exist_ok=True)
assert not (out/'manifest.json').exists()
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
decode=lambda s:struct.unpack('>d',bytes.fromhex(s[2:]))[0]
values=sorted(set(abs(decode(row['args'][0])) for row in json.loads(source.read_text(encoding='utf-8-sig'))['witnesses']))
manifest={'authority':'dependency_discovery_from_failed_SINH_inputs','source':source.relative_to(root).as_posix(),'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'batches':[]}
for fn in ['EXP','EXPON.DIST','GAMMA.DIST']:
    rows=[]
    for x in values:
        args=[(x,),(-x,)] if fn=='EXP' else [(x,1.,1.)] if fn=='EXPON.DIST' else [(x,1.,1.,1.)]
        rows.extend(tuple(map(bits,a)) for a in args)
    p=out/f'batch-{fn.lower()}.json'
    p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'w111sinhem1-{fn}-{i:05d}','args':a}} for i,a in enumerate(rows)]},separators=(',',':')))
    manifest['batches'].append({'function':fn,'rows':len(rows),'path':p.name,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));print([(r['function'],r['rows']) for r in manifest['batches']])
