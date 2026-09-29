"""Fresh normal-density arithmetic controls plus one existing NEGBINOM pin."""
import hashlib,json,math,random,struct,sys
from pathlib import Path

SEED=int(sys.argv[2]) if len(sys.argv)>2 else 2026092926;rng=random.Random(SEED)
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
bits=lambda x:f'0x{struct.unpack("<Q",struct.pack("<d",x))[0]:016x}'
num=lambda u:struct.unpack('<d',struct.pack('<Q',u))[0]
rows={}
def add(x,mean,sigma,kind):
    args=[x,mean,sigma,0.0]
    if all(math.isfinite(v) and (v==0 or abs(v)>=sys.float_info.min) for v in args):
        rows.setdefault(tuple(map(bits,args)),kind)
for _ in range(4000):
    sigma=math.ldexp(1+rng.random(),rng.randint(-950,950));mean=math.ldexp(rng.uniform(-2,2),rng.randint(-950,950))
    z=rng.uniform(-40,40);add(mean+z*sigma,mean,sigma,'fresh-scaled-density')
for _ in range(1500):
    raw=lambda: num((rng.randrange(1,2047)<<52)|rng.getrandbits(52)|(rng.getrandbits(1)<<63))
    add(raw(),raw(),abs(raw()),'fresh-full-bit-triple')
for sigma in [sys.float_info.min,1e-300,1e-200,1e-50,.1,.3,.75,1.,1.1,3.,1e50,1e200,1e300,sys.float_info.max]:
    for z in [0.,2**-26,.25,1.,math.sqrt(2),8.,26.,37.,38.,38.5,39.,40.]:
        for sign in [-1,1]:
            center=sign*z
            for neighbor in [math.nextafter(center,-math.inf),center,math.nextafter(center,math.inf)]:
                add(neighbor*sigma,0.,sigma,'square-exp-underflow-boundary')
for mean in [2**k for k in [-900,-54,-1,0,1,52,900]]:
    for dx in [2**-53,2**-52,.5,1.,2.,3.]:
        for s in [.1,.7,1.,3.,7.]:
            add(-mean*dx,mean,s,'mean-subtraction-rounding')
probes=[dict(probe=dict(id=f'norm-density-{SEED}-{i:05d}-{kind}',args=list(args))) for i,(args,kind) in enumerate(rows.items())]
packets={'NORM.DIST':probes}
controls=[]
for delta in [-2,-1,0,1,2]:
    p=num(int(bits(.4),16)+delta)
    for failures in [4.,5.,6.]:
        for flag in [0.,1.]:
            controls.append(dict(probe=dict(id=f'negbinom-pin-{len(controls):03d}',args=list(map(bits,[failures,3.,p,flag])))))
packets['NEGBINOM.DIST']=controls
manifest=dict(generator=Path(__file__).name,seed=SEED,authority='independent_after_density_operation_graph_freeze',batches=[])
for fn,probes in packets.items():
    p=out/f'batch-{fn.lower()}.json';p.write_text(json.dumps(dict(function=fn,probes=probes),indent=2))
    manifest['batches'].append(dict(function=fn,path=p.name,rows=len(probes),sha256=hashlib.sha256(p.read_bytes()).hexdigest()))
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));print(json.dumps(manifest))
