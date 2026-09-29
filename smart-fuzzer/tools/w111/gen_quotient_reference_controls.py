"""Separate QUOTIENT breadth controls; no inference from MROUND outcomes."""
import hashlib, json, sys
from pathlib import Path
source=Path('smart-fuzzer/runs/w111-mround-reference-controls-typed-20260929/typed.json')
data=json.loads(source.read_text())
data['tranche_id']='w111-quotient-reference-controls-typed-20260929'
data['generator']=Path(__file__).name
data['selection_template_sha256']=hashlib.sha256(source.read_bytes()).hexdigest()
for case in data['cases']:
    case['tranche_id']=data['tranche_id']
    case['case_id']=case['case_id'].replace('mround','quotient')
    case['function_id']='FUNC.QUOTIENT'
    case['canonical_surface_name']='QUOTIENT'
    case['formula_text']=case['formula_text'].replace('MROUND(','QUOTIENT(')
    case['axis']=case['axis'].replace('mround','quotient')
data['tranches']=[dict(tranche_id=data['tranche_id'],case_ids=[c['case_id'] for c in data['cases']])]
out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(data,separators=(',',':')))
print(len(data['cases']),out)
