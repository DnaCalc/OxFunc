"""Compare public EXP/LN and cumulative surfaces without interpreting internals."""
import json,struct
from pathlib import Path
root=Path(__file__).resolve().parents[3]
base=root/'smart-fuzzer/runs/w111-sinh-expm1-dependencies-20260929'
decode=lambda s:struct.unpack('>d',bytes.fromhex(s[2:]))[0]
bits=lambda x:'0x'+struct.pack('>d',float(x)).hex()
def bank(name):return json.loads((base/f'answers-{name}.json').read_text(encoding='utf-8-sig'))['witnesses']
exp={r['args'][0]:decode(r['expected_bits']) for r in bank('exp')}
ln={r['args'][0]:decode(r['expected_bits']) for r in bank('ln')}
gamma={r['args'][0]:r['expected_bits'] for r in bank('gamma.dist')}
counts={k:0 for k in ['current_kahan','ratio_first','inner_ratio','identity','gamma_equals_expon']}
rows=[]
for row in bank('expon.dist'):
    x=decode(row['args'][0]);t=-x;u=exp[bits(t)]
    l=ln[bits(u)]
    values={'current_kahan':t if u==1. else (u-1)*t/l,
            'ratio_first':t if u==1. else ((u-1)/l)*t,
            'inner_ratio':t if u==1. else (t/l)*(u-1),
            'identity':t}
    actual={k:bits(-v) for k,v in values.items()}
    for k,v in actual.items():counts[k]+=v==row['expected_bits']
    counts['gamma_equals_expon']+=gamma[row['args'][0]]==row['expected_bits']
    if actual['current_kahan']!=row['expected_bits']:
        rows.append({'input':row['args'][0],'expected':row['expected_bits'],'gamma':gamma[row['args'][0]],'hypotheses':actual,'exp_negative_bits':bits(u),'ln_bits':bits(l)})
report={'rows':len(bank('expon.dist')),'matches':counts,'current_kahan_misses':rows,
        'qualification':'Observed worksheet surfaces constrain mathematical hypotheses; no binary internals or universal identity claim.'}
(base/'negative-half-analysis.json').write_text(json.dumps(report,indent=2))
print(json.dumps({k:v for k,v in report.items() if k!='current_kahan_misses'}))
for r in rows[:8]:print(r)
names=['current_kahan','ratio_first','inner_ratio','identity']
def hypotheses(t):
    u=exp[bits(t)];l=ln[bits(u)]
    return [t,t,t,t] if u==1 else [(u-1)*t/l,((u-1)/l)*t,(t/l)*(u-1),t]
paired=json.loads((root/'smart-fuzzer/runs/w111-tanh-dependencies-20260929/answers-sinh.json').read_text(encoding='utf-8-sig'))['witnesses']
pair_counts={f'{a}/{b}':0 for a in names for b in names}
pair_misses={k:[] for k in pair_counts}
for row in paired:
    x=decode(row['args'][0]);positive=hypotheses(x);negative=hypotheses(-x)
    for a,va in zip(names,positive):
        for b,vb in zip(names,negative):
            key=f'{a}/{b}';actual=bits((va-vb)/2)
            pair_counts[key]+=actual==row['expected_bits']
            if actual!=row['expected_bits']:pair_misses[key].append({'row':row,'actual':actual})
pair_report={'rows':len(paired),'matches':pair_counts,'misses':pair_misses}
(base/'sinh-pair-analysis.json').write_text(json.dumps(pair_report,indent=2))
print('SINH paired:',sorted(pair_counts.items(),key=lambda p:-p[1]))
