"""Date serial, rollover and selector discovery; every numeric input uses bits."""
import hashlib,json,math,sys
from pathlib import Path
from gen_w111_broad_20260929 import bits
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
serials=[-1e300,-2**31,-365,-1,-.999,-.5,-.001,-1e-300,0,1e-300,.5,1,31,59,60,61,365,366,367,1462,2958464,2958465,2958466,2**31,1e300]
serials+= [math.nextafter(float(v),d) for v in [0,1,59,60,61,2958465,2958466] for d in [-math.inf,math.inf] if not (0<abs(math.nextafter(float(v),d))<2**-1022)]
serials+= [2958465+x for x in [.0001,.5,.999,.99999999]]
m={'generator':Path(__file__).name,'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'phase':'discovery','seed':20260929,'batches':[]}
def save(fn,rows):
    path=out/('batch-'+fn.lower()+'.json');rows=list(rows)
    path.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'date-boundary-{fn}-{i:05d}','args':[bits(x) for x in row]}} for i,row in enumerate(rows)]}),encoding='utf-8')
    m['batches'].append({'function':fn,'path':path.name,'rows':len(rows),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
for fn in ['DAY','MONTH','YEAR','HOUR','MINUTE','SECOND','ISOWEEKNUM']:save(fn,[(x,) for x in serials])
for fn in ['WEEKDAY','WEEKNUM']:
    save(fn,[(x,t) for x in serials for t in [-1,0,1,2,3,10,11,17,18,20,21,22,23,27,28,99,127,128,255,256,257,2**31]]+
         [(x,t) for x in [0,1,59,60,61,365,366,367,43830,43831,44196,44197,2958465] for t in range(-5,130)]+
         [(43831,t+.9999999) for t in range(1,32)])
for fn in ['EDATE','EOMONTH']:
    save(fn,[(x,m) for x in serials for m in [-1e300,-2**31,-120001,-1.9,-1,-.9,0,.9,1,1.9,12,120001,2**31,1e300]])
save('DAYS',[(a,b) for a in serials for b in [0,-.1,-1,60,2958465,2958465.9,2958466]])
save('DATE',[(y,m,d) for y in [-1901,-1900,-1899,-2,-1,0,1,1899,1900,1901,9999,10000] for m in [-25,-13,-12,-1,0,1,2,3,12,13,25] for d in [-366,-307,-306,-31,-30,-1,0,1,28,29,30,31,32,366]]+
     [(y,m,d) for y in [0,1900,2000,-2**31,2**31,1e300,-1e300] for m in [1,1e300,-1e300,2**31,-2**31] for d in [1,1e300,-1e300,2**31,-2**31]])
(out/'manifest.json').write_text(json.dumps(m,indent=2),encoding='utf-8')
print(len(m['batches']),sum(x['rows'] for x in m['batches']))
