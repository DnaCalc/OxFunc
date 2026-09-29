"""Distinguish directed count fallback and ROUND signed-width interaction."""
import hashlib,json,math,struct
from pathlib import Path

def bits(n):return '0x'+struct.pack('>d',float(n)).hex()
def main():
    out=Path('smart-fuzzer/runs/w111-rounding-count-refinement-20260929');(out/'batches').mkdir(parents=True,exist_ok=True)
    rows=[]
    counts=[98,99,100,101,102,307,308,309,310,350,500,999,1000,1001,32766,32767,32768,32769,65534,65535,65536,65537,2**24-1,2**24,2**30-1,2**30,2**31-2,2**31-1,2**31,2**31+1,2**31+99,2**31+100,2**31+101,2**32-1,2**32,2**32+1,1e100]
    for exp in [-308,-200,-113,-101,-100,-99,-98,-87,-86,-2,-1,0,1,14,99,100,101,113,200,308]:
        x=1.234567890123456*10.0**exp
        if x<2**-1022:continue
        for d in counts:
            for sign in [-1,1]:rows.append((x,sign*d))
    # Decimal-width rounding overflow: move signed count around the magnitude's
    # decimal point, include both sides of zero exponent and conversion bounds.
    for exp in [-308,-100,-15,-2,-1,0,1,2,14,15,100,307]:
        x=1.125*10.0**exp
        if x<2**-1022:continue
        for center in [2**31,-2**31]:
            for shift in [-exp-17,-exp-16,-exp-15,-exp-2,-exp-1,-exp,-exp+1,-exp+2,-2,-1,0,1,2]:
                for sign in [-1,1]:rows.append((sign*x,center+shift))
    # Find the directed fallback transition independently of exact decimal ties.
    for d in [400,511,512,513,1000,1023,1024,1025,2047,2048,2049,4095,4096,4097,8191,8192,8193,16383,16384,16385,30000,32000,32760,32766,32767,32768,32769,65535,65536,100000]:
        for x in [1.125e-99,1.125e-101,1.125e100,1.125e101]:
            for sign in [-1,1]:rows.append((x,sign*d))
    rows=list(dict.fromkeys(tuple(map(bits,r)) for r in rows))
    for fn in ['TRUNC','ROUNDDOWN','ROUNDUP','ROUND']:
        packet={'function':fn,'probes':[{'probe':{'id':f'w111roundcount-{fn}-{i:05d}','args':list(row)}} for i,row in enumerate(rows)]}
        (out/'batches'/f'batch-{fn.lower()}.json').write_text(json.dumps(packet,separators=(',',':')),encoding='utf-8')
    (out/'design.json').write_text(json.dumps({'phase':'discovery_after_failed_frozen_candidate','numeric_per_function':len(rows),'purpose':'Test finite count fallback around signed/unsigned widths and decimal-scale overflow; no runtime edits before capture.','source_sha256':{f:hashlib.sha256(Path('crates/oxfunc_core/src/functions',f).read_bytes()).hexdigest() for f in ['round_fn.rs','rounddown_fn.rs','roundup_fn.rs','trunc_fn.rs']}},indent=2),encoding='utf-8');print(len(rows),'per function')
if __name__=='__main__':main()
