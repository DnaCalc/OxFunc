"""Shared numeric-text grammar discovery with verified direct/reference origins."""
import copy,hashlib,json,sys
from pathlib import Path
import gen_broad_typed_20260929 as g
g.TRANCHE='w111-numeric-text-20260929'
specs={fn:[g.n(2)] for fn in ['ABS','INT','TRUNC','SIGN','SQRT','FACT','FACTDOUBLE','ISEVEN','ISODD','SIN','COS','ATAN','ACOT','LN','EXP','DAY','MONTH','YEAR','HOUR','MINUTE','SECOND','ISOWEEKNUM','NOT']}
specs.update({fn:list(map(g.n,args)) for fn,args in {
 'BASE':[2,10],'DEC2BIN':[2],'DEC2HEX':[2],'DEC2OCT':[2],
 'BITAND':[2,3],'BITOR':[2,3],'BITXOR':[2,3],'BITLSHIFT':[2,1],'BITRSHIFT':[2,1],
 'ADDRESS':[2,3],'COMPLEX':[2,3],'SLN':[2,1,2],'DDB':[100,2,10,1],
 'DATE':[2000,2,2],'DAYS':[2,1],'EDATE':[2,1],'EOMONTH':[2,1],'WEEKDAY':[2,1],
 'ROUND':[2,0],'QUOTIENT':[2,3],'MOD':[2,3],'POWER':[2,2],
 'SUM':[2,3],'AVERAGE':[2,3],'MIN':[2,3],'MAX':[2,3],'PRODUCT':[2,3],
 'COMBIN':[2,1],'PERMUT':[2,1],'PERMUTATIONA':[2,1]}.items()})
texts=['2',' 2 ','\t2','2\n','\u00a02','2\u00a0','2%','200%','(2)','+2','2e1','2E-1','2,000','2 000','R2','$2','2026/09/29','1:00','24:00','TRUE','2%%','2.0','2.','0x2']
for fn,args in specs.items():
 for i,text in enumerate(texts):
  changed=copy.deepcopy(args);changed[0]=g.t(text)
  g.emit(fn,f'direct-{i}',changed,axis='numeric_text_grammar')
  changed[0]=g.r('B1');g.emit(fn,f'reference-{i}',changed,[g.fix('B1',g.t(text))],axis='numeric_text_grammar')
for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111numtext-')
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
doc={'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]}
out.write_text(json.dumps(doc,ensure_ascii=False,indent=2),encoding='utf-8')
print(len(specs),'functions',len(g.cases),'rows')
