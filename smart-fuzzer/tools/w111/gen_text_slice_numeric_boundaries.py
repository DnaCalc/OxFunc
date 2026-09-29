"""Numeric count/start probes distinguish tolerant integer rules and width."""
import json,math
from pathlib import Path
import gen_broad_typed_20260929 as g

def main():
    g.TRANCHE='w111-text-slice-numeric-boundaries-20260929'
    values=set([-1e20,-1,-0.5,-1e-20,0,0.5,1,1.5,2,32767,32768,65535,65536,1e20,1e308])
    threshold=2**-22+2**-33
    for center in [0,1,2,7,31,64]:
        for distance in [0,2**-22,threshold,2**-23,threshold/2,1e-15,1e-12,1e-9,1e-6]:
            x=center-distance
            values.update([math.nextafter(x,-math.inf),x,math.nextafter(x,math.inf)])
    for center in [2**15,2**16,2**31-1,2**31,2**32-1,2**32,2**53]:
        for offset in [-1,-0.5,0,0.5,1]:
            x=float(center)+offset;values.update([math.nextafter(x,-math.inf),x,math.nextafter(x,math.inf)])
    # Exclude signed zero/subnormal inputs that Value2 changes, before capture.
    values=sorted(x for x in values if x==0 or abs(x)>=2**-1022)
    text='ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!?'
    for fn in ['LEFT','RIGHT','LEFTB','RIGHTB']:
        for i,value in enumerate(values):
            g.emit(fn,f'count-{i}',[g.t(text),g.n(value)],axis='text_count_binary_boundaries')
    for fn in ['MID','MIDB']:
        for i,value in enumerate(values):
            g.emit(fn,f'count-{i}',[g.t(text),g.n(2),g.n(value)],axis='text_mid_count_binary_boundaries')
            g.emit(fn,f'start-{i}',[g.t(text),g.n(value),g.n(3)],axis='text_start_binary_boundaries')
    for case in g.cases:case['case_id']=case['case_id'].replace('w111typed-','w111textnumeric-')
    out=Path('smart-fuzzer/runs/w111-text-slice-numeric-boundaries-20260929');out.mkdir(parents=True,exist_ok=True)
    (out/'typed.json').write_text(json.dumps({'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0','tranche_id':g.TRANCHE,'cases':g.cases,'tranches':[{'tranche_id':g.TRANCHE,'case_ids':[c['case_id'] for c in g.cases]}]},separators=(',',':')),encoding='utf-8')
    print(len(values),len(g.cases))
if __name__=='__main__':main()
