"""Fresh numeric distributions after the arithmetic/domain candidate freeze."""
import hashlib,itertools,json,math,random,struct,subprocess,sys
from pathlib import Path

SEED=2026092930;rng=random.Random(SEED);out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
bits=lambda x:f'0x{struct.unpack("<Q",struct.pack("<d",float(x)))[0]:016x}'
subprocess.run([sys.executable,str(Path(__file__).with_name('gen_distribution_density_holdout.py')),str(out),str(SEED)],check=True)
p=out/'batch-norm.dist.json';density=json.loads(p.read_text())
search=json.loads(subprocess.check_output(['target/debug/normal_density_normalization_search.exe',str(SEED)],text=True))
(out/'mathematical-normalization-discriminators.json').write_text(json.dumps(search,indent=2))
for r in search['selected']:density['probes'].append(dict(probe=dict(id='fresh-'+r['id'],args=r['args'])))
for k in [-800,-455,-27,-3,3,37,510,900]:
 x=2.**k
 for offset,s in itertools.product([-7,-3,3,7],[.7,1.1,2.3]):
  density['probes'].append(dict(probe=dict(id=f'fresh-subtraction-{len(density["probes"])}',args=list(map(bits,[x,-x*(2**-53+offset*2**-65),x*s,0.])))))
p.write_text(json.dumps(density,indent=2))
packets={}
def emit(fn,args,kind):
 rows=packets.setdefault(fn,[]);rows.append(dict(probe=dict(id=f'fresh-{fn.lower()}-{len(rows):05d}-{kind}',args=list(map(bits,args)))))
edges=[-2**-52,0.,math.nextafter(sys.float_info.min,math.inf),2**-51,.125,.375,.75,math.nextafter(1.,0.),1.,math.nextafter(1.,math.inf)]
for n,p,alpha in itertools.product([3,4,9,13,23],edges,edges):emit('BINOM.INV',[n,p,alpha],'fresh-endpoints')
emit('BINOM.INV',[6,.5,0],'old-seed-exact-input')
for f,s,p,flag in itertools.product([3,4,11],[3,7,11],edges,[0,1]):emit('NEGBINOM.DIST',[f,s,p,flag],'fresh-endpoints')
emit('NEGBINOM.DIST',[0,2,1,0],'old-seed-exact-input')
for N in [7,8,9,13,31]:
 for _ in range(100):
  n=rng.choice([-.9,-.1,0.,.1,1.5,N-.75,float(N),N+.9,float(N+1)])
  K=rng.choice([-.1,0.,1.,N/2,N-.1,float(N),N+1.])
  k=rng.choice([-.1,0.,1.,min(n,K),max(0,n+K-N),min(n,K)+.5,min(n,K)+1])
  for flag in [0,1]:emit('HYPGEOM.DIST',[k,n,K,N,flag],'fractional-support-controls')
emit('HYPGEOM.DIST',[4,5,2,10,0],'old-seed-exact-input')
for fn,rows in packets.items():(out/f'batch-{fn.lower()}.json').write_text(json.dumps(dict(function=fn,probes=rows),indent=2))
manifest=dict(generator=Path(__file__).name,seed=SEED,authority='independent_after_production_density_and_domain_freeze',batches=[])
for p in sorted(out.glob('batch-*.json')):
 d=json.loads(p.read_text());manifest['batches'].append(dict(function=d['function'],path=p.name,rows=len(d['probes']),sha256=hashlib.sha256(p.read_bytes()).hexdigest()))
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));print(json.dumps(manifest))
