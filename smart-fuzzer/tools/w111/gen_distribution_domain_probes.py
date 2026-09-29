"""Public boundary controls for observed discrete distribution domain gaps."""
import itertools,json,math,struct,sys
from pathlib import Path

out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
bits=lambda x:f'0x{struct.unpack("<Q",struct.pack("<d",float(x)))[0]:016x}'
edges=[-math.ulp(1.),0.,sys.float_info.min,math.ulp(1.),.25,.5,math.nextafter(1.,0.),1.,math.nextafter(1.,math.inf)]
packets={}
def emit(fn,args,kind):
 rows=packets.setdefault(fn,[]);rows.append(dict(probe=dict(id=f'{fn.lower()}-domain-{len(rows):05d}-{kind}',args=list(map(bits,args)))))
for n in [0,1,2,7,12]:
 for p,alpha in itertools.product(edges,repeat=2):emit('BINOM.INV',[n,p,alpha],'unit-endpoints')
for failures,successes,p,flag in itertools.product([0,1,2,7],[0,1,2,5],edges,[0,1]):emit('NEGBINOM.DIST',[failures,successes,p,flag],'unit-endpoints')
for population in range(0,7):
 for sample,pop_success in itertools.product(range(-1,population+2),repeat=2):
  for successes in sorted(set([-1,0,1,min(sample,pop_success),max(0,sample+pop_success-population),sample+1])):
   for flag in [0,1]:emit('HYPGEOM.DIST',[successes,sample,pop_success,population,flag],'integer-support')
manifest=dict(generator=Path(__file__).name,authority='discrete_domain_discovery_before_guard_changes',batches=[])
for fn,rows in packets.items():
 p=out/f'batch-{fn.lower()}.json';p.write_text(json.dumps(dict(function=fn,probes=rows),indent=2));manifest['batches'].append(dict(function=fn,path=p.name,rows=len(rows)))
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));print(json.dumps(manifest))
