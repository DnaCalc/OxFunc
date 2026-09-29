"""Discovery discriminators for integer rounding, parity tolerance and TRUNC."""
import hashlib,json,math,random,struct
from pathlib import Path
import gen_broad_typed_20260929 as g
out=g.ROOT/'smart-fuzzer/runs/w111-integer-refinement-20260929';batch=out/'batches';batch.mkdir(parents=True,exist_ok=True)
rng=random.Random(202609292511);bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
decode=lambda x:struct.unpack('>d',int(x).to_bytes(8,'big'))[0]
safe=lambda x:math.isfinite(x) and (x==0 or abs(x)>=2**-1022) and bits(x)!='0x8000000000000000'
manifest={'phase':'discovery_discriminators_not_holdout','seed':202609292511,'batches':[]}
def save(fn,rows):
 rows=list(dict.fromkeys(tuple(map(bits,row)) for row in rows if all(safe(x) for x in row)))
 p=batch/('batch-'+fn.lower()+'.json');assert not p.exists()
 p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'integer-refinement-{fn}-{i:05d}','args':r}} for i,r in enumerate(rows)]},separators=(',',':')))
 manifest['batches'].append({'function':fn,'path':p.name,'rows':len(rows),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
ints=[];parity=[]
for _ in range(400):
 n=rng.randrange(1,10**rng.randrange(1,16))
 for sign in [-1,1]:
  for offset in [-.5,-.25,0,.25,.5]:
   b=int(bits(float(n)+offset),16)
   for delta in range(-3,4):ints.append((sign*decode(b+delta),))
for n in [1,2,3,17,100,9485,2**20,2**30,2**32,2**53-1,2**53]:
 for epsilon in [1e-10,1e-10/2,1e-10*2]:
  b=int(bits(n-epsilon),16)
  for delta in range(-180,181):
   for sign in [-1,1]:
    x=sign*decode(b+delta);parity.append((x,))
for _ in range(1000):
 x=decode((rng.randrange(2)<<63)|(rng.randrange(1,2047)<<52)|rng.getrandbits(52));ints.append((x,));parity.append((x,))
save('INT',ints);save('ISEVEN',parity);save('ISODD',parity)
trunc=[]
for n in [-1e308,-1e16,-283025097725723.5,-1.9999999999999998,-1.005,-.1,0,.1,.9999999999999999,1.005,283025097725723.5,1e16,1e308]:
 for d in [-1e100,-2**32,-2**31,-32769,-32768,-309,-308,-16,-15,-1,-.9,0,.9,1,15,16,308,309,32767,32768,2**31-1,2**31,2**32,1e100]:trunc.append((n,d))
for _ in range(3000):
 x=decode((rng.randrange(2)<<63)|(rng.randrange(1,2047)<<52)|rng.getrandbits(52));trunc.append((x,rng.randrange(-330,331)+rng.choice([0,.25,.75])))
save('TRUNC',trunc);save('ROUNDDOWN',trunc)
(batch/'manifest.json').write_text(json.dumps(manifest,indent=2));print(sum(x['rows'] for x in manifest['batches']),'discriminators')
