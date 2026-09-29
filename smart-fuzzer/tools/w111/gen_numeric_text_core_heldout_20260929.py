"""Fresh narrow numeric-text candidate validation; captures no oracle values."""
import copy
import hashlib
import json
import random
from pathlib import Path
import gen_broad_typed_20260929 as g

seed=202609291517
rng=random.Random(seed)
g.TRANCHE='w111-numeric-text-core-heldout-20260929'
forms=[]
for _ in range(90):
    body=format(rng.uniform(.001,99.999),'.17g')
    forms.append(rng.choice([body+'%', '%'+body, '('+body+')', '('+body+'%)', '('+body+')%']))
for _ in range(50):
    body=f'{rng.randrange(1,10**14)}.{rng.randrange(0,1000):03d}e{rng.randrange(-18,-10):+d}'
    forms.append(rng.choice([body+'%', '% '+body, '('+body+')', '('+body+' % )', '% ('+body+')']))
for body in ['0','0.29','2.3','9.7','1.005','.5','2.','2e-2','200e-2']:
    forms += ['%'+body,body+'%', '('+body+'%)','('+body+')%', '%('+body+')',
              '(%'+body+')', '-'+body+'%', '%-'+body, '+%'+body, '-%'+body,
              '- '+body, '+ '+body, '% + '+body, '('+body+') %', ' % ( '+body+' ) ']
forms += ['((2))','%%2','2%%','%2%','(+2)','(-2)','(-2%)','%(+2)',
          '(2)(3)','2%3','2% %','%','(%)','()',' % () ', '(  )',
          '\t2','2\n','\u00a02','2\u00a0','(\t2)','2\t%','%\n2']
forms=list(dict.fromkeys(forms))
specs={'ABS':[g.n(2)],'INT':[g.n(2)],'ROUND':[g.n(2),g.n(8)],
       'ADDRESS':[g.n(2),g.n(3),g.n(4),g.b(False)],
       'SUM':[g.n(2),g.n(3)],'NOT':[g.n(2)]}
for fn,args in specs.items():
    for i,text in enumerate(forms):
        changed=copy.deepcopy(args);changed[0]=g.t(text)
        g.emit(fn,f'direct-{i}',changed,axis='numeric_text_core_heldout')
        changed[0]=g.r('B1')
        g.emit(fn,f'reference-{i}',changed,[g.fix('B1',g.t(text))],axis='numeric_text_core_heldout')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111numtextcore-')
out=Path('smart-fuzzer/runs/w111-broad-20260929/numeric-text-core-heldout.json')
doc={'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,
     'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]}
out.write_text(json.dumps(doc,ensure_ascii=False,separators=(',',':')),encoding='utf-8')
freeze={'seed':seed,'phase':'before_independent_oracle_validation','case_count':len(g.cases),
        'distinct_forms':len(forms),'input_sha256':hashlib.sha256(out.read_bytes()).hexdigest(),
        'source_sha256':{p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in
                         ['crates/oxfunc_core/src/coercion.rs',__file__]},
        'open_lanes':['locale/grouping/currency/digits/date/time/context',
                      'percentage rounding and decoration placements being validated']}
Path('docs/function-lane/evidence/w111-broad-20260929/numeric-text/core-candidate-freeze.json').write_text(
    json.dumps(freeze,indent=2),encoding='utf-8')
print(len(forms),'forms',len(g.cases),'cases',out)
