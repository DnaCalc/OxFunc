"""Independent lexical precision, decimal scaling and decoration validation."""
import copy
import hashlib
import json
import random
from pathlib import Path
import gen_broad_typed_20260929 as g

seed=202609291603
rng=random.Random(seed)
g.TRANCHE='w111-numeric-text-precision-heldout-20260929'
def decorate(body):
    return rng.choice([body,' + '+body,' - '+body,'( '+body+' )',body+' %',
                       '% '+body,'+% '+body,'-% '+body,'% ( '+body+' )','( '+body+' % )'])
forms=[]
for _ in range(1200):
    digits=str(rng.randrange(1,10))+''.join(str(rng.randrange(10)) for _ in range(rng.randrange(0,65)))
    point=rng.randrange(len(digits)+1)
    exponent=rng.randrange(-295,296)-(point-1)
    body='0'*rng.randrange(7)+digits[:point]+'.'+digits[point:]+rng.choice(['e','E'])+f'{exponent:+d}'
    forms.append(decorate(body))
edges=['0','-0','+0','000.000','0e99999999999999999999','0e-99999999999999999999',
       '1e308','1e309','1e310%','1e-307','1e-308','1e-309','1e-323','5e-324','1e-324',
       '2.2250738585072014e-308','2.22507385850721e-308','2.22507385850720e-308',
       '1.7976931348623157e308','1.79769313486232e308','1.79769313486231e308',
       '1e99999999999999999999','1e-99999999999999999999','000000000000000000001.23',
       '12345678901234549999999999999','12345678901234550000000000000',
       '000.00000000000000012345678901234567','1.0000000000000000000000000000000000001',
       '9999999999999999999999999999999999999e-36']
for i,text in enumerate(forms+edges):
    for direct in [True,False]:
        arg=g.t(text) if direct else g.r('B1')
        g.emit('ABS',f'precision-{i}-'+('direct' if direct else 'reference'),[arg],
               [] if direct else [g.fix('B1',g.t(text))],axis='ascii_numeric_text_precision_heldout')
for i in range(96):
    text=decorate(format(rng.uniform(.001,99),'.17g'))
    for fn,args in {'INT':[g.n(2)],'ROUND':[g.n(2),g.n(8)],'SUM':[g.n(2),g.n(3)],
                    'ADDRESS':[g.n(2),g.n(3),g.n(4),g.b(False)]}.items():
        for direct in [True,False]:
            changed=copy.deepcopy(args);changed[0]=g.t(text) if direct else g.r('B1')
            g.emit(fn,f'cross-{i}-'+('direct' if direct else 'reference'),changed,
                   [] if direct else [g.fix('B1',g.t(text))],axis='ascii_numeric_text_crossfunction_heldout')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111textprecision-')
out=Path('smart-fuzzer/runs/w111-broad-20260929/numeric-text-precision-heldout.json')
doc={'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,
     'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]}
out.write_text(json.dumps(doc,ensure_ascii=False,separators=(',',':')),encoding='utf-8')
freeze={'seed':seed,'case_count':len(g.cases),'new_random_normal_decimal_strings':1200,
        'phase':'before_independent_oracle_capture','input_sha256':hashlib.sha256(out.read_bytes()).hexdigest(),
        'source_sha256':{p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in
                         ['crates/oxfunc_core/src/coercion.rs',__file__]}}
Path('docs/function-lane/evidence/w111-broad-20260929/numeric-text/precision-candidate-freeze.json').write_text(
    json.dumps(freeze,indent=2),encoding='utf-8')
print(len(g.cases),out)
