"""Call only the documented C asin API; never inspect binary contents."""
import ctypes,json,struct,sys
from pathlib import Path

base=Path(sys.argv[1]);out=Path(sys.argv[2]);reports=[]
for lib in ['msvcrt.dll','ucrtbase.dll']:
    fn=ctypes.CDLL(lib).asin
    fn.argtypes=[ctypes.c_double];fn.restype=ctypes.c_double
    for inp in ['discovery.json','isolation-asin.json']:
        rows=json.loads((base/inp).read_text())['witnesses'];miss=[]
        for row in rows:
            x=struct.unpack('<d',struct.pack('<Q',int(row['args'][0],16)))[0]
            y=fn(x)
            actual='error:Num' if not -1<=x<=1 else f'0x{struct.unpack("<Q",struct.pack("<d",y))[0]:016x}'
            if actual!=row['expected_bits']:miss.append(dict(row,actual=actual))
        reports.append(dict(library=lib,public_api='double asin(double)',capture=inp,
                            rows=len(rows),matches=len(rows)-len(miss),misses=miss))
out.write_text(json.dumps(dict(documented_signature='https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/asin-asinf-asinl?view=msvc-170',
                              authority='documented_public_C_API_observation_only',reports=reports),indent=2)+'\n')
for row in reports:print(row['library'],row['capture'],row['matches'],row['rows'])
