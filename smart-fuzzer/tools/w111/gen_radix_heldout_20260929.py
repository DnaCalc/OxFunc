"""Independent post-repair radix corpus. Preserves candidate hashes before oracle."""
import hashlib,json,math,random,sys
from pathlib import Path
import gen_broad_typed_20260929 as g
from gen_w111_broad_20260929 import bits

out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
seed=2026092906;r=random.Random(seed)
root=Path(__file__).resolve().parents[3]
m={'generator':Path(__file__).name,'seed':seed,'phase':'heldout','batches':[],
   'candidate_sha256':{str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [root/'crates/oxfunc_core/src/functions/base_fn.rs',root/'crates/oxfunc_core/src/functions/engineering_radix_family.rs']}}
for fn,limit in [('DEC2BIN',512),('DEC2OCT',2**29),('DEC2HEX',2**39),('BASE',2**53)]:
    rows=[]
    for i in range(2000):
        x=r.uniform(-limit,limit) if fn!='BASE' else r.uniform(0,limit)
        if i%4==0:x=r.choice([-1,0,1,-limit,limit-1,limit])+r.choice([.01,.125,.75,math.ulp(float(limit))])
        args=[x,r.uniform(-2,12)] if fn!='BASE' else [x,r.uniform(1,38),r.uniform(-1,258)]
        if i%5==0:args=args[:1] if fn!='BASE' else args[:2]
        rows.append(args)
    path=out/('batch-'+fn.lower()+'.json')
    path.write_text(json.dumps({'function':fn,'probes':[{'probe':{'id':f'radix-holdout-{seed}-{fn}-{i:05d}','args':[bits(x) for x in row]}} for i,row in enumerate(rows)]}),encoding='utf-8')
    m['batches'].append({'function':fn,'path':path.name,'rows':len(rows),'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})

previous=json.loads((root/'smart-fuzzer/runs/w111-radix-20260929/typed-discovery.json').read_text(encoding='utf-8'))
seen={(x['function_id'],json.dumps(x['args'],sort_keys=True)) for x in previous['cases']}
functions=['BIN2DEC','BIN2HEX','BIN2OCT','OCT2DEC','OCT2BIN','OCT2HEX','HEX2DEC','HEX2BIN','HEX2OCT','DEC2BIN','DEC2HEX','DEC2OCT','BASE']
for fn in functions:
    count=0;attempt=0
    while count<150:
        attempt+=1
        sourcebase=2 if fn.startswith('BIN') else 8 if fn.startswith('OCT') else 16
        chars='0123456789ABCDEF'[:sourcebase]
        digits=''.join(r.choice(chars) for _ in range(r.randint(1,12)))
        value=g.t(r.choice([digits,digits.lower(),digits+' ', ' '+digits,digits+'G',digits[:0]]))
        if fn.startswith('DEC') or fn=='BASE':value=g.n(r.uniform(-1000,1000))
        if attempt%5==0:value=r.choice([g.b(True),g.b(False),g.blank(),g.missing(),g.e('Ref'),g.t('3.75'),g.t('NaN')])
        places=r.choice([g.n(r.uniform(-1,12)),g.t(str(r.uniform(-1,12))),g.b(bool(r.randrange(2))),g.missing(),g.blank(),g.e('Div0')])
        args=[value] if fn.endswith('2DEC') else [value,places]
        if fn=='BASE':args=[value,r.choice([g.n(r.uniform(1,37)),g.t('16'),g.b(True),g.missing(),g.e('NA')]),places]
        if len(args)==1 and value['kind']=='missing_arg':continue
        key=('FUNC.'+fn,json.dumps(args,sort_keys=True))
        if key in seen:continue
        seen.add(key)
        if count%11==0 and value['kind'] not in ['empty_cell','missing_arg']:
            args[0]=g.a([[value,g.n(1),g.t('')],[g.n(0),g.e('Value'),g.b(True)]])
        if count%7==0 and value['kind'] not in ['missing_arg','array']:
            g.emit(fn,'heldout-reference',[g.r('C1')]+args[1:],[g.fix('C1',value)])
        else:g.emit(fn,'heldout-typed',args)
        count+=1
typed=out.parent/'typed-heldout.json'
typed.write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':'w111-radix-heldout-20260929','cases':g.cases,'seed':seed,'candidate_sha256':m['candidate_sha256']},ensure_ascii=False),encoding='utf-8')
m['typed_rows']=len(g.cases);m['typed_sha256']=hashlib.sha256(typed.read_bytes()).hexdigest()
(out/'manifest.json').write_text(json.dumps(m,indent=2),encoding='utf-8')
print('numeric',sum(x['rows'] for x in m['batches']),'typed',len(g.cases))
