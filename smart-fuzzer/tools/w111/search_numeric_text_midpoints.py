"""Search rational approximants for finite f64 values close to 15-digit ties.

This only tests the candidate's decimal precision against exact arithmetic;
no result is an Excel semantic claim until captured independently.
"""
from fractions import Fraction
from decimal import Decimal,localcontext,ROUND_HALF_DOWN
import json,math,struct
from pathlib import Path

def convergents(value):
    p,q=value.numerator,value.denominator
    h0,h1,k0,k1=0,1,1,0
    while q:
        a,r=divmod(p,q)
        # Near-terminal intermediate convergents also expose close odd ties.
        choices={a}
        for bound in [2*10**14,2*10**15-1]:
            if k1:
                t=(bound-k0)//k1
                choices.update([t-1,t,t+1])
        for t in choices:
            if 0<t<=a:yield t*h1+h0,t*k1+k0
        h0,h1=h1,a*h1+h0;k0,k1=k1,a*k1+k0;p,q=q,r

def main():
    candidates={};misses=[]
    with localcontext() as ctx:
        ctx.prec=1100
        for exponent in range(-308,309):
            scale=exponent-14
            power=Fraction(10**scale,2) if scale>=0 else Fraction(1,2*10**(-scale))
            approx_binary=math.floor(exponent*math.log2(10))-52
            for binary in range(approx_binary-1,approx_binary+5):
                binary_unit=Fraction(2**binary) if binary>=0 else Fraction(1,2**(-binary))
                ratio=power/binary_unit
                for mantissa,odd in convergents(ratio):
                    if not odd:continue
                    low=max((2*10**14+odd-1)//odd,((1<<52)+mantissa-1)//mantissa)
                    high=min((2*10**15-1)//odd,((1<<53)-1)//mantissa)
                    for factor in set([low,low+1,high-1,high]):
                        if factor<low or factor>high or not (factor*odd)%2:continue
                        try:v=math.ldexp(float(factor*mantissa),binary)
                        except OverflowError:continue
                        if not math.isfinite(v) or v<2.2250738585072014e-308:continue
                        bits=struct.pack('>d',v).hex();candidates[bits]=v
        for bits,v in candidates.items():
            exact=Decimal.from_float(v)
            wanted=exact.quantize(Decimal(1).scaleb(exact.adjusted()-14),rounding=ROUND_HALF_DOWN)
            limited=Decimal(format(v,'.30e'))
            got=limited.quantize(Decimal(1).scaleb(limited.adjusted()-14),rounding=ROUND_HALF_DOWN)
            if wanted!=got:misses.append({'bits':bits,'value':v,'exact':str(exact),'candidate_decimal':str(got),'exact_decimal':str(wanted)})
    out=Path('docs/function-lane/evidence/w111-broad-20260929/numeric-to-text/precision-search.json')
    out.write_text(json.dumps({'research_only':True,'candidate_count':len(candidates),'differences':misses},indent=2),encoding='utf-8')
    print('candidates',len(candidates),'differences',len(misses))
if __name__=='__main__':main()
