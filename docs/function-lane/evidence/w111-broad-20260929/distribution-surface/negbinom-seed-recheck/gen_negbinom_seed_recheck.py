import hashlib,json,struct
from pathlib import Path
root=Path.cwd();out=root/'smart-fuzzer/runs/w111-negbinom-seed-recheck-20260929';out.mkdir(exist_ok=True)
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex();decode=lambda x:struct.unpack('>d',x.to_bytes(8,'big'))[0]
p0=int(bits(.4),16);ps=[decode(p0+i) for i in range(-8,9)]
for fn in ['NEGBINOM.DIST','BETA.DIST']:
    rows=[(5.,3.,p,1.) if fn=='NEGBINOM.DIST' else (p,3.,6.,1.) for p in ps]
    rows.extend([rows[8]]*3)
    p=out/f'batch-{fn.lower()}.json';p.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'w111-seed-recheck-{fn}-{i:03d}','args':list(map(bits,row))}} for i,row in enumerate(rows)]},separators=(',',':')))
(out/'manifest.json').write_text(json.dumps({'purpose':'verify the existing strict NEGBINOM(5,3,0.4,true) unit-test pin and adjacent p values using paired public beta CDF','rows_each':len(rows),'seed_probability_bits':hex(p0)},indent=2));print('20 each')
