"""Fresh per-function serializer evidence after the Number-only candidate freeze."""
import hashlib,json,random,struct
from collections import Counter
from pathlib import Path
import gen_broad_typed_20260929 as g

def main():
    seed=202609292347;rng=random.Random(seed);g.TRANCHE='w111-explicit-number-text-heldout-20260929'
    prior=json.loads(Path('smart-fuzzer/runs/w111-explicit-number-text-20260929/design.json').read_text(encoding='utf-8'))
    old={r['bits'] for r in prior['inputs']}
    generic=json.loads(Path('smart-fuzzer/runs/w111-numeric-to-text-heldout-20260929/design.json').read_text(encoding='utf-8'))
    pool=[r for r in generic['inputs'] if r['bits'] not in old]
    rows=rng.sample(pool,128)
    for _ in range(192):
        bits=f'{rng.randrange(0x0010000000000000,0x7ff0000000000000)|(rng.randrange(2)<<63):016x}'
        rows.append({'bits':bits,'value':struct.unpack('>d',bytes.fromhex(bits))[0],'origin':'fresh_uniform_normal_bits'})
    for i,row in enumerate(rows):
        n=g.n(row['value'])
        for mode in [0,1]:
            g.emit('VALUETOTEXT',f'number-{i}-mode-{mode}',[n,g.n(mode)],axis='fresh_explicit_number_serializer')
            g.emit('ARRAYTOTEXT',f'number-{i}-mode-{mode}',[g.a([[n,n]]),g.n(mode)],axis='fresh_explicit_number_serializer')
            if i%17==0:
                g.emit('VALUETOTEXT',f'reference-{i}-mode-{mode}',[g.r('B200'),g.n(mode)],[g.fix('B200',n)],axis='serializer_reference_origin')
                g.emit('ARRAYTOTEXT',f'reference-{i}-mode-{mode}',[g.r('B200:C200'),g.n(mode)],[g.fix('B200:C200',g.a([[n,n]]))],axis='serializer_reference_origin')
        if i%10==0:
            arr=g.a([[n,g.t('x"y')],[g.b(False),n]])
            g.emit('VALUETOTEXT',f'mixed-{i}',[arr,g.n(1)],axis='serializer_mixed_array')
            g.emit('ARRAYTOTEXT',f'mixed-{i}',[arr,g.n(1)],axis='serializer_mixed_array')
    for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111explicitnumbertextholdout-')
    out=Path('smart-fuzzer/runs/w111-explicit-number-text-heldout-20260929');out.mkdir(parents=True,exist_ok=True)
    (out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
    sources=[Path('crates/oxfunc_core/src/functions')/f for f in ['valuetotext_fn.rs','array_text_split_family.rs','adapters.rs','numeric_text.rs','round_fn.rs']]
    design={'seed':seed,'rows':len(g.cases),'functions':dict(Counter(c['canonical_surface_name'] for c in g.cases)),'inputs':rows,'candidate_sha256':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in sources},'intent':'192freshfullbitnormalvalues+128previouslyunseenserializerboundaryinputs, both modes, reference and mixed arrays.'}
    (out/'design.json').write_text(json.dumps(design,indent=2),encoding='utf-8');print(len(g.cases),design['functions'])
if __name__=='__main__':main()
