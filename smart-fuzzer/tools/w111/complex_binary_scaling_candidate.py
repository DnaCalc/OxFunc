"""Second COMPLEX research candidate: exact-rational simulation, not an oracle.

For k = 14 - floor(log10(abs(x))), round the positive integer power of ten to
64 binary significand bits. Multiply for k >= 0, divide for k < 0; round that
operation to 64 binary significand bits, then round the scaled magnitude to an
integer with ties away. RN64 uses nearest-even. This is an arithmetic model
derived from black-box observations, not a claim about Excel internals.
"""
from fractions import Fraction as F
import json
import math
from pathlib import Path
import sys

from gen_complex_midpoint_search_20260929 import (
    extended_power, half_away, number, round_binary,
)


def decimal_pair(value):
    k = 14 - int(format(abs(value), '.30e').split('e')[1])
    operand = F(abs(value))
    power = extended_power(abs(k))
    scaled = operand * power if k >= 0 else operand / power
    return half_away(round_binary(scaled, 64)), -k


def coefficient(value):
    if value == 0:
        return '0'
    if not math.isfinite(value) or abs(value) == sys.float_info.max:
        raise OverflowError()
    sign = '-' if value < 0 else ''
    significand, scale = decimal_pair(value)
    digits = str(significand)
    if float(f'{significand}e{scale}') < sys.float_info.min:
        return sign + '0'
    point = len(digits) + scale
    if point <= 0:
        fixed = '0.' + '0' * -point + digits
    elif point >= len(digits):
        fixed = digits + '0' * (point - len(digits))
    else:
        fixed = digits[:point] + '.' + digits[point:]
    if '.' in fixed:
        fixed = fixed.rstrip('0').rstrip('.')
    if len(fixed) > 21:
        fixed = (digits[:1] + '.' + digits[1:]).rstrip('0').rstrip('.') + f'E{point-1:+03d}'
    return sign + fixed


def predict(function, args):
    values = [number(int(arg, 16)) for arg in args]
    real, imaginary = values if function == 'COMPLEX' else [values[0], 0.0]
    try:
        if imaginary == 0:
            text = coefficient(real)
        else:
            imag_text = '' if abs(imaginary) == 1 else coefficient(abs(imaginary))
            text = ('' if real == 0 else coefficient(real))
            text += ('-' if imaginary < 0 else ('' if real == 0 else '+')) + imag_text + 'i'
        return 'text:' + text
    except OverflowError:
        return 'error:Num'


if __name__ == '__main__':
    total = misses = 0
    for path in map(Path, sys.argv[1:]):
        capture = json.loads(path.read_text(encoding='utf-8-sig'))
        failures = []
        for row in capture['witnesses']:
            actual = predict(capture['function'], row['args'])
            if actual != row['expected_bits']:
                failures.append(dict(id=row['id'], args=row['args'], actual=actual,
                                     expected=row['expected_bits']))
        total += len(capture['witnesses'])
        misses += len(failures)
        print(path, len(capture['witnesses'])-len(failures), '/', len(capture['witnesses']))
        for failure in failures[:5]:
            print(failure)
    print(f'total {total-misses}/{total}')
