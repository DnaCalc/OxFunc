"""Second ADDRESS discriminators, frozen before the next oracle observations."""
import json
import math
from pathlib import Path
from gen_address_probe_20260929 import bits
from gen_address_typed_20260929 import n, t, b, e, build


def typed():
    cases = []
    def add(values, tag):
        fixtures = [{'target': f'A{1+10*i}', 'value': v} for i, v in enumerate(values)]
        cases.append({'schema_version': 'oxfunc.smart_fuzzer.scenario_seed_case.v0',
                      'run_id': 'assigned_by_runner', 'tranche_id': 'address-refinement-20260929',
                      'case_id': f'address-refinement-20260929-{len(cases):05d}',
                      'function_id': 'FUNC.ADDRESS', 'canonical_surface_name': 'ADDRESS',
                      'case_tag': tag, 'axis': tag, 'expected_probe_class': 'reference_value_call',
                      'formula_text': '=ADDRESS(A1,A11,A21,A31,A41)',
                      'args': [{'kind': 'reference', 'reference_kind': 'A1', 'target': x['target']} for x in fixtures],
                      'cell_fixture': fixtures, 'formula_cell': 'J70', 'category': 'Lookup & Reference',
                      'blocked_or_deferred_lanes': [], 'known_deviation_tags': []})
    for i in range(5):
        for j in range(i+1, 5):
            for ec1, ec2 in [('Div0', 'NA'), ('NA', 'Div0'), ('Ref', 'Num')]:
                v = [n(3), n(2), n(1), b(True), t('Alpha')]
                v[i], v[j] = e(ec1), e(ec2)
                add(v, 'explicit-error-order')
    for i in range(5):
        for j in range(5):
            if i == j: continue
            v = [n(3), n(2), n(1), b(True), t('Alpha')]
            v[i], v[j] = t('bad numeric') if i < 4 else t('x'*256), e('NA')
            add(v, 'coercion-before-error')
    for s in [' TRUE', 'TRUE ', ' TRUE ', '\tTRUE', 'TRUE\n', '\u00a0TRUE', 'TrUe', ' false ',
              '0', '-1', '1.0', '1e0', '0%', '', ' ', 'true\x00', '\x00true']:
        add([n(3), n(2), n(1), t(s), t('Alpha')], 'style-text')
    for length in [125,126,127,128,129,250,251,252,253,254,255,256]:
        for s in ["'"*length, 'x'*(length-1)+"'", "'"+'x'*(length-1), ' '*length]:
            for mode, style, row, col in [(1,True,1048576,16384),(4,False,-1048575,-16383),
                                          (4,False,0,0),(4,True,1,1)]:
                add([n(row),n(col),n(mode),b(style),t(s)], 'escaped-length-and-rendering')
    for code in list(range(128,256))+[0,9,10,13,0x300,0x301,0x36f,0x37e,0x387,0x660,0x669,
                                    0x6f0,0x966,0x2000,0x200c,0x200d,0x2028,0x20ac,0x2102,
                                    0x2160,0x221e,0x3000,0x3042,0x4e2d,0xff10,0xff21,0x1f600]:
        for s in [chr(code)+'z', 'z'+chr(code)+'z']:
            add([n(1),n(1),n(1),b(True),t(s)], 'unicode-identifier')
    for s in ['[a.b]x','[a-b]x','[a?b]x','[a\\b]x','[1]x','[a]','[a].x','[a]\\x','[a]_x',
              '[a]FALSE','[a]RC','[a]x?y','[a]x.y','[a]😀','[a]١','[a]１', '[a]x[y',
              'R1048576C16384','R1048577C16384','R1048576C16385','RC16384','RC16385',
              'R1048576C','R1048577C','R000C1','R1C000','R01C01','R999999999999999999C1']:
        add([n(1),n(1),n(1),b(True),t(s)], 'sheet-reference-token')
    result = build()
    result.update({'tranche_id': 'address-refinement-20260929', 'cases': cases,
                   'summary': {'case_count': len(cases), 'surfaces_covered': 1}})
    return result


def floor_zoom():
    rows = []
    for axis, anchors in [(0,[-1048575,-16383,-10,-1,0,1,2,10,16383,1048575]),
                          (1,[-16383,-1,0,1,10,16383]),(2,[1,2,3,4,5])]:
        for anchor in anchors:
            eps=2**(-22 if anchor>0 else -23)
            for i in range(-32,65):
                # Fixed absolute increments locate additional arithmetic rounding;
                # unlike the prior packet these do not shrink with input ULP.
                x=float(anchor)-(eps+i*2**-38)
                args=[1,1,4,0];args[axis]=x;rows.append(args)
    seen=set();probes=[]
    for row in rows:
        a=tuple(bits(x) for x in row)
        if a in seen:continue
        seen.add(a)
        probes.append({'probe': {'id': f'address-floor-zoom-20260929-{len(probes):05d}', 'args':list(a)}})
    return {'function':'ADDRESS','probes':probes}


if __name__=='__main__':
    out=Path('smart-fuzzer/runs/w111-broad-20260929')
    for name,result in [('address-refinement.json',typed()),('address-floor-zoom.json',floor_zoom())]:
        (out/name).write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
        print(name,len(result.get('cases',result.get('probes',[]))))
