"""Independent validation of the frozen staged decimal scale arithmetic model."""
import hashlib,json,random
from pathlib import Path
import gen_broad_typed_20260929 as g
from analyze_decimal_parser_scaling import F,bits,nibble64,rn

SEED=202609291857
rng=random.Random(SEED)
g.TRANCHE='w111-decimal-parser-nibble-heldout-20260929'
centers=[];draws=0;hypotheses=[]
while len(centers)<160:
    draws+=1;s=rng.randrange(10**14,10**15);k=rng.randrange(-305,289)
    candidate=bits(nibble64(s,k));exact=bits(F(s)*F(10)**k)
    if candidate!=exact or candidate!=bits(rn(F(s)*F(10)**k)):
        centers.append((s,k))

def emit(text,tag,functions=('ABS','IMREAL')):
    for fn in functions:g.emit(fn,tag,[g.t(text)],axis='decimal_parser_nibble_independent_validation')

for i,(s,k) in enumerate(centers):
    for delta in [-2,-1,0,1,2]:
        text=f'{s+delta}e{k}';emit(text,f'center-{i}-neighbor-{delta}')
        hypotheses.append({'text':text,'nibble_rn64':bits(nibble64(s+delta,k)),'exact_rn53':bits(F(s+delta)*F(10)**k)})
for i in range(1200):
    width=rng.randrange(1,16);s=rng.randrange(10**(width-1),10**width)
    k=rng.randrange(-305,294)-width
    emit(f'{s}e{k}',f'random-{i}',('ABS',))
for i in range(160):
    width=rng.randrange(1,15);s=rng.randrange(10**(width-1),10**width);k=rng.randrange(-290,280)-width
    for j,text in enumerate([f'{s}e{k}',f'{s}'+('0'*(15-width))+f'e{k-(15-width)}',f'0.{s}e{k+width}']):
        emit(text,f'equivalent-{i}-{j}')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111decimalnibble-')
out=Path('smart-fuzzer/runs/w111-decimal-parser-nibble-heldout-20260929');out.mkdir(parents=True,exist_ok=True)
packet=out/'typed.json';packet.write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
(out/'hypotheses.json').write_text(json.dumps({'seed':SEED,'draws':draws,'centers':centers,'hypotheses':hypotheses},separators=(',',':')),encoding='utf-8')
freeze={'seed':SEED,'rows':len(g.cases),'phase':'before_independent_oracle_capture','model':'RN64 integer significand scaled by correctly-rounded RN64 powers10 in ascending base16 exponent digits, then RN53','source_sha256':{p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in ['smart-fuzzer/tools/w111/analyze_decimal_parser_scaling.py',__file__]},'input_sha256':hashlib.sha256(packet.read_bytes()).hexdigest()}
Path('docs/function-lane/evidence/w111-broad-20260929/numeric-text/nibble-candidate-freeze.json').write_text(json.dumps(freeze,indent=2),encoding='utf-8')
print(len(g.cases),draws,'draws',packet)
