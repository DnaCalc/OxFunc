"""General staged-power graphs for retained generic decimal precision controls.

All models are selected before comparison, with no per-input fallback. This
extends previously rejected full-power models to the observed parser's staged
exponent decomposition; shared arithmetic is a hypothesis, not an assumption.
"""
import json,struct
from collections import Counter
from fractions import Fraction as F
from pathlib import Path
from analyze_decimal_parser_scaling import rn,power,nibble64
from analyze_numeric_text_precision import half_down

def main():
    run=Path('smart-fuzzer/runs/w111-numeric-text-rendering-precision-typed-20260929')
    cases=list(map(json.loads,(run/'cases/cases.jsonl').read_text(encoding='utf-8-sig').splitlines()))
    answers={r['case_id']:r['outcome']['bits_hex'] for r in map(json.loads,(run/'outcomes/excel.jsonl').read_text(encoding='utf-8-sig').splitlines()) if r['outcome'].get('bits_hex')}
    counts=Counter();rows=0;residuals={}
    for case in cases:
        if case['canonical_surface_name']!='ROUND' or case['args'][0]['value']<0:continue
        rows+=1;x=case['args'][0]['value'];k=int(case['args'][1]['value']);ex=abs(k)
        for p in [53,64,80,96]:
            for decomposition in ['nibble','binary','decimal','normalize']:
                chunks={'nibble':[ex%16,ex//16%16*16,ex//256*256],'binary':[ex&(1<<i) for i in range(10)],'decimal':[ex%10,ex//10%10*10,ex//100*100],'normalize':[abs(k-14),14]}[decomposition]
                for reverse in [False,True]:
                    for signmode in ['multiply','divide','split']:
                        divide=signmode=='divide' or signmode=='split' and k<0
                        sign=(-1 if k<0 else 1)*(-1 if divide else 1)
                        powers=[power(sign*chunk,p) for chunk in chunks if chunk]
                        if decomposition=='normalize':
                            powers=[power((k-14)*(-1 if divide else 1),p),power(14*(-1 if divide else 1),p)]
                        if reverse:powers.reverse()
                        for combine in [False,True]:
                            if combine:
                                combined=F(1)
                                for factor in powers:combined=rn(combined*factor,p)
                                factors=[combined]
                            else:factors=powers
                            scaled=F(x)
                            for factor in factors:scaled=rn(scaled/factor if divide else scaled*factor,p)
                            significand=half_down(scaled)
                            for assembly in ['nearest','nibble64']:
                                result=float(F(significand)*F(10)**(-k)) if assembly=='nearest' else float(nibble64(significand,-k))
                                bits='0x'+struct.pack('>d',result).hex();label=f'RN{p}-{decomposition}-{reverse}-{signmode}-{combine}-{assembly}'
                                wrong=bits!=answers[case['case_id']];counts[label]+=wrong
                                if wrong:residuals.setdefault(label,[]).append(case['case_id'])
    ranked=sorted(counts.items(),key=lambda p:p[1]);out={'research_only':True,'rows':rows,'model_differences':dict(ranked),'best_residual_ids':{k:residuals.get(k,[]) for k,_ in ranked[:20]},'policy':'Uniform whole-corpus hypotheses only; no runtime edits.'}
    Path('docs/function-lane/evidence/w111-broad-20260929/numeric-to-text/staged-precision-models.json').write_text(json.dumps(out,indent=2),encoding='utf-8');print(rows,ranked[:15])
if __name__=='__main__':main()
