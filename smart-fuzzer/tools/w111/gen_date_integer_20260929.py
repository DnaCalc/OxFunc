"""Distinguish DATE and weekday integer conversion using exposed output changes."""
import hashlib,json,math,random,sys
from pathlib import Path
from gen_w111_broad_20260929 import bits
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
manifest={'generator':Path(__file__).name,'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'phase':'discovery','seed':2026092910,'batches':[]}
def save(fn,rows):
 path=out/('batch-'+fn.lower()+'.json');rows=list(dict.fromkeys(tuple(row) for row in rows))
 path.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'date-integer-{fn}-{i:05d}','args':[bits(x) for x in row]}} for i,row in enumerate(rows)]},separators=(',',':')),encoding='utf-8')
 manifest['batches'].append({'function':fn,'path':path.name,'rows':len(rows),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
rows=[]
for k in range(1,1024):
 x=-math.ldexp(1.0,k)
 rows.extend([(x,1,1),(math.nextafter(x,0),1,1),(math.nextafter(x,-math.inf),1,1)])
for bound in [-2**31,-2**30,-2**16,-32768,-32767,-1900,-1,0,1,2,1900,32766,32767,32768,2**31]:
 for delta in [-1,-.9,-.5,-.1,-2**-20,-2**-22,0,2**-22,.1,.5,.9,1]:
  x=bound+delta
  rows.extend([(1900,x,1),(5000,1,x),(x,1,1)])
for i in [-32767,-1900,-2,-1,0,1,2,1900,32766,32767]:
 for e in range(-36,-16):
  for mult in [-1.01,-1,-.99,0,.99,1,1.01]:
   x=i+mult*2**e
   rows.extend([(5000,1,x),(1900,x,1),(x,1,1)])
save('DATE',rows)
selectors=set()
for n in range(-2,24):
 for power in range(-36,-16):
  for mult in [-1.01,-1,-.99,0,.99,1,1.01]:selectors.add(n+mult*2**power)
save('WEEKDAY',[(43831,x) for x in sorted(selectors)])
save('WEEKNUM',[(43831,x) for x in sorted(selectors) if math.floor(x) in [0,1,2,10,11,12,13,14,15,16,17,20,21]])
(out/'manifest.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
print(sum(x['rows'] for x in manifest['batches']))
