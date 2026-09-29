"""Research model for complex coefficient formatting; not semantic authority.

The rule is frozen for the discriminator capture: 15 significant decimal
digits, no near-integer snapping, fixed notation up to 21 unsigned characters,
otherwise scientific notation with at least two exponent digits. Decimal
rounding crossing the normal binary64 limits is predicted as signed text zero
or #NUM. Oracle capture remains independent of this model.
"""
import json
import math
from pathlib import Path
import struct
import sys


def number(arg):
    return struct.unpack('>d', bytes.fromhex(arg[2:]))[0]


def coefficient(value):
    if value == 0:
        return '0'
    sign = '-' if value < 0 else ''
    mantissa, exponent = format(abs(value), '.14e').split('e')
    rounded = float(mantissa + 'e' + exponent)
    if not math.isfinite(rounded):
        raise OverflowError()
    if 0 < rounded < float.fromhex('0x1p-1022'):
        return sign + '0'
    exponent = int(exponent)
    digits = mantissa.replace('.', '')
    position = exponent + 1
    if position <= 0:
        fixed = '0.' + '0' * -position + digits
    elif position >= len(digits):
        fixed = digits + '0' * (position - len(digits))
    else:
        fixed = digits[:position] + '.' + digits[position:]
    if '.' in fixed:
        fixed = fixed.rstrip('0').rstrip('.')
    if len(fixed) > 21:
        fixed = mantissa.rstrip('0').rstrip('.') + f'E{exponent:+03d}'
    return sign + fixed


def complex_text(real, imaginary):
    if imaginary == 0:
        return coefficient(real)
    imaginary_text = '' if abs(imaginary) == 1 else coefficient(abs(imaginary))
    if real == 0:
        return ('-' if imaginary < 0 else '') + imaginary_text + 'i'
    return coefficient(real) + ('-' if imaginary < 0 else '+') + imaginary_text + 'i'


def predict(args):
    try:
        return 'text:' + complex_text(*map(number, args))
    except OverflowError:
        return 'error:Num'


if __name__ == '__main__':
    batch = json.loads(Path(sys.argv[1]).read_text(encoding='utf-8-sig'))
    assert batch['function'] == 'COMPLEX'
    witnesses = [{'id': row['probe']['id'], 'args': row['probe']['args'],
                  'expected_bits': predict(row['probe']['args'])} for row in batch['probes']]
    Path(sys.argv[2]).write_text(json.dumps({'function': 'COMPLEX', 'witnesses': witnesses,
        'authority': 'research_prediction_not_oracle_evidence'}, indent=2), encoding='utf-8')
    print(f'{len(witnesses)} predictions -> {sys.argv[2]}')
