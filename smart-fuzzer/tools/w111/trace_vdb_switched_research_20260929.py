"""Offline trace of research selector30064773330, not a production kernel.

This intentionally preserves the pre-absolute-end graph for operation
discrimination rather than silently following the latest research selector.
"""
from fractions import Fraction
import json
from pathlib import Path

root=Path(__file__).resolve().parents[3]
cache=root/'smart-fuzzer/cache/w111-depreciation-20260929'
def store64(x):
    if not x:return 0.0
    negative=x<0;x=abs(x);n,d=x.numerator,x.denominator
    exponent=n.bit_length()-d.bit_length()
    if (n < d<<exponent) if exponent>=0 else (n<<-exponent < d):exponent-=1
    shift=63-exponent
    q,rem=divmod(n<<shift,d) if shift>=0 else divmod(n,d<<-shift)
    denominator=d if shift>=0 else d<<-shift
    if 2*rem>denominator or (2*rem==denominator and q&1):q+=1
    result=Fraction(q,1<<shift) if shift>=0 else Fraction(q<<-shift)
    return float(-result if negative else result)
def xdiv(a,b):return store64(Fraction(a)/Fraction(b))
def xmul(a,b):return store64(Fraction(a)*Fraction(b))
def xadd(a,b):return store64(Fraction(a)+Fraction(b))

def trace(args):
    c,s,l,start,end,factor,_=args
    assert 0<=start<end<=l<1000 and c>=s
    rate=xdiv(factor,l);dd=xmul(c,rate);sl=xdiv(c-s,l)
    if sl>min(dd,c-s):return {'result':sl*(end-start),'initial_straight':True,'steps':[]}
    fraction=start-int(start);book=c-xmul(min(dd,c-s),fraction)
    remaining_life=l-fraction;sl_fixed=None;steps=[]
    for phase in [0,1]:
        inherited=sl_fixed is not None
        duration=float(int(start)) if phase==0 else end-start if inherited else (end-fraction)-int(start)
        total=0.;year=0
        while year<duration:
            dd=xmul(book,rate);available=book-s
            denominator=max(remaining_life,1.) if dd>available else remaining_life
            sl=sl_fixed if sl_fixed is not None else xdiv(available,denominator)
            switched=sl_fixed is None and dd<=available and sl>min(dd,available)
            if sl_fixed is None and dd>available:amount=available
            elif sl>min(dd,available):sl_fixed=sl;amount=sl
            else:amount=min(dd,available)
            take=end-start if phase==1 and year==0 and sl_fixed is not None else duration-year if sl_fixed is not None else min(duration-year,1.)
            part=amount*take;total=xadd(total,part)
            steps.append(dict(phase=phase,year=year,inherited=inherited,switched=switched,
                              book=book,available=available,remaining_life=remaining_life,
                              dd=dd,sl=sl,amount=amount,take=take,total=total))
            book-=part;remaining_life-=take
            if sl_fixed is not None:break
            year+=1
    return {'result':total,'initial_straight':False,'steps':steps}
if __name__=='__main__':
    report=json.loads((cache/'vdb-first-step-sl-duration-heldout-vdb-switched-research-answers.json').read_text())
    rows=[]
    for seed in report['misses']:
        row=trace(seed['args']);row['id']=seed['id'];row['args']=seed['args']
        import struct
        actual='0x'+struct.pack('>d',row['result']).hex()
        assert actual==seed['actual'],(seed['id'],actual,seed['actual'])
        rows.append(row)
    (cache/'vdb-switched-eight-local-stage-traces.json').write_text(json.dumps(rows,indent=2)+'\n')
    print(json.dumps([{'id':r['id'],'switches':[(s['phase'],s['year']) for s in r['steps'] if s['switched']],
                      'steps':len(r['steps'])} for r in rows]))
