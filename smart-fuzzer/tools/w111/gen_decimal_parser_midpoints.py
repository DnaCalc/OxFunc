"""Independent decimal-to-binary discrimination, using only exact arithmetic.

Hypotheses are numerical models, not assertions about Excel internals. Seek
15-digit decimal inputs where one/two binary-rounding or power-scaling models
disagree. Neighbor decimal significands test whether each model generalizes.
"""
import json,random,struct
from fractions import Fraction as F
from pathlib import Path
import gen_broad_typed_20260929 as g

SEED=202609291739
rng=random.Random(SEED)
g.TRANCHE='w111-decimal-parser-midpoints-20260929'

def rn(q,p):
    if not q:return q
    e=q.numerator.bit_length()-q.denominator.bit_length()
    if q < (F(2)**e):e-=1
    shift=p-1-e
    z=q*(F(2)**shift)
    n,r=divmod(z.numerator,z.denominator)
    if r*2>z.denominator or (r*2==z.denominator and n&1):n+=1
    return F(n)*(F(2)**(-shift))

def bits(q):return '0x'+struct.pack('>d',float(q)).hex()
def hypotheses(s,k):
    q=F(s)*(F(10)**k)
    return {
        'exact_rn53':bits(q),
        'exact_rn63_rn53':bits(rn(q,63)),
        'exact_rn64_rn53':bits(rn(q,64)),
        'multiply_rn64_power':bits(rn(F(s)*rn(F(10)**k,64),64)),
        'divide_rn64_power':bits(rn(F(s)/rn(F(10)**(-k),64),64)),
        'multiply_rn63_power':bits(rn(F(s)*rn(F(10)**k,63),63)),
        'divide_rn63_power':bits(rn(F(s)/rn(F(10)**(-k),63),63)),
    }

centers=[(935037643088324,-142)]
draws=0
while len(centers)<96:
    draws+=1
    s=rng.randrange(10**14,10**15);k=rng.randrange(-305,289)
    h=hypotheses(s,k)
    if len(set(h.values()))>1:centers.append((s,k))
rows=[]
for ci,(s,k) in enumerate(centers):
    for delta in [-2,-1,0,1,2]:
        text=f'{s+delta}e{k}'
        rows.append({'text':text,'center':ci,'delta':delta,'hypotheses':hypotheses(s+delta,k)})
        for fn in ['ABS','IMREAL']:
            g.emit(fn,f'center-{ci}-neighbor-{delta}',[g.t(text)],axis='decimal_parser_midpoint_discriminator')
for i in range(1000):
    s=rng.randrange(10**14,10**15);k=rng.randrange(-305,289)
    text=f'{s}e{k}'
    rows.append({'text':text,'random_control':i,'hypotheses':hypotheses(s,k)})
    g.emit('ABS',f'random-{i}',[g.t(text)],axis='decimal_parser_fresh_normal_control')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111decimalmid-')
out=Path('smart-fuzzer/runs/w111-decimal-parser-midpoints-20260929');out.mkdir(parents=True,exist_ok=True)
(out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
(out/'hypotheses.json').write_text(json.dumps({'seed':SEED,'draws':draws,'centers':centers,'rows':rows},separators=(',',':')),encoding='utf-8')
print(len(g.cases),'cases',draws,'draws',out/'typed.json')
