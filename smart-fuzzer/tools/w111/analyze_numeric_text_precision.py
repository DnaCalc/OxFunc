"""Compare general rounding graphs; do not select a policy per captured input."""
import decimal,json,struct
from collections import Counter
from fractions import Fraction as F
from pathlib import Path
from analyze_decimal_parser_scaling import rn,power,nibble64

def bits(n):return '0x'+struct.pack('>d',n).hex()
def half_down(q):
    n,r=divmod(q.numerator,q.denominator);return n+(2*r>q.denominator)
def main():
    run=Path('smart-fuzzer/runs/w111-numeric-text-rendering-precision-typed-20260929')
    cases=list(map(json.loads,(run/'cases/cases.jsonl').read_text(encoding='utf-8-sig').splitlines()))
    excel={r['case_id']:r for r in map(json.loads,(run/'outcomes/excel.jsonl').read_text(encoding='utf-8-sig').splitlines())}
    counts=Counter();n=0;residual=[];decimal.getcontext().prec=1100
    for c in cases:
        if c['canonical_surface_name']!='ROUND' or c['args'][0]['value']<0:continue
        n+=1;x=c['args'][0]['value'];k=int(c['args'][1]['value']);expected=excel[c['case_id']]['outcome']['bits_hex'];pairs={}
        exact=decimal.Decimal.from_float(x);quantum=decimal.Decimal(1).scaleb(-k)
        pairs['exact15']=int(exact.quantize(quantum,rounding=decimal.ROUND_HALF_DOWN).scaleb(k))
        for prec in [16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36]:
            limited=exact.quantize(decimal.Decimal(1).scaleb(exact.adjusted()-prec+1),rounding=decimal.ROUND_HALF_EVEN)
            pairs[f'decimal{prec}then15']=int(limited.quantize(quantum,rounding=decimal.ROUND_HALF_DOWN).scaleb(k))
        for p in [53,64,80,96,100,101,102,103,104,105,106,107,108,112,113,128]:
            for policy in ['signed','split']:
                v=rn(F(x)*power(k,p),p) if policy=='signed' or k>=0 else rn(F(x)/power(-k,p),p)
                pairs[f'rn{p}-{policy}']=half_down(v)
        for name,s in pairs.items():
            for final in ['nearest','nibble64']:
                value=float(F(s)*F(10)**(-k)) if final=='nearest' else float(nibble64(s,-k))
                guess=bits(value);counts[name+'-'+final]+=guess!=expected
                if name=='decimal31then15' and guess!=expected:residual.append({'case_id':c['case_id'],'bits':bits(x),'final':final,'expected':expected,'actual':guess})
    out={'rows':n,'model_differences':dict(sorted(counts.items(),key=lambda item:item[1])),'decimal31_residuals':residual,'research_only':True}
    Path('docs/function-lane/evidence/w111-broad-20260929/numeric-to-text/precision-models.json').write_text(json.dumps(out,indent=2),encoding='utf-8');print(n,list(out['model_differences'].items())[:25])
if __name__=='__main__':main()
