"""Exhaustive BMP scalar character classes for ADDRESS sheet-name quoting.

ASCII/Latin1 were captured through the typed ingress-checked harness already.
These strings start with U+0100 or later, avoiding Excel's apostrophe/formula
entry prefixes. Surrogates are excluded because standalone code units need a
different, explicitly qualified transport. Supplementary characters sample all
surrogate lead and trail classes and boundaries with reproducible spacing.
"""
import json
from pathlib import Path
from gen_address_probe_20260929 import bits

rows=[]
for code in range(256,65536):
    if 0xd800 <= code <= 0xdfff:continue
    for text in [chr(code)+'z','z'+chr(code)+'z']:
        rows.append({'probe':{'id':f'address-unicode-{code:06x}-{"start" if text[0]!="z" else "middle"}',
                              'args':[bits(1),bits(1),bits(1),bits(1),text]}})
for code in sorted(set([0x10000,0x10ffff,0x1f600]+list(range(0x10000,0x110000,1024))+list(range(0x10000,0x10400)))):
    for text in [chr(code)+'z','z'+chr(code)+'z']:
        rows.append({'probe':{'id':f'address-unicode-{code:06x}-{"start" if text[0]!="z" else "middle"}',
                              'args':[bits(1),bits(1),bits(1),bits(1),text]}})
out=Path('smart-fuzzer/runs/w111-broad-20260929/address-unicode.json')
out.write_text(json.dumps({'function':'ADDRESS','probes':rows},ensure_ascii=False,separators=(',',':')),encoding='utf-8')
print(len(rows),out)
