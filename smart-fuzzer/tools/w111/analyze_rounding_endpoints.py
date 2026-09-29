"""Retain public endpoint controls and test a uniform decimal-publication hypothesis.

The unrestricted-exponent model is research only. It is not an assertion that
IEEE nonfinite encodings have ordinary IEEE semantics in a worksheet.
"""
import collections
import hashlib
import json
import shutil
from pathlib import Path

from analyze_decimal_parser_scaling import F, nibble64, rn
from model_rounding_boundaries import decode, encode, i32, pair

RUN = Path('smart-fuzzer/runs/w111-rounding-endpoint-refinement-20260929')
OUT = Path('docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries')


def decimal_encoding(significand, scale, raw=False, initial=False):
    if not significand or scale < -323:
        return 0
    if scale > 308:
        return 0x7ff0000000000000
    value = rn(nibble64(significand, scale), 53)
    exponent = value.numerator.bit_length() - value.denominator.bit_length()
    if value < F(2) ** exponent:
        exponent -= 1
    mantissa = int(value / F(2) ** (exponent - 52))
    if exponent < -1022 and not initial:
        return 0
    if not raw:
        if exponent > 1023:
            return 0x7ff0000000000000
    # The raw layout is tested only for -1023 <= exponent <= 1024.
    assert -1023 <= exponent <= 1024
    return ((exponent + 1023) << 52) | (mantissa & ((1 << 52) - 1))


def predict(function, args, preserve_overflow=False, difference_policy='decimal'):
    number, count = map(decode, args)
    if number == 0 or abs(number) < 2**-1022:
        return encode(0.0)
    significand, scale = pair(number, function != 'ROUND')
    directed = function != 'ROUND'
    original = min(int(abs(count)), (2**32 - 1) if directed else (2**31 - 1))
    if directed:
        original &= 65535
    if count < 0:
        original = -original
    decimal_count = original
    if directed and abs(original) > 32767:
        decimal_count = -100 if original < 0 else 100
    dropped = -(scale + decimal_count) if directed else 15 - i32(original + scale + 15)
    initial = decimal_encoding(significand, scale, raw=True, initial=True)
    if directed and initial < 0x0010000000000000:
        result = initial
    else:
        assembly = lambda s, k: decimal_encoding(s, k, raw=preserve_overflow)
        if dropped <= 0:
            result = assembly(significand, scale)
        else:
            divisor = 10 ** min(dropped, 32)
            quotient, remainder = divmod(significand, divisor)
            if function == 'ROUND':
                result = assembly(quotient + (2 * remainder >= divisor), scale + dropped)
            elif function in ['TRUNC', 'ROUNDDOWN']:
                result = assembly(quotient, scale + dropped)
            else:
                down = decode(f'0x{assembly(quotient, scale + dropped):016x}')
                unit = decode(f'0x{decimal_encoding(1, -original):016x}') if remainder else 0.0
                if difference_policy != 'decimal':
                    start = abs(number) if difference_policy == 'source' else decode(f'0x{decimal_encoding(significand, scale):016x}')
                    difference = start - down
                    if difference_policy == 'top16' and number < 0:
                        difference = -start + down
                    suppressed = int(encode(difference), 16) >> 48 == 0 if difference_policy == 'top16' else abs(difference) < 2**-1022
                    if suppressed:
                        unit = 0.0
                result = int(encode(down + unit), 16)
    if (result >> 52) == 2047 and (function == 'ROUNDUP' or not preserve_overflow):
        return 'error:Num'
    if result and number < 0:
        result |= 1 << 63
    return f'0x{result:016x}'


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    repeated_path = RUN / 'repeated-session-controls.json'
    repeated = json.loads(repeated_path.read_text(encoding='utf-8-sig'))
    groups = collections.defaultdict(list)
    for row in repeated['records']:
        key = tuple(row['source'][name] for name in ['function', 'bits', 'count'])
        groups[key].append(row)
    differing = []
    fields = ['before_dependents', 'after_dependents', 'scalar_value2',
              'after_all_recalculations', 'isnumber', 'iserror', 'type', 'add_zero']
    for key, rows in groups.items():
        if len({json.dumps({f: r[f] for f in fields}, sort_keys=True) for r in rows}) != 1:
            differing.append({'key': key, 'rows': rows})
    report = {
        'status': 'in_progress',
        'scope_completeness': 'scope_partial', 'target_completeness': 'target_partial',
        'integration_completeness': 'partial',
        'open_lanes': ['historical_TRUNC_MAX_capture_conflict', 'overflow_numeric_publication',
                       'initial15_precision', 'post_refinement_independent_validation'],
        'repeated_controls': {
            'rows': len(repeated['records']), 'independent_processes': repeated['sessions'],
            'distinct_inputs': len(groups), 'repetitions_per_input': sorted({len(r) for r in groups.values()}),
            'ingress_failures': [r for r in repeated['records'] if not r['ingress_exact']],
            'inconsistent_groups': differing,
            'tested_axes': ['General/scientific', 'Calculate/CalculateFull',
                            'Formula2/Formula2R1C1', 'before/after dependent formulas'],
            'conclusion': 'No tested session, format, API, or calculation setting explains the historical disagreement.'
        },
        'models': {},
        'sources': [],
    }
    for function in ['TRUNC', 'ROUNDDOWN', 'ROUNDUP', 'ROUND']:
        path = RUN / f'answers-{function.lower()}.json'
        evidence = json.loads(path.read_text(encoding='utf-8-sig'))
        models = {}
        for raw in [False, True]:
            misses = []
            for row in evidence['witnesses']:
                actual = predict(function, row['args'], preserve_overflow=raw)
                if actual != row['expected_bits']:
                    misses.append(dict(row, actual=actual))
            models['raw_exponent_research' if raw else 'finite_subnormal_repair'] = {
                'rows': len(evidence['witnesses']), 'mismatches': len(misses), 'misses': misses,
            }
        report['models'][function] = models
    for source in [repeated_path, Path('.tmp/w111-rounding-repeated-session-controls.log')]:
        destination = OUT / ('endpoint-' + source.name)
        shutil.copyfile(source, destination)
        report['sources'].append({'source': source.as_posix(), 'retained': destination.as_posix(),
                                  'sha256': hashlib.sha256(source.read_bytes()).hexdigest()})
    (OUT / 'endpoint-refinement-analysis.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print('Repeated controls', report['repeated_controls']['rows'], 'inconsistencies', len(differing))
    for function, models in report['models'].items():
        print(function, {key: model['mismatches'] for key, model in models.items()})


if __name__ == '__main__':
    main()
