"""ADDRESS typed, omitted-argument, sheet-name and broadcast discovery packet.

Run-ArraySupportTranche.ps1 consumes OUT. Numeric cells use Value2; references
remain references locally, blank cells stay blank, omitted positions stay omitted.
Text literals and reference text are intentionally tested as separate cases.
"""
import argparse
import json
from pathlib import Path


def n(x): return {'kind': 'number', 'value': x}
def t(x): return {'kind': 'text', 'value': x}
def b(x): return {'kind': 'logical', 'value': x}
def e(x): return {'kind': 'error', 'code': x}
def a(rows): return {'kind': 'array', 'rows': rows}
MISSING = {'kind': 'missing_arg'}
EMPTY = {'kind': 'empty_cell'}
ERRORS = {'Div0': '1/0', 'Value': 'VALUE("x")', 'Ref': 'INDIRECT("#REF!")',
          'NA': 'NA()', 'Num': 'SQRT(-1)', 'Name': 'nonexistent_name_address_probe'}


def build():
    cases = []

    def add(args, tag, references=False):
        local, formula, fixtures = [], [], []
        for i, arg in enumerate(args):
            kind = arg['kind']
            target = f'A{1 + 10*i}'
            if kind == 'missing_arg':
                local.append(arg)
                formula.append('')
            elif references or kind in ['number', 'empty_cell', 'array']:
                if kind == 'array':
                    rows, cols = len(arg['rows']), len(arg['rows'][0])
                    # A separate 10-row band for each argument avoids overlap.
                    start = 1 + 10*i
                    target = f'A{start}:{chr(ord("A")+cols-1)}{start+rows-1}'
                fixtures.append({'target': target, 'value': arg})
                local.append({'kind': 'reference', 'reference_kind': 'Area' if kind == 'array' else 'A1', 'target': target})
                formula.append(target)
            elif kind == 'text':
                local.append(arg)
                formula.append('"' + arg['value'].replace('"', '""') + '"')
            elif kind == 'logical':
                local.append(arg)
                formula.append('TRUE' if arg['value'] else 'FALSE')
            elif kind == 'error':
                local.append(arg)
                formula.append(ERRORS[arg['code']])
            else:
                raise ValueError(kind)
        case_id = f'address-typed-20260929-{len(cases):04d}-{tag}'
        cases.append({'schema_version': 'oxfunc.smart_fuzzer.scenario_seed_case.v0',
                      'run_id': 'assigned_by_runner', 'tranche_id': 'address-typed-20260929',
                      'case_id': case_id, 'function_id': 'FUNC.ADDRESS', 'canonical_surface_name': 'ADDRESS',
                      'case_tag': tag, 'axis': 'typed_address', 'expected_probe_class': 'reference_value_call',
                      'formula_text': '=ADDRESS(' + ','.join(formula) + ')', 'args': local,
                      'cell_fixture': fixtures, 'formula_cell': 'J70', 'category': 'Lookup & Reference',
                      'blocked_or_deferred_lanes': [], 'known_deviation_tags': []})

    values = [n(0), n(-1), n(.5), n(1), n(4.9), t(''), t('1'), t(' 2 '),
              t('TRUE'), t('FALSE'), t('true'), t('x'), b(True), b(False),
              EMPTY, MISSING] + [e(code) for code in ERRORS]
    for axis in range(5):
        for index, value in enumerate(values):
            for references in [False, True]:
                args = [n(3), n(2), n(1), b(True), t('Alpha')]
                args[axis] = value
                add(args, f'axis{axis}-value{index}-{"ref" if references else "literal"}', references)
    for name in ['', 'Alpha', 'Quarter 1', "O'Brien", 'a.b', 'a_b', '123', '1A', 'A1',
                 'R1C1', 'R', 'C', 'TRUE', 'a-b', 'a+b', 'a!b', 'a:b', '[Book]Sheet',
                 'é', '漢字', 'a\nb', 'a\tb', ' a', 'a ', "'Alpha'", 'x'*255, 'x'*256]:
        for style in [False, True]:
            add([n(3), n(2), n(4), b(style), t(name)], f'sheet-{len(cases)}')
    for axis in range(5):
        for array in [a([[n(-1), n(0), n(1), n(2)]]), a([[n(1)], [n(2)]]),
                      a([[n(1), e('NA')], [n(3), t('x')]]),
                      a([[t('Alpha'), t('Quarter 1'), t("O'Brien")]])]:
            args = [n(3), n(2), n(4), b(False), t('Alpha')]
            args[axis] = array
            add(args, f'broadcast-axis{axis}-{len(cases)}')
    add([a([[n(0)], [n(-1)], [n(1)]]), a([[n(-1), n(0), n(1)]]), n(4), b(False)], 'broadcast-outer')
    for args in [[n(3), n(2)], [n(3), n(2), MISSING], [n(3), n(2), MISSING, MISSING],
                 [n(3), n(2), MISSING, MISSING, MISSING], [n(3), n(2), MISSING, b(False)],
                 [n(3), n(2), MISSING, b(False), t('')]]:
        add(args, f'optional-{len(cases)}')
    return {'schema_version': 'oxfunc.smart_fuzzer.scenario_seed_case_set.v0',
            'authority': 'non_semantic_exploration_input', 'generator': Path(__file__).name,
            'tranche_id': 'address-typed-20260929', 'comparison_policy': 'exact_typed_bit_match_no_tolerance',
            'cases': cases, 'tranches': [], 'skipped': [],
            'summary': {'case_count': len(cases), 'surfaces_covered': 1}}


def build_error_order():
    cases = []
    for row, col, mode, style in [(0, 1, 1, 1), (-1, 1, 1, 1), (1048577, 1, 1, 1),
                                  (1, 16385, 1, 1), (0, 0, 4, 0), (-1, -1, 4, 0),
                                  (1, 1, 0, 1)]:
        for error_axis in [1, 2, 3, 4]:
            for code in ['NA', 'Div0', 'Ref']:
                values = [n(row), n(col), n(mode), n(style), t('Alpha')]
                values[error_axis] = e(code)
                fixtures = [{'target': f'A{1+10*i}', 'value': value} for i, value in enumerate(values)]
                args = [{'kind': 'reference', 'reference_kind': 'A1', 'target': fixture['target']}
                        for fixture in fixtures]
                case_id = f'address-error-order-20260929-{len(cases):04d}'
                cases.append({'schema_version': 'oxfunc.smart_fuzzer.scenario_seed_case.v0',
                              'run_id': 'assigned_by_runner', 'tranche_id': 'address-error-order-20260929',
                              'case_id': case_id, 'function_id': 'FUNC.ADDRESS', 'canonical_surface_name': 'ADDRESS',
                              'case_tag': 'coordinate-selector-error-order', 'axis': 'error_precedence',
                              'expected_probe_class': 'reference_value_call',
                              'formula_text': '=ADDRESS(' + ','.join(x['target'] for x in fixtures) + ')',
                              'args': args, 'cell_fixture': fixtures, 'formula_cell': 'J70',
                              'category': 'Lookup & Reference', 'blocked_or_deferred_lanes': [], 'known_deviation_tags': []})
    result = build()
    result.update({'tranche_id': 'address-error-order-20260929', 'cases': cases,
                   'summary': {'case_count': len(cases), 'surfaces_covered': 1}})
    return result


def build_sheet_discovery():
    names = ['', '_', '.', '\\', 'R', 'r', 'C', 'c', 'RC', 'rc', 'TRUE', 'True', 'false',
             'A0', 'A00', 'A01', 'A1', 'A1048576', 'A1048577', 'XFD1', 'XFE1', 'ZZZ1', 'AAAA1',
             'R0', 'R1', 'R1048576', 'R1048577', 'C0', 'C1', 'C16384', 'C16385',
             'R0C0', 'R1C1', 'RC1', 'R1C', 'R[1]C[1]', 'R[-1]C', 'r01c02',
             '[Book]Sheet', '[Book]', '[Book Name]Sheet', '[Book]Sheet Name', 'a[Book]Sheet',
             '[a][b]', '[]a', '[a]A1', '[a]TRUE', '[a]1a', '[a]R', '[a]é',
             'é', '漢字', '😀', 'a😀', '١a', '１a', 'e\u0301', '\u0301a', '€', 'a€',
             '1a', '0abc', 'a1a', 'a.b', 'a_b', 'a\\b', 'a..b']
    names += [chr(i) for i in range(32, 127)]
    names += ['a' + chr(i) + 'b' for i in range(32, 127)]
    cases = []

    def add(name, style, row=1048576, col=16384, mode=1):
        values = [n(row), n(col), n(mode), b(style), t(name)]
        fixtures = [{'target': f'A{1+10*i}', 'value': value} for i, value in enumerate(values)]
        case_id = f'address-sheet-discovery-20260929-{len(cases):04d}'
        cases.append({'schema_version': 'oxfunc.smart_fuzzer.scenario_seed_case.v0',
                      'run_id': 'assigned_by_runner', 'tranche_id': 'address-sheet-discovery-20260929',
                      'case_id': case_id, 'function_id': 'FUNC.ADDRESS', 'canonical_surface_name': 'ADDRESS',
                      'case_tag': 'sheet-quoting-and-output-length', 'axis': 'sheet_text',
                      'expected_probe_class': 'reference_value_call', 'formula_text': '=ADDRESS(A1,A11,A21,A31,A41)',
                      'args': [{'kind': 'reference', 'reference_kind': 'A1', 'target': f['target']} for f in fixtures],
                      'cell_fixture': fixtures, 'formula_cell': 'J70', 'category': 'Lookup & Reference',
                      'blocked_or_deferred_lanes': [], 'known_deviation_tags': []})

    for name in dict.fromkeys(names):
        for style in [False, True]:
            add(name, style)
    for length in [240, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255, 256, 257]:
        for name in ['x'*length, ' '*length, "'"*length, 'é'*length, '😀'*(length//2) + ('x' if length%2 else '')]:
            for style in [False, True]:
                add(name, style)
    result = build()
    result.update({'tranche_id': 'address-sheet-discovery-20260929', 'cases': cases,
                   'summary': {'case_count': len(cases), 'surfaces_covered': 1}})
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('out', type=Path)
    p.add_argument('--error-order', action='store_true', help='Separate coordinate/selector error-precedence supplement')
    p.add_argument('--sheet-discovery', action='store_true', help='Sheet token grammar and UTF-16 length discriminator')
    args = p.parse_args()
    result = build_error_order() if args.error_order else build_sheet_discovery() if args.sheet_discovery else build()
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding='utf-8')
    print(len(result['cases']), args.out)


if __name__ == '__main__':
    main()
