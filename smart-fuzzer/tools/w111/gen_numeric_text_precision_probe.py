"""Distinguish finite intermediate decimal rounding from exact 15-digit rounding."""
import decimal,hashlib,json,math
from collections import Counter
from pathlib import Path
import gen_broad_typed_20260929 as g
from analyze_numeric_to_text_discovery import address_model

def main():
    g.TRANCHE='w111-numeric-text-rendering-precision-20260929'
    source=Path('docs/function-lane/evidence/w111-broad-20260929/numeric-to-text/precision-search.json')
    rows=json.loads(source.read_text(encoding='utf-8'))['differences'];hypotheses=[]
    decimal.getcontext().prec=1100
    for i,row in enumerate(rows):
        base=row['value'];pool=[('center',base)]
        if i%7==0:pool += [('below',math.nextafter(base,0)),('above',math.nextafter(base,math.inf))]
        for kind,number in pool:
            for sign in [1,-1]:
                n=number*sign;tag=f'{i}-{kind}-{sign}';digits=14-decimal.Decimal.from_float(number).adjusted()
                g.emit('LEFT',tag,[g.n(n),g.n(32767)],axis='rational_midpoint_precision_discriminator')
                g.emit('ADDRESS',tag,[g.n(1),g.n(1),g.n(1),g.b(True),g.n(n)],axis='rational_midpoint_precision_discriminator')
                g.emit('ROUND',tag,[g.n(n),g.n(digits)],axis='initial_decimal_rounding_control')
                if kind=='center' and sign==1:hypotheses.append({'source':row,'exact_generic_model':address_model(n),'round_digits':digits})
    for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111renderprecision-')
    out=Path('smart-fuzzer/runs/w111-numeric-text-rendering-precision-20260929');out.mkdir(parents=True,exist_ok=True)
    (out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
    design={'rows':len(g.cases),'functions':dict(Counter(c['canonical_surface_name'] for c in g.cases)),'input_search_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'hypotheses':hypotheses,'intent':'Compare frozen30-decimal-helper with exact15-rounding; no expected Excel behavior assumed.'}
    (out/'design.json').write_text(json.dumps(design,indent=2),encoding='utf-8');print(len(g.cases),design['functions'])
if __name__=='__main__':main()
