"""Black-box date conversion discriminators and repeated WEEKNUM selectors."""
import hashlib,itertools,json,math,random,sys
from pathlib import Path
from gen_w111_broad_20260929 import bits
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
m={'generator':Path(__file__).name,'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'phase':'discovery','seed':2026092908,'batches':[]}
def save(fn,rows):
 path=out/('batch-'+fn.lower()+'.json');rows=list(rows)
 doc={'function':fn,'probes':[{'probe':{'id':f'date-conversion-{fn}-{i:05d}','args':[bits(x) for x in row]}} for i,row in enumerate(rows)]}
 path.write_text(json.dumps(doc,separators=(',',':')),encoding='utf-8')
 m['batches'].append({'function':fn,'path':path.name,'rows':len(rows),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
xs=set()
for day,sec in itertools.product([0,1,58,59,60,366,43831,1048576,2958465],[0,1,59,60,3599,3600,86399]):
 for offset in [-.50001,-.5,-.49999,0,.49999,.5,.50001]:
  x=day+(sec+offset)/86400
  xs.update([x,math.nextafter(x,-math.inf),math.nextafter(x,math.inf)])
xs={x for x in xs if x==0 or abs(x)>=2**-1022}
for fn in ['DAY','MONTH','YEAR','HOUR','MINUTE','SECOND','ISOWEEKNUM']:save(fn,[(x,) for x in sorted(xs)])
save('WEEKDAY',[(x,t) for x in sorted(xs) for t in [1,2,3,11,17]])
edges=[-1e300,-2**31-1,-2**31,-2**31+1,-2**16-1,-2**16,-2**15-1,-2**15,-2**15+1,-1901,-1900,-1899,-1,0,1,1899,1900,9999,10000,2**15-1,2**15,2**15+1,2**16-1,2**16,2**16+1,2**31-1,2**31,2**31+1,1e300]
rows=[]
for axis in range(3):
 for x,delta in itertools.product(edges,[-.9,0,.9]):
  a=[1900,1,1];a[axis]=x+delta;rows.append(a)
for y,mn,d in itertools.product([-.99999999,-1,0,1899,1900,9999,10000],[0,1,2,3,12,13],[0,1,29,30,32766,32767,32768,-32768,-32769]):rows.append([y,mn,d])
for axis in range(3):
 for x in [0,1,2,1900,32767]:
  for direction in [-math.inf,math.inf]:
   a=[1900,1,1];a[axis]=math.nextafter(float(x),direction)
   if a[axis]==0 or abs(a[axis])>=2**-1022:rows.append(a)
save('DATE',rows)
rows=[(s,t) for s,t in itertools.product([0,1,59,60,61,43831,2958465],range(-5,130))]
rows=rows*3;random.Random(m['seed']).shuffle(rows)
save('WEEKNUM',rows)
(out/'manifest.json').write_text(json.dumps(m,indent=2),encoding='utf-8')
print(len(m['batches']),sum(x['rows'] for x in m['batches']))
