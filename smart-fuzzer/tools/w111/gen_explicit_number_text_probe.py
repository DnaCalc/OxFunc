"""Distinguish explicit text serializers from generic Number-to-Text coercion."""
import json,random
from collections import Counter
from pathlib import Path
import gen_broad_typed_20260929 as g

def main():
    seed=202609292319;rng=random.Random(seed);g.TRANCHE='w111-explicit-number-text-20260929'
    source=json.loads(Path('smart-fuzzer/runs/w111-numeric-to-text-discovery-20260929/design.json').read_text(encoding='utf-8'))
    rows=source['inputs'][:24]+rng.sample(source['inputs'][24:],104)
    for i,row in enumerate(rows):
        n=g.n(row['value'])
        for mode in [0,1]:
            g.emit('VALUETOTEXT',f'number-{i}-mode-{mode}',[n,g.n(mode)],axis='explicit_number_serializer')
            g.emit('ARRAYTOTEXT',f'number-{i}-mode-{mode}',[g.a([[n,n]]),g.n(mode)],axis='explicit_number_serializer')
        if i<24:
            g.emit('VALUETOTEXT',f'array-{i}',[g.a([[n,g.t('x')],[g.b(True),n]]),g.n(1)],axis='explicit_array_serializer')
            g.emit('ARRAYTOTEXT',f'array-{i}',[g.a([[n,g.t('x')],[g.b(True),n]]),g.n(1)],axis='explicit_array_serializer')
            g.emit('TEXT',f'general-{i}',[n,g.t('General')],axis='explicit_general_format')
    for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111explicitnumbertext-')
    out=Path('smart-fuzzer/runs/w111-explicit-number-text-20260929');out.mkdir(parents=True,exist_ok=True)
    (out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
    design={'seed':seed,'rows':len(g.cases),'functions':dict(Counter(c['canonical_surface_name'] for c in g.cases)),'inputs':rows,'intent':'No policy shared with generic coercion until observed; discovery includes both serializer modes and mixed arrays.'}
    (out/'design.json').write_text(json.dumps(design,indent=2),encoding='utf-8');print(len(g.cases),design['functions'])
if __name__=='__main__':main()
