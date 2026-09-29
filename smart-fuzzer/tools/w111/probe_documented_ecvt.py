"""Public documented CRT API hypothesis only; no binary/internal inspection.

https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/ecvt-s
The outputs are research evidence, never a dependency or Excel authority.
"""
import ctypes,json,platform,struct
from collections import Counter
from pathlib import Path

def main():
    run=Path('smart-fuzzer/runs/w111-numeric-text-rendering-precision-typed-20260929')
    cases=list(map(json.loads,(run/'cases/cases.jsonl').read_text(encoding='utf-8-sig').splitlines()))
    excel={r['case_id']:r for r in map(json.loads,(run/'outcomes/excel.jsonl').read_text(encoding='utf-8-sig').splitlines())}
    counts=Counter();observations=[]
    for dll in ['ucrtbase.dll','msvcrt.dll']:
        lib=ctypes.CDLL(dll);fn=lib._ecvt_s
        fn.argtypes=[ctypes.c_char_p,ctypes.c_size_t,ctypes.c_double,ctypes.c_int,ctypes.POINTER(ctypes.c_int),ctypes.POINTER(ctypes.c_int)];fn.restype=ctypes.c_int
        for case in cases:
            if case['canonical_surface_name']!='ROUND' or case['args'][0]['value']<0:continue
            n=case['args'][0]['value'];buffer=ctypes.create_string_buffer(512);point=ctypes.c_int();sign=ctypes.c_int();rc=fn(buffer,512,n,15,ctypes.byref(point),ctypes.byref(sign))
            digits=buffer.value.decode('ascii');number=float(f'{digits}e{point.value-len(digits)}');bits='0x'+struct.pack('>d',number).hex();expected=excel[case['case_id']]['outcome']['bits_hex']
            counts[dll]+=bits!=expected;observations.append({'dll':dll,'case_id':case['case_id'],'input_bits':struct.pack('>d',n).hex(),'return':rc,'digits':digits,'point':point.value,'sign':sign.value,'rounded_bits':bits,'excel_bits':expected})
    out={'research_only':True,'documented_api':'_ecvt_s','documentation':'https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/ecvt-s','platform':platform.platform(),'differences':dict(counts),'observations':observations}
    Path('docs/function-lane/evidence/w111-broad-20260929/numeric-to-text/documented-ecvt.json').write_text(json.dumps(out,indent=2),encoding='utf-8');print(counts)
if __name__=='__main__':main()
