"""POISSON zero-count domain and fresh HYPGEOM raw-negative controls."""
import itertools,json,math,random,struct,sys
from pathlib import Path

SEED=2026092932;rng=random.Random(SEED);out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
bits=lambda x:f'0x{struct.unpack("<Q",struct.pack("<d",float(x)))[0]:016x}'
packets={}
def emit(fn,args,kind):
 rows=packets.setdefault(fn,[]);rows.append(dict(probe=dict(id=f'{fn.lower()}-branch-{len(rows):05d}-{kind}',args=list(map(bits,args)))))
for x,mean,flag in itertools.product([-2.,-1.,-.9,-.5,-sys.float_info.min,0.,sys.float_info.min,.1,.9,1.,1.1,2.,3.],[-1000.,-710.,-709.,-5.,-1.,-.1,0.,.1,1.,5.,710.,1000.],[0,1]):
 emit('POISSON.DIST',[x,mean,flag],'count-mean-order')
for _ in range(240):
 N=rng.randrange(2,49);draws=rng.randrange(0,N+1)+rng.choice([0.,.25,.75]);K=rng.randrange(0,N+1)+rng.choice([0.,.25,.75]);k=rng.randrange(0,N+2)+rng.choice([0.,.25,.75])
 args=[k,draws,K,N];axis=rng.randrange(4)
 if rng.randrange(2):args[axis]=-rng.choice([sys.float_info.min,2**-40,.25,.75])
 for flag in [0,1]:emit('HYPGEOM.DIST',args+[flag],'fresh-raw-negative')
manifest=dict(generator=Path(__file__).name,seed=SEED,authority='poisson_branch_discovery_and_hypergeometric_refinement_holdout',batches=[])
for fn,rows in packets.items():
 p=out/f'batch-{fn.lower()}.json';p.write_text(json.dumps(dict(function=fn,probes=rows),indent=2));manifest['batches'].append(dict(function=fn,path=p.name,rows=len(rows)))
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));print(json.dumps(manifest))
