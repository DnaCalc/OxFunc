"""Select normal-source DB final-year stage publication discriminators."""
import hashlib,json,math,random,struct
from pathlib import Path
root=Path(__file__).resolve().parents[3]
out=root/'smart-fuzzer/cache/w111-depreciation-20260929'
rng=random.Random(202609291859)
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
minimum=2**-1022
rows={};counts={'quotient-subnormal-product-recovers':0,'product-subnormal-final-recovers':0}
for _ in range(200000):
 cost=minimum*2**rng.uniform(0,8);life=rng.randrange(1,10)
 rate=rng.choice([-.125,-.4,-.7,-1.,-1.5,-3.,.1,.3,.8]);month=rng.randrange(1,12)
 salvage=cost*(1-rate)**life
 if salvage<minimum:continue
 amount=cost*rate*(month/12)
 if abs(amount)<minimum:amount=0.
 book=cost-amount
 for year in range(2,life+1):
  amount=book*rate
  if abs(amount)<minimum:amount=0.
  book-=amount
 quotient=book/12;product=quotient*rate;final=product*(12-month)
 if abs(quotient)<minimum and abs(product)>=minimum:tag='quotient-subnormal-product-recovers'
 elif abs(quotient)>=minimum and abs(product)<minimum and abs(final)>=minimum:tag='product-subnormal-final-recovers'
 else:continue
 if counts[tag]>=150:continue
 args=tuple(map(bits,[cost,salvage,float(life),life+1.,float(month)]))
 if args not in rows:rows[args]=tag;counts[tag]+=1
 if min(counts.values())>=150:break
path=out/'refinement9-db.json'
path.write_text(json.dumps(dict(function='DB',probes=[dict(probe=dict(id=f'w111db-final-publish-{i:04d}',args=list(a)),probe_region=tag) for i,(a,tag) in enumerate(rows.items())]),indent=2)+'\n',encoding='utf-8')
(out/'refinement9-db-manifest.json').write_text(json.dumps(dict(rows=len(rows),counts=counts,generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),batch_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),classification='Discovery selected to distinguish internal quotient/product publication. All source values normal or positive zero.'),indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(path=str(path),rows=len(rows),counts=counts)))
