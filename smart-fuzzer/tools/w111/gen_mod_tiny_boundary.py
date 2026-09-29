"""Discriminate tiny MOD output using only normal binary64 input values."""
import hashlib,json,struct
from pathlib import Path
root=Path(__file__).resolve().parents[3];out=root/'smart-fuzzer/runs/w111-mod-tiny-boundary-20260929';out.mkdir(parents=True,exist_ok=True)
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex();decode=lambda b:struct.unpack('>d',b.to_bytes(8,'big'))[0]
rows=[]
for exponent in [1,2,3,20,52,53]:
    base=exponent<<52
    for i in range(1,129):
        for sign in [-1,1]:
            rows.extend([(sign*decode(base+i),decode(base)),(sign*decode(base+i),-decode(base)),(sign*decode(base),decode(base+i)),(sign*decode(base),-decode(base+i))])
encoded=list(dict.fromkeys(tuple(map(bits,row)) for row in rows));p=out/'batch-mod.json';p.write_text(json.dumps({'function':'MOD','probes':[{'probe':{'id':f'w111mod-tiny-{i:05d}','args':r}} for i,r in enumerate(encoded)]},separators=(',',':')))
(out/'manifest.json').write_text(json.dumps({'rows':len(encoded),'purpose':'normal-input tiny remainders and endpoint normalization','sha256':hashlib.sha256(p.read_bytes()).hexdigest()},indent=2));print(len(encoded))
