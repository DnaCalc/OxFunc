"""Typed radix grammar/coercion discriminators; transport through existing runner."""
import hashlib
import json
import sys
from pathlib import Path
import gen_broad_typed_20260929 as g

for fn in ['BIN2DEC','BIN2HEX','BIN2OCT','OCT2DEC','OCT2BIN','OCT2HEX','HEX2DEC','HEX2BIN','HEX2OCT']:
    for value in [g.t(x) for x in ['', ' ', ' 1', '1 ', '\t1', '\n1', '\u00a01', '+1', '-1', '0x10', '1e1', '00000000000', '00000000001', 'fffffffffe', 'FFFFFFFFFF', '1111111111', '7777777777', 'A', 'a', '.1', '1.0']]+[g.blank(),g.missing(),g.b(False),g.b(True),g.n(0),g.n(1.1),g.e(),g.e('Div0')]:
        g.emit(fn,'grammar',[value])
        if not fn.endswith('2DEC'):
            for places in [g.n(0),g.n(1),g.n(10.9),g.n(11),g.b(True),g.b(False),g.t('2'),g.t(''),g.blank(),g.missing(),g.e('Ref')]:
                g.emit(fn,'grammar-places',[value,places])
    for value in [g.t(''),g.t('1'),g.b(True),g.e()]:
        g.emit(fn,'typed-array',[g.a([[value,value]])])

for fn in ['DEC2BIN','DEC2HEX','DEC2OCT']:
    for value in [g.n(-1),g.n(0),g.n(.9),g.n(1),g.t(''),g.t(' '),g.t('1'),g.t('-0.9'),g.b(True),g.b(False),g.blank(),g.missing(),g.e(),g.e('Value')]:
        g.emit(fn,'number-kind',[value])
        for places in [g.n(-1),g.n(0),g.n(.9),g.n(1),g.n(10.9),g.n(11),g.t(''),g.t('2'),g.b(True),g.b(False),g.blank(),g.missing(),g.e('Ref')]:
            g.emit(fn,'places-kind',[value,places])

for value in [g.n(-.9),g.n(0),g.n(.9),g.n(1),g.t(''),g.t('1'),g.b(True),g.b(False),g.blank(),g.missing(),g.e()]:
    for radix in [g.n(2),g.n(36.9),g.t('10'),g.b(True),g.missing(),g.e('Div0')]:
        g.emit('BASE','typed',[value,radix])
        for length in [g.n(0),g.n(255.9),g.t(''),g.b(True),g.missing(),g.e('Ref')]:
            g.emit('BASE','typed-length',[value,radix,length])

out=Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
doc={'schema_version':'oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
     'authority':'non_semantic_exploration_input','generator':str(Path(__file__)),
     'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
     'tranche_id':'w111-radix-typed-20260929','cases':g.cases}
out.write_text(json.dumps(doc,ensure_ascii=False),encoding='utf-8')
print(len(g.cases),out)
