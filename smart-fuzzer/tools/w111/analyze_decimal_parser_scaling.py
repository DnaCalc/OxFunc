"""Black-box arithmetic hypotheses for retained decimal parser observations."""
import collections,json,struct
from fractions import Fraction as F
from functools import lru_cache
from pathlib import Path

def rn(q,p=64):
    if not q:return q
    e=q.numerator.bit_length()-q.denominator.bit_length()
    if q<F(2)**e:e-=1
    sh=p-1-e;z=q*F(2)**sh;n,r=divmod(z.numerator,z.denominator)
    if r*2>z.denominator or (r*2==z.denominator and n&1):n+=1
    return F(n)*F(2)**(-sh)
def bits(q):return '0x'+struct.pack('>d',float(q)).hex()
@lru_cache(None)
def power(k,p):return rn(F(10)**k,p)

def nibble64(s,k):
    """Candidate frozen after the first midpoint capture; unbounded exponent carrier."""
    q=F(s);ex=abs(k)
    for exp in [ex%16,(ex//16%16)*16,(ex//256)*256]:
        if exp:q=rn(q*power(-exp if k<0 else exp,64),64)
    return q

def decimal_parts(text):
    """Independent lexical extraction for already-admitted ASCII decorated forms."""
    import re
    body=text.strip(' ');percent=False;signed=False;negative=False
    while True:
        if body.startswith('%') or body.endswith('%'):
            if percent:return None
            percent=True;body=(body[1:] if body.startswith('%') else body[:-1]).strip(' ')
        elif body.startswith(('+','-')):
            if signed:return None
            signed=True;negative=body[0]=='-';body=body[1:].strip(' ')
        elif body.startswith('(') and body.endswith(')'):
            if signed:return None
            signed=True;negative=True;body=body[1:-1].strip(' ')
        else:break
    match=re.fullmatch(r'([0-9]*\.?[0-9]*)(?:[eE]([+-]?[0-9]+))?',body)
    if not match:return None
    mantissa,exponent=match.groups();digits=mantissa.replace('.','')
    if not digits:return None
    significant=digits.lstrip('0');fraction=len(mantissa.split('.')[1]) if '.' in mantissa else 0
    exponent=int(exponent or 0)-fraction-(2 if percent else 0)
    position=exponent+len(significant)
    if not -308<=position<=308:return None
    s=int(significant[:15] or 0);k=exponent+max(0,len(significant)-15)
    return s,k,negative

def models(s,k):
    out={}
    for p in [63,64,65]:
        for pattern in ['binary','chunk8','chunk16','chunk32','chunk64']:
            if pattern=='binary':chunks=[1<<i for i in range(10) if abs(k)&(1<<i)]
            else:
                width=int(pattern[5:]);hi,lo=divmod(abs(k),width);chunks=([lo] if lo else [])+([hi*width] if hi else [])
            for order in ['ascending','descending']:
                chunks=sorted(chunks,reverse=order=='descending')
                for negative_mode in ['reciprocal','divide']:
                    for accumulation in ['from_significand','power_first']:
                        q=F(s) if accumulation=='from_significand' else F(1)
                        for exp in chunks:
                            if k<0 and negative_mode=='divide':q=rn(q/power(exp,p),p)
                            else:q=rn(q*power(-exp if k<0 else exp,p),p)
                        if accumulation=='power_first':q=rn(q*s,p)
                        out[f'p{p}-{pattern}-{order}-{negative_mode}-{accumulation}']=bits(q)
    return out

def main():
    run=Path('smart-fuzzer/runs/w111-decimal-parser-midpoints-typed-20260929')
    cases={r['case_id']:r for r in map(json.loads,(run/'cases/cases.jsonl').read_text(encoding='utf-8-sig').splitlines())}
    answers={r['case_id']:r['outcome']['digest_payload'].split(':')[1] for r in map(json.loads,(run/'outcomes/excel.jsonl').read_text(encoding='utf-8-sig').splitlines())}
    counts=collections.Counter();n=0
    for key,case in cases.items():
        if case['canonical_surface_name']!='ABS':continue
        s,k=map(int,case['args'][0]['value'].split('e'))
        for name,guess in models(s,k).items():counts[name]+=guess!=answers[key]
        n+=1
    report={'rows':n,'counts':dict(sorted(counts.items(),key=lambda item:item[1]))}
    Path('docs/function-lane/evidence/w111-broad-20260929/numeric-text/midpoint-scaling-models.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(n,'rows best',list(report['counts'].items())[:20])

if __name__=='__main__':main()
