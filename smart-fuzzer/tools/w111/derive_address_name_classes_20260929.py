"""Derive context character classes from an exhaustive, ingress-qualified sweep.

This classifies characters, not whole sheet-name answers. The grammar consuming
these classes must be independently tested with composed, unobserved strings.
"""
import gzip
import hashlib
import itertools
import json
from pathlib import Path

root=Path('smart-fuzzer/runs/w111-broad-20260929')
source=root/'address-unicode-answers.json'
capture=json.loads(source.read_text(encoding='utf-8-sig'))
ingress=json.loads((root/'address-unicode-ingress.json').read_text(encoding='utf-8-sig'))
assert ingress['failure_count']==0 and ingress['exact_ordinal_count']==130562
classes={'start':[],'middle':[]}
supplementary={'start':[],'middle':[]}
for w in capture['witnesses']:
    _,_,code,position=w['id'].split('-')
    cp=int(code,16);name=w['args'][-1]
    if w['expected_bits']=='text:'+name+'!$A$1':
        (classes if cp<65536 else supplementary)[position].append(cp)
    else:
        assert cp<65536 and w['expected_bits']=="text:'"+name+"'!$A$1"
assert len(supplementary['start'])==2049 and len(supplementary['middle'])==2049
ranges={}
for position,values in classes.items():
    ranges[position]=[]
    for _,group in itertools.groupby(enumerate(values),lambda pair:pair[1]-pair[0]):
        group=[pair[1] for pair in group]
        ranges[position].append([group[0],group[-1]])
dest=Path('docs/function-lane/evidence/w111-broad-20260929/address')
dest.mkdir(parents=True,exist_ok=True)
profile={'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
         'source_rows':len(capture['witnesses']),'capture_provenance':capture.get('capture_provenance'),
         'ingress':ingress,'grammar':'NameStart NameContinue*; external prefix and reserved/reference ambiguity checked separately',
         'unicode_scalar_bmp_range':[256,65535],'excluded_isolated_surrogates':[55296,57343],
         'supplementary_each_position':2049,'ranges':ranges}
(dest/'name-classes.json').write_text(json.dumps(profile,indent=2),encoding='utf-8')
# Retain the full oracle evidence compactly; this is worksheet observation data,
# not a session transcript. gzip mtime=0 makes regeneration deterministic.
(dest/'unicode-answers.json.gz').write_bytes(gzip.compress(source.read_bytes(),mtime=0))
parts=['//! ADDRESS name grammar character classes, Excel build20430/CV2.\n'
       '//! Derived from exhaustive qualified BMP observations, not whole-name answers.\n'
       '//! See evidence/w111-broad-20260929/address/name-classes.json and independent composed-name validation.\n\n']
for key,constant in [('start','NAME_START'),('middle','NAME_CONTINUE')]:
    parts.append(f'pub(super) const {constant}: &[(u16, u16)] = &[\n')
    parts.extend(f'    (0x{a:04X}, 0x{b:04X}),\n' for a,b in ranges[key])
    parts.append('];\n\n')
Path('crates/oxfunc_core/src/functions/address_name_classes.rs').write_text(''.join(parts),encoding='utf-8')
print({k:len(v) for k,v in ranges.items()})
