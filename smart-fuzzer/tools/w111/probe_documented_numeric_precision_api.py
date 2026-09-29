"""Compare public Automation formatting on exceptional decimal midpoints.

Only documented VarBstrFromR8, SysStringLen and SysFreeString calls are used.
This is a rejected/accepted research hypothesis, never an Excel authority.
"""
import ctypes,decimal,json,platform,struct
from collections import Counter
from pathlib import Path

def main():
    api=ctypes.WinDLL('oleaut32')
    api.VarBstrFromR8.argtypes=[ctypes.c_double,ctypes.c_uint32,ctypes.c_uint32,ctypes.POINTER(ctypes.c_void_p)]
    api.VarBstrFromR8.restype=ctypes.c_long
    api.SysStringLen.argtypes=[ctypes.c_void_p];api.SysStringLen.restype=ctypes.c_uint32
    api.SysFreeString.argtypes=[ctypes.c_void_p];api.SysFreeString.restype=None
    run=Path('smart-fuzzer/runs/w111-numeric-text-rendering-precision-typed-20260929')
    cases=list(map(json.loads,(run/'cases/cases.jsonl').read_text(encoding='utf-8-sig').splitlines()))
    excel={r['case_id']:r for r in map(json.loads,(run/'outcomes/excel.jsonl').read_text(encoding='utf-8-sig').splitlines())}
    rows=[];counts=Counter()
    for case in cases:
        if case['canonical_surface_name']!='ROUND' or case['args'][0]['value']<0:continue
        x=case['args'][0]['value'];ptr=ctypes.c_void_p();status=api.VarBstrFromR8(x,0x409,0,ctypes.byref(ptr))
        try:text=ctypes.wstring_at(ptr.value,api.SysStringLen(ptr)) if ptr.value else None
        finally:
            if ptr.value:api.SysFreeString(ptr)
        # Removing notation differences asks only whether this API supplies the
        # evidenced initial significant digits, not whether its display matches.
        value=float(decimal.Decimal(text));bits='0x'+struct.pack('>d',value).hex()
        expected=excel[case['case_id']]['outcome']['bits_hex'];counts['different']+=bits!=expected
        rows.append({'case_id':case['case_id'],'input_bits':struct.pack('>d',x).hex(),'hresult':status,'text':text,'parsed_bits':bits,'excel_bits':expected})
    target=Path('docs/function-lane/evidence/w111-broad-20260929/numeric-to-text/documented-api-precision.json')
    target.write_text(json.dumps({'api':'VarBstrFromR8','signature_source':'https://learn.microsoft.com/en-us/windows/win32/api/oleauto/nf-oleauto-varbstrfromr8','provenance':'Documented public C API calls only; no binary/internal inspection. Research hypothesis, not a production dependency.','platform':platform.platform(),'lcid':0x409,'flags':0,'rows':len(rows),'differences':counts['different'],'observations':rows},separators=(',',':')),encoding='utf-8')
    print(len(rows),'rows;',counts['different'],'differences')
if __name__=='__main__':main()
