"""Fresh bounded switched-VDB public dispatch typing after the production freeze."""
import copy,json,random,hashlib
from pathlib import Path
import gen_broad_typed_20260929 as g
cache=g.ROOT/'smart-fuzzer/cache/w111-depreciation-20260929';g.TRANCHE='w111-vdb-switched-typed-independent-20260929'
rng=random.Random(0x20260929a74f01);g.cases.clear()
for i in range(30):
 cost=rng.uniform(0.01,1e6);salvage=cost*rng.uniform(0,1.5);life=rng.uniform(.5,80);start=rng.uniform(0,life*.8);end=rng.uniform(start,life)
 values=[cost,salvage,life,start,end];base=list(map(g.n,values));factor=rng.uniform(.01,8)
 variants=[('default',base),('factor',base+[g.n(factor)]),('false',base+[g.n(factor),g.b(False)]),('missing-factor',base+[g.missing(),g.b(False)]),('missing-flag',base+[g.n(factor),g.missing()]),('blank-flag',base+[g.n(factor),g.blank()]),('numeric-text-flag',base+[g.n(factor),g.t('0')])]
 for tag,args in variants:g.emit('VDB',f'{i}-{tag}',args,axis='switched_typed_independent')
 args=base+[g.n(factor),g.b(False)];fixtures=[g.fix(f'{chr(66+j)}2',value) for j,value in enumerate(args)]
 g.emit('VDB',f'{i}-all-references',[g.r(f'{chr(66+j)}2') for j in range(7)],fixtures,axis='switched_reference_independent')
 # Row/column and reference areas include independent observed numerical lanes.
 for pos in [0,3,5,6]:
  changed=copy.deepcopy(args);v=changed[pos]
  alt=g.b(True) if pos==6 else g.n(values[0]*.5 if pos==0 else start*.5 if pos==3 else factor*.5)
  changed[pos]=g.a([[v,alt]])
  g.emit('VDB',f'{i}-row{pos}',changed,axis='switched_array_independent')
  changed[pos]=g.r('B4:B5');g.emit('VDB',f'{i}-refcol{pos}',changed,[g.fix('B4',v),g.fix('B5',alt)],axis='switched_reference_array_independent')
# Position-sensitive errors and missing required values remain visible.
for pos in range(7):
 for tag,value in [('missing',g.missing()),('blank',g.blank()),('ref-error',g.e('Ref')),('value-error',g.e('Value')),('numeric-text',g.t('2')),('logical',g.b(False))]:
  args=list(map(g.n,[1397.25,84.5,13.5,2.25,8.75,2]))+[g.b(False)];args[pos]=value
  g.emit('VDB',f'position{pos}-{tag}',args,axis='switched_typed_position')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111vdb-independent-')
p=cache/'typed-vdb-switched-independent.json';p.write_text(json.dumps(dict(schema_version='oxfunc.smart_fuzzer.scenario_seed_case_set.v0',tranche_id=g.TRANCHE,cases=g.cases,tranches=[dict(tranche_id=g.TRANCHE,case_ids=[c['case_id'] for c in g.cases])]),indent=2)+'\n')
(cache/'typed-vdb-switched-independent-manifest.json').write_text(json.dumps(dict(cases=len(g.cases),batch_sha256=hashlib.sha256(p.read_bytes()).hexdigest(),generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),source_freeze='switched-production-v1-freeze.json',seed='0x20260929a74f01',qualification='Fresh deterministic tuples after explicit production freeze; default factor, optional/required missing, text/logical/error, direct/reference arrays; endpoints below80. Arbitrary locale grammar remains an independent open seam.'),indent=2)+'\n')
print(json.dumps(dict(path=str(p),cases=len(g.cases),tranche=g.TRANCHE)))
