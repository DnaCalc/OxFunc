"""Pure black-box hypothesis: count conversion and decimal pair reduction."""
import json,math,struct,sys
from collections import Counter
from fractions import Fraction as F
from pathlib import Path
from analyze_decimal_parser_scaling import nibble64

def decode(s):return struct.unpack('>d',bytes.fromhex(s[2:]))[0]
def encode(x):return '0x'+struct.pack('>d',x).hex()
def i32(n):return (n+2**31)%2**32-2**31
def pair(x,away):
    mantissa,exp=format(abs(x),'.30e').split('e');digits=mantissa.replace('.','');head=int(digits[:15]);tail=digits[15:];half='5'+'0'*(len(tail)-1)
    return head+(tail>half or tail==half and away),int(exp)-14
def decimal(s,k,assembly):
    if not s:return 0.0
    if k>400:return math.inf
    if k<-400:return 0.0
    try:
        value=float(f'{s}e{k}') if assembly=='nearest' else float(nibble64(s,k))
        return 0.0 if abs(value)<2**-1022 else value
    except OverflowError:return math.inf
def predict(fn,args,assembly='nearest',round_graph='none',fallback=32767):
    x,raw=map(decode,args)
    if x==0 or abs(x)<2**-1022:return encode(0.)
    away=fn!='ROUND';s,k=pair(x,away)
    if fn=='ROUND':
        d=min(math.trunc(abs(raw)),2**31-1)
        if raw<0:d=-d
    else:
        d=min(math.trunc(abs(raw)),2**32-1)&0xffff
        original_d=-d if raw<0 else d
        if d>fallback:d=100
        if raw<0:d=-d
    dropped=-(k+d)
    if fn=='ROUND' and round_graph!='none':
        exponent=k+14
        if round_graph=='count':
            precision=i32(d+exponent+1);dropped=15-precision
        elif round_graph=='exponent':
            precision=i32(d+exponent);dropped=14-precision
        elif round_graph=='edge':
            precision=i32(d+exponent)
            if precision==2**31-1:return encode(0.)
            dropped=14-precision
        elif round_graph=='scale':dropped=i32(-(i32(k+d)))
    if dropped<=0:result=decimal(s,k,assembly)
    else:
        divisor=10**dropped if dropped<32 else 10**32;q,r=divmod(s,divisor);kept=k+dropped
        if fn=='ROUND':result=decimal(q+(2*r>=divisor),kept,assembly)
        elif fn in ['ROUNDDOWN','TRUNC']:result=decimal(q,kept,assembly)
        else:result=decimal(q,kept,assembly)+(decimal(1,-original_d,assembly) if r else 0)
    if not math.isfinite(result):return 'error:Num'
    return encode(math.copysign(result,x) if result else 0.)
def main():
    run=Path(sys.argv[1]) if len(sys.argv)>1 else Path('smart-fuzzer/runs/w111-integer-hyperbolic-heldout-20260929/answers');reports={}
    for fn in ['TRUNC','ROUNDDOWN','ROUNDUP','ROUND']:
        data=json.loads((run/f'answers-{fn.lower()}.json').read_text());counts=Counter();misses={}
        for assembly in ['nearest','nibble64']:
            for graph in (['none','count','exponent','edge','scale'] if fn=='ROUND' else ['none']):
                name=f'{assembly}-{graph}';bad=[]
                for row in data['witnesses']:
                    actual=predict(fn,row['args'],assembly,graph)
                    if actual!=row['expected_bits']:bad.append(dict(row,actual=actual))
                counts[name]=len(bad);misses[name]=bad
        print(fn,dict(counts));reports[fn]={'rows':len(data['witnesses']),'counts':dict(counts),'misses':misses}
    output=Path(sys.argv[2]) if len(sys.argv)>2 else Path('docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries/offline-models.json')
    output.write_text(json.dumps(reports,separators=(',',':')),encoding='utf-8')
if __name__=='__main__':main()
