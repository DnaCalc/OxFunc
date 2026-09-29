"""Describe generic formatter evidence and test non-authoritative hypotheses."""
import decimal,hashlib,json,struct,sys,urllib.parse
from collections import Counter
from pathlib import Path

def bits(n):return struct.pack('>d',n).hex()
def address_model(n):
    if not n:return '0'
    d=decimal.Decimal.from_float(abs(n))
    q=d.quantize(decimal.Decimal(1).scaleb(d.adjusted()-14),rounding=decimal.ROUND_HALF_DOWN)
    fixed=format(q,'f').rstrip('0').rstrip('.') if q.as_tuple().exponent<0 else format(q,'f')
    if len(fixed)<=20:result=fixed
    else:
        if abs(q.adjusted())>=99:q=q.quantize(decimal.Decimal(1).scaleb(q.adjusted()-13),rounding=decimal.ROUND_HALF_UP)
        exponent=q.adjusted();mantissa=format(q.scaleb(-exponent),'f').rstrip('0').rstrip('.')
        result=f'{mantissa}E{"-" if exponent<0 else "+"}{abs(exponent):02}'
    return ('-' if n<0 else '')+result
def main():
    root=Path('smart-fuzzer/runs/w111-numeric-to-text-discovery-typed-20260929')
    cases=[json.loads(x) for x in (root/'cases/cases.jsonl').read_text(encoding='utf-8-sig').splitlines()]
    outcomes={x['case_id']:x for x in map(json.loads,(root/'outcomes/excel.jsonl').read_text(encoding='utf-8-sig').splitlines())}
    assert len(cases)==7048 and len(outcomes)==len(cases),'oracle capture is not yet a full packet'
    api=json.loads(Path('docs/function-lane/evidence/w111-broad-20260929/numeric-to-text/documented-api.json').read_text(encoding='utf-8'))
    api={x['bits']:x['text'] for x in api['observations']}
    strings={};withheld=[]
    for c in cases:
        o=outcomes[c['case_id']]
        if o['execution_status']!='ok':withheld.append({'case':c,'excel':o});continue
        if c['canonical_surface_name']=='LEFT':strings[bits(c['args'][0]['value'])]=o['outcome']
    cross=Counter();cross_misses=[];models=Counter();model_misses=[];shape=Counter();errors=[];address_misses=[]
    decimal.getcontext().prec=1000
    for key,o in strings.items():
        if o['kind']!='text':errors.append({'bits':key,'outcome':o});continue
        actual=o['value'];n=struct.unpack('>d',bytes.fromhex(key))[0]
        candidate=address_model(n)
        if actual==candidate:models['existing_ADDRESS_model_exact_text']+=1
        else:address_misses.append({'bits':key,'excel':actual,'address_model':candidate})
        shape['scientific' if 'E' in actual else 'fixed']+=1
        if actual==api.get(key):models['documented_VarBstrFromR8_exact_text']+=1
        d=decimal.Decimal.from_float(abs(n));answer=decimal.Decimal(actual).copy_abs()
        row={'bits':key,'excel':actual,'api':api.get(key)}
        for mode in [decimal.ROUND_HALF_EVEN,decimal.ROUND_HALF_UP,decimal.ROUND_HALF_DOWN]:
            q=d.quantize(decimal.Decimal(1).scaleb(d.adjusted()-14),rounding=mode) if d else d
            if q==answer:models['exact15_'+mode]+=1
            else:row[mode]=str(q)
        if any(mode in row for mode in [decimal.ROUND_HALF_EVEN,decimal.ROUND_HALF_UP,decimal.ROUND_HALF_DOWN]):model_misses.append(row)
    for c in cases:
        fn=c['canonical_surface_name'];o=outcomes[c['case_id']]
        if o['execution_status']!='ok' or fn in ['LEFT','ADDRESS','COMPLEX']:continue
        n=c['args'][2 if fn=='TEXTJOIN' else 0]['value'];base=strings[bits(n)]
        if base['kind']!='text':continue
        text=base['value']
        if fn=='LEN':expected='number:0x'+bits(float(len(text)))
        elif fn=='EXACT':expected='logical:'+str(text==c['args'][1]['value']).lower()
        else:
            if fn=='ENCODEURL':text=urllib.parse.quote(text,safe='-_.~')
            elif fn=='LOWER':text=text.lower()
            elif fn=='UPPER':text=text.upper()
            expected='text:'+text
        if expected==o['outcome']['digest_payload']:cross[fn+'_consistent']+=1
        else:cross_misses.append({'case':c,'excel':o,'expected_from_LEFT':expected})
    report={'source_run':root.name,'inputs':len(strings),'format_shape':dict(shape),'model_matches':dict(models),'model_discriminators':model_misses,'nontext_LEFT':errors,'cross_function_matches':dict(cross),'cross_function_differences':cross_misses,'withheld':withheld,
        'address_model_differences':address_misses,'source_sha256':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [root/'cases/cases.jsonl',root/'outcomes/excel.jsonl']}}
    p=Path('docs/function-lane/evidence/w111-broad-20260929/numeric-to-text/discovery-analysis.json');p.write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(json.dumps({k:v for k,v in report.items() if k not in ['model_discriminators','cross_function_differences','withheld','source_sha256']},indent=2));print('cross_misses',len(cross_misses),'model_discriminators',len(model_misses),'withheld',len(withheld))

if __name__=='__main__':main()
