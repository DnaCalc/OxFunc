"""Independent composed-name ADDRESS validation after character grammar freeze."""
import hashlib
import json
import random
from pathlib import Path
from gen_address_probe_20260929 import bits

rng=random.Random(202609291337)
root=Path('smart-fuzzer/runs/w111-broad-20260929')
classified=json.loads(Path('docs/function-lane/evidence/w111-broad-20260929/address/name-classes.json').read_text())
def choose_class(position,allowed=True):
    if allowed:
        lo,hi=rng.choice(classified['ranges'][position]);return chr(rng.randint(lo,hi))
    ranges=classified['ranges'][position]
    while True:
        cp=rng.randint(256,65535)
        if 0xd800<=cp<=0xdfff:continue
        if not any(lo<=cp<=hi for lo,hi in ranges):return chr(cp)

names=[]
for i in range(2048):
    size=rng.choice([1,2,3,4,8,16,32])
    chars=[choose_class('start')]+[choose_class('middle') for _ in range(size-1)]
    if i%3==0:
        j=rng.randrange(size);chars[j]=choose_class('start' if j==0 else 'middle',False)
    name=''.join(chars)
    if i%8==0:
        # A book token uses continuation characters even in its first slot.
        name='['+choose_class('middle')+choose_class('middle')+']'+name
    if i%16==1:name=choose_class('start')+'['+name+']'
    names.append(name)
for i in range(1536):
    size=rng.randint(230,260)
    alphabet=[choose_class('middle') for _ in range(8)]+["'",' ','x','?']
    name=choose_class('start')+''.join(rng.choice(alphabet) for _ in range(size-1))
    names.append(name)
for i in range(512):
    row=rng.choice(['','0','1','01','1048576','1048577',str(rng.randrange(2000000))])
    col=rng.choice(['','0','1','01','16384','16385',str(rng.randrange(2000000))])
    name=rng.choice(['R'+row+'C'+col,'C'+col+'R'+row])+rng.choice(['','x','foo','_','?','.'])
    if i%3==0:name='[Book]'+name
    names.append(name)
probes=[]
for i,name in enumerate(names):
    style=rng.choice([0,1]);mode=rng.randint(1,4)
    row=rng.choice([1,2,10,999,1048575,1048576])
    col=rng.choice([1,2,26,27,702,703,16383,16384])
    if style==0 and mode in [3,4]:row=rng.choice([-1048575,-100,-1,0,1,100,1048575])
    if style==0 and mode in [2,4]:col=rng.choice([-16383,-100,-1,0,1,100,16383])
    probes.append({'probe':{'id':f'address-grammar-validation-202609291337-{i:05d}',
                            'args':[bits(row),bits(col),bits(mode),bits(style),name]}})
out=root/'address-grammar-validation.json'
out.write_text(json.dumps({'function':'ADDRESS','probes':probes},ensure_ascii=False,separators=(',',':')),encoding='utf-8')
files=['crates/oxfunc_core/src/functions/reference_metadata_family.rs',
       'crates/oxfunc_core/src/functions/address_name_classes.rs',str(out)]
freeze={'seed':202609291337,'state':'character_class_and_composition_candidate_before_validation',
        'rows':len(probes),'files':[{'path':s,'sha256':hashlib.sha256(Path(s).read_bytes()).hexdigest()} for s in files]}
Path('docs/function-lane/evidence/w111-broad-20260929/address/grammar-freeze.json').write_text(json.dumps(freeze,indent=2),encoding='utf-8')
print(len(probes),out)
