"""Public VarBstrFromR8 API output comparison; no Excel or binary inspection.

The API is a black-box hypothesis, not a production dependency or semantic
authority. Its documented signature is linked in the resulting artifact.
"""
import ctypes,hashlib,json,platform
from pathlib import Path

def main():
    source=Path('smart-fuzzer/runs/w111-numeric-to-text-discovery-20260929/design.json')
    design=json.loads(source.read_text(encoding='utf-8'))
    api=ctypes.WinDLL('oleaut32')
    api.VarBstrFromR8.argtypes=[ctypes.c_double,ctypes.c_uint32,ctypes.c_uint32,ctypes.POINTER(ctypes.c_void_p)]
    api.VarBstrFromR8.restype=ctypes.c_long
    api.SysStringLen.argtypes=[ctypes.c_void_p];api.SysStringLen.restype=ctypes.c_uint32
    api.SysFreeString.argtypes=[ctypes.c_void_p];api.SysFreeString.restype=None
    rows=[]
    for value in design['inputs']:
        ptr=ctypes.c_void_p();status=api.VarBstrFromR8(value['value'],0x409,0,ctypes.byref(ptr))
        try:text=ctypes.wstring_at(ptr.value,api.SysStringLen(ptr)) if ptr.value else None
        finally:
            if ptr.value:api.SysFreeString(ptr)
        rows.append({'bits':value['bits'],'hresult':status,'text':text})
    out={'api':'VarBstrFromR8','signature_source':'https://learn.microsoft.com/en-us/windows/win32/api/oleauto/nf-oleauto-varbstrfromr8',
        'provenance':'Documented public C API call only; no binary internals inspected. Hypothesis comparison, never a semantic authority.',
        'platform':platform.platform(),'lcid':0x409,'flags':0,'input_design_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'observations':rows}
    target=Path('docs/function-lane/evidence/w111-broad-20260929/numeric-to-text/documented-api.json');target.write_text(json.dumps(out,separators=(',',':')),encoding='utf-8')
    print(len(rows),'public API observations retained')

if __name__=='__main__':main()
