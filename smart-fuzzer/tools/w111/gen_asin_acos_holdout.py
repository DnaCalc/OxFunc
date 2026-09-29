"""Second independent inverse-angle packet, including final-subtraction midpoints."""
import hashlib,json,struct,subprocess,sys
from pathlib import Path

out=Path(sys.argv[1]);seed=int(sys.argv[2]) if len(sys.argv)>2 else 2026092924
subprocess.run([sys.executable,str(Path(__file__).with_name('gen_asin_holdout.py')),str(out),str(seed)],check=True)
bits=lambda x:struct.unpack('<Q',struct.pack('<d',x))[0]
number=lambda n:struct.unpack('<d',struct.pack('<Q',n))[0]
p=out/'batch-asin.json';packet=json.loads(p.read_text())
seen={int(r['probe']['args'][0],16) for r in packet['probes']}
for k in range(32):
    midpoint=(2*k+1)*2.0**-53
    # Half an extended-format ULP around each binary64 publication midpoint.
    for shift in [-2.0**-64,0.0,2.0**-64]:
        center=bits(midpoint+shift)
        for neighbor in range(-3,4):
            for sign in [0,1]:
                raw=(center+neighbor)|(sign<<63)
                if raw in seen:continue
                seen.add(raw)
                packet['probes'].append(dict(probe=dict(id=f'asin-heldout-{seed}-{len(seen):05d}-final-subtraction-midpoint',args=[f'0x{raw:016x}'])))
p.write_text(json.dumps(packet,indent=2))
manifest=json.loads((out/'manifest.json').read_text());manifest['generator']=Path(__file__).name
manifest['final_subtraction_midpoints']=32
manifest['batches']=[dict(function='ASIN',path=p.name,rows=len(packet['probes']),sha256=hashlib.sha256(p.read_bytes()).hexdigest())]
packet['function']='ACOS'
for row in packet['probes']:row['probe']['id']=row['probe']['id'].replace('asin-heldout-','acos-heldout-')
p=out/'batch-acos.json';p.write_text(json.dumps(packet,indent=2))
manifest['batches'].append(dict(function='ACOS',path=p.name,rows=len(packet['probes']),sha256=hashlib.sha256(p.read_bytes()).hexdigest()))
(out/'manifest.json').write_text(json.dumps(manifest,indent=2))
print('Combined rows:',2*len(packet['probes']))
