"""Fresh ASIN paired-sign samples plus mathematically selected angle disagreements.

The optional discriminator file is produced by the frozen research program
from fresh bit patterns without consulting Excel answers.
"""
import hashlib,json,math,random,struct,subprocess,sys
from pathlib import Path

SEED=int(sys.argv[2]) if len(sys.argv)>2 else 2026092923
rng=random.Random(SEED)
bits=lambda x:struct.unpack('<Q',struct.pack('<d',x))[0]
number=lambda x:struct.unpack('<d',struct.pack('<Q',x))[0]
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
search=[]
for i in range(300000):
    raw=((1022<<52)|rng.getrandbits(52))|(rng.getrandbits(1)<<63)
    search.append(dict(id=f'angle-search-{i}',args=[f'0x{raw:016x}']))
p=out/'angle-search-inputs.json';p.write_text(json.dumps(dict(function='ASIN',witnesses=search)))
result=subprocess.check_output(['target/debug/asin_probe.exe',str(p),'--find-angle-differences'],text=True)
discriminators=json.loads(result);(out/'angle-discriminators.json').write_text(json.dumps(discriminators,indent=2))
values={}
def add(raw,kind):
    magnitude=raw&((1<<63)-1)
    if magnitude<=0x7fefffffffffffff and (magnitude==0 or magnitude>=0x0010000000000000):
        for sign in [0,1]:values.setdefault(magnitude|(sign<<63),kind)
for _ in range(2000):add(bits(rng.random()),'fresh-uniform-pair')
for _ in range(1000):add((rng.randint(1,1022)<<52)|rng.getrandbits(52),'fresh-scaled-pair')
for center in discriminators['centers']:
    magnitude=int(center['input'],16)&((1<<63)-1)
    for k in range(-3,4):add(magnitude+k,'direct-reduced-angle-discriminator')
for x in [0.,sys.float_info.min,2**-53,2**-27,.25,.5,math.sqrt(.5),.75,1.]:
    for k in range(-9,10):add(bits(x)+k,'boundary-neighbor')
rows=[dict(probe=dict(id=f'asin-heldout-{SEED}-{i:05d}-{kind}',args=[f'0x{raw:016x}']))
      for i,(raw,kind) in enumerate(values.items())]
p=out/'batch-asin.json';p.write_text(json.dumps(dict(function='ASIN',probes=rows),indent=2))
(out/'manifest.json').write_text(json.dumps(dict(generator=Path(__file__).name,seed=SEED,
 authority='independent_exploration_input',angle_search_draws=300000,angle_centers=len(discriminators['centers']),
 batches=[dict(function='ASIN',path=p.name,rows=len(rows),sha256=hashlib.sha256(p.read_bytes()).hexdigest())]),indent=2))
print(len(rows),'rows;',len(discriminators['centers']),'fresh arithmetic discriminator centers ->',out)
