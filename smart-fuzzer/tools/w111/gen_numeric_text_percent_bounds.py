"""Separate exponent/significand admission from percent scaling and publication."""
import json
from pathlib import Path
import gen_broad_typed_20260929 as g

g.TRANCHE='w111-numeric-text-percent-bounds-20260929'
for i,body in enumerate([f'{m}e{e}' for m in ['0','00','0.0','0.00','1','10','.1','1.0','2.22507385850721'] for e in [-311,-310,-309,-308,-307,307,308,309,310,311]]):
    for j,text in enumerate([body,body+'%','% '+body,'( '+body+' % )']):
        g.emit('ABS',f'boundary-{i}-{j}',[g.t(text)],axis='decimal_percent_admission')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111textbounds-')
out=Path('smart-fuzzer/runs/w111-broad-20260929/numeric-text-percent-bounds.json')
out.write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
print(len(g.cases),out)
