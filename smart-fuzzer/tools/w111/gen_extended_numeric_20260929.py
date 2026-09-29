"""Second broad numeric discovery wave, using bounded domains and Value2 bits."""
import argparse, hashlib, json, math
from pathlib import Path
from gen_w111_broad_20260929 import Corpus, EDGE, bits, signed_log

def make(seed,n):
    c=Corpus(seed,n)
    edge=[v for v in EDGE if bits(v)!='0x8000000000000000' and not 0<abs(v)<2.0**-1022]
    for fn in ['SIN','COS','TAN','COT','SEC','CSC','SINH','COSH','COTH','SECH','CSCH','ATAN','ACOT','ASIN','ATANH','ACOSH','ACOTH','EXP','LN','LOG10','FISHER','SQRTPI','PHI']:
        c.add(fn,lambda r,i:(r.uniform(-2,2) if i%3==0 else r.uniform(-750,750) if i%3==1 else signed_log(r,-300,300),),[(x,) for x in edge])
    c.add('LOG',lambda r,i:(10**r.uniform(-300,300),r.choice([2,10,math.e,r.uniform(.01,100)])),[(x,b) for x in edge for b in [-1,0,.1,1,2,10]])
    c.add('POWER',lambda r,i:(r.uniform(-10,10),r.choice([r.randint(-100,100),r.uniform(-10,10)])),[(a,b) for a in [-10,-1,0,.1,1,10,1e100] for b in [-100,-2,-1,-.5,0,.5,1,2,100]])
    c.add('MROUND',lambda r,i:(r.uniform(-1e8,1e8),r.choice([0,.1,.3,1,2,10,r.uniform(-100,100)])),[(a,b) for a in [-1.5,-.5,0,.5,1.5,1e15] for b in [-10,-1,0,.1,1,10]])
    for fn in ['COMBIN','COMBINA']:
        c.add(fn,lambda r,i:(r.uniform(-2,500),r.uniform(-2,100)),[(a,b) for a in [-1,0,.9,1,2,170,500] for b in [-1,0,.9,1,2,170]])
    for fn in ['BESSELI','BESSELK']:
        c.add(fn,lambda r,i:(r.uniform(-1,50),r.uniform(-1,20)),[(a,b) for a in [-1,0,.001,1,10,100] for b in [-1,0,.9,1,2,10]])
    for fn in ['IMABS','IMAGINARY','IMARGUMENT','IMCONJUGATE','IMCOS','IMCOSH','IMCOT','IMCSC','IMCSCH','IMLN','IMLOG10','IMLOG2','IMREAL','IMSECH','IMSIN','IMSINH','IMSQRT','IMTAN']:
        c.add(fn,lambda r,i:(r.uniform(-100,100),),[(v,) for v in [-100,-1,-.1,0,.1,1,100,1e-300,1e100]])
    for fn in ['IMDIV','IMSUB','IMSUM','IMPRODUCT']:
        c.add(fn,lambda r,i:(r.uniform(-1e6,1e6),r.uniform(-1e6,1e6)),[(x,y) for x in [-1,0,.1,1] for y in [-1,0,.1,1]])
    for fn in ['DOLLARDE','DOLLARFR','EFFECT','NOMINAL']:
        c.add(fn,lambda r,i:(r.uniform(-1,100),r.uniform(-1,100)),[(x,y) for x in [-1,0,.1,1,10] for y in [-1,0,.1,1,2,12,100]])
    for fn in ['FV','PV']:
        c.add(fn,lambda r,i:(r.uniform(-.5,.5),r.uniform(-5,100),r.uniform(-1e4,1e4),r.uniform(-1e4,1e4),r.choice([0,1])),[(rate,period,-100,1000,ty) for rate in [-1,-.1,0,1e-10,.1] for period in [0,1,12] for ty in [0,1]])
    c.add('NPER',lambda r,i:(r.uniform(-.1,.5),r.uniform(-1e4,1e4),r.uniform(-1e5,1e5),r.uniform(-1e5,1e5),r.choice([0,1])),[(rate,-100,1000,0,ty) for rate in [-1,-.1,0,1e-10,.1] for ty in [0,1]])
    c.add('PDURATION',lambda r,i:(r.uniform(-.1,.5),r.uniform(-100,1e4),r.uniform(-100,1e4)),[(r,100,200) for r in [-1,0,.01,.1,1]])
    for fn in ['DB','DDB']:
        c.add(fn,lambda r,i:(r.uniform(1,1e6),r.uniform(0,1e5),r.uniform(.1,50),r.uniform(.1,50)),[(1000,100,10,p) for p in [0,.5,1,2,10,11]])
    c.add('VDB',lambda r,i:(r.uniform(1,1e6),r.uniform(0,1e5),r.uniform(.1,50),r.uniform(0,10),r.uniform(10,50)),[(1000,100,10,a,b) for a,b in [(0,1),(0,10),(1,2),(.5,1.5),(1,11)]])
    for fn in ['BINOM.INV','CRITBINOM']:
        c.add(fn,lambda r,i:(r.uniform(0,200),r.uniform(0,1),r.uniform(0,1)),[(n,p,a) for n in [0,1,10,100] for p in [0,.1,.5,1] for a in [0,.1,.5,1]])
    c.add('BINOM.DIST.RANGE',lambda r,i:(r.randrange(1,100),r.random(),r.randrange(0,100)),[(10,p,k) for p in [0,.1,.5,1] for k in [-1,0,1,9,10,11]])
    for fn in ['EXPON.DIST','EXPONDIST']:
        c.add(fn,lambda r,i:(r.uniform(-1,100),10**r.uniform(-5,5),r.choice([0,1])),[(x,l,cum) for x in [-1,0,.1,1,100] for l in [-1,0,.1,1] for cum in [0,1]])
    for fn in ['TRUE','FALSE','PI','NA']:
        c.batches[fn]=[()]
    return c

def main():
    p=argparse.ArgumentParser();p.add_argument('out',type=Path);p.add_argument('--seed',type=int,default=2026092903);p.add_argument('--rows',type=int,default=1200);a=p.parse_args()
    c=make(a.seed,a.rows);a.out.mkdir(parents=True,exist_ok=True)
    m={'generator':Path(__file__).name,'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'seed':a.seed,'random_rows_per_function':a.rows,'phase':'discovery','scope':'numeric_scalar_arguments_only','batches':[]}
    for fn,rows in c.batches.items():
        path=a.out/('batch-'+fn.lower()+'.json');doc={'function':fn,'probes':[{'probe':{'id':f'w111extended-{a.seed}-{fn}-{i:05d}','args':[bits(x) for x in row]}} for i,row in enumerate(rows)]}
        path.write_text(json.dumps(doc,separators=(',',':')),encoding='utf-8');m['batches'].append({'function':fn,'path':path.name,'rows':len(rows),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
    (a.out/'manifest.json').write_text(json.dumps(m,indent=2),encoding='utf-8');print('functions',len(m['batches']),'rows',sum(x['rows'] for x in m['batches']))
if __name__=='__main__':main()
