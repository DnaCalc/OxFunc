"""Black-box discovery of digit-conversion edges and hyperbolic overflow."""
import hashlib,json,math,random,struct
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/runs/w111-trunc-hyperbolic-20260929';batch=out/'batches';batch.mkdir(parents=True,exist_ok=True)
rng=random.Random(202609292612);bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
decode=lambda x:struct.unpack('>d',int(x).to_bytes(8,'big'))[0]
manifest={'phase':'discovery_not_holdout','seed':202609292612,'batches':[]}
def save(fn,rows):
 rows=list(dict.fromkeys(tuple(map(bits,row)) for row in rows if all(math.isfinite(x) and (x==0 or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000' for x in row)))
 p=batch/('batch-'+fn.lower()+'.json');assert not p.exists();p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'w111trunchyp-{fn}-{i:06d}','args':row}} for i,row in enumerate(rows)]},separators=(',',':')))
 manifest['batches'].append({'function':fn,'path':p.name,'rows':len(rows),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
digit=[]
for center in [2**31-1,2**31,2**31+1,2**32,2**63,1e100]:
 b=int(bits(center),16)
 for delta in range(-40,41):
  for sign in [-1,1]:digit.append(sign*decode(b+delta))
for offset in range(-64,65):digit.extend([2**31+offset,-2**31+offset])
rows=[(n,d) for n in [1.125,-1.125,12.875,-12.875,0.1234567890123456,123456789012345.5] for d in digit]
save('TRUNC',rows);save('ROUNDDOWN',rows)
values=[]
for center in [0.,1.,.5,20.,350.,354.,355.,700.,709.,709.782712893384,710.,710.475860073944,711.,730.,800.,1e100]:
 b=int(bits(center),16)
 for delta in range(-16,17):
  if b+delta<=0:continue
  for sign in [-1,1]:values.append(sign*decode(b+delta))
values.extend(rng.uniform(-750,750) for _ in range(5000))
values.extend(decode((rng.randrange(2)<<63)|(rng.randrange(1,2047)<<52)|rng.getrandbits(52)) for _ in range(3000))
for fn in ['TANH','COTH','SINH','COSH','CSCH']:save(fn,[(x,) for x in values])
(batch/'manifest.json').write_text(json.dumps(manifest,indent=2));print(sum(b['rows'] for b in manifest['batches']),'fresh discovery rows')
