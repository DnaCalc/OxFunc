"""Admitted normal endpoint neighborhoods and count interactions, not subnormal ingress."""
import json,struct
from pathlib import Path

def encode(n):return '0x'+struct.pack('>d',float(n)).hex()
def decode(b):return struct.unpack('>d',b.to_bytes(8,'big'))[0]
def main():
    out=Path('smart-fuzzer/runs/w111-rounding-endpoint-refinement-20260929');(out/'batches').mkdir(parents=True,exist_ok=True)
    values=[decode(0x0010000000000000+k) for k in list(range(41))+[64,128,256]]
    values += [decode(0x7fefffffffffffff-k) for k in list(range(81))+[128,256]]
    counts=[-500,-309,-308,-307,-1,0,1,15,307,308,309,323,32767,32768]
    rows=[(sign*x,d) for x in values for sign in [-1,1] for d in counts]
    for fn in ['TRUNC','ROUNDDOWN','ROUNDUP','ROUND']:
        (out/'batches'/f'batch-{fn.lower()}.json').write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'w111roundendpoint-{fn}-{i:05d}','args':list(map(encode,row))}} for i,row in enumerate(rows)]},separators=(',',':')),encoding='utf-8')
    (out/'design.json').write_text(json.dumps({'phase':'discovery_endpoint_refinement','numeric_per_function':len(rows),'source_ingress':'All normal inputs, both signs; no source negative zero or subnormals. Output subnormals retained as observed.','purpose':'Distinguish initial15 decimal normalization overflow/underflow handling from ordinary digit-count reduction.'},indent=2),encoding='utf-8');print(len(rows),'per function')
if __name__=='__main__':main()
