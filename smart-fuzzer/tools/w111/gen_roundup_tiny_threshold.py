"""Signed residual exponent controls, independent of a guessed cutoff."""
import json
import math
import struct
from pathlib import Path


def bits(number):
    return '0x' + struct.pack('>d', number).hex()


def main():
    run = Path('smart-fuzzer/runs/w111-roundup-tiny-threshold-20260929')
    (run / 'batches').mkdir(parents=True, exist_ok=True)
    rows = set()
    bases = [(1e-307, [307]), (1e-305, [305, 307]), (1e-300, [300, 307]),
             (1e-298, [298, 305, 307]), (7.875e-300, [307]), (7.875e-298, [305, 307])]
    for base, counts in bases:
        deltas = [math.ldexp(1.0, exponent) for exponent in range(-1074, -1015)]
        deltas += [coefficient * 10.0**exponent for exponent in range(-321, -306)
                   for coefficient in [1., 2.2250738585072, 5., 9.]]
        for delta in deltas:
            center = base + delta
            for number in [math.nextafter(center, 0.), center, math.nextafter(center, math.inf)]:
                for count in counts:
                    for sign in [-1., 1.]:
                        rows.add((bits(sign * number), bits(float(count))))
    ordered = sorted(rows)
    for function in ['ROUNDUP', 'ROUNDDOWN']:
        packet = {'function': function, 'probes': [{'probe': {
            'id': f'w111roundtinythreshold-{function}-{index:05d}', 'args': list(args)}}
            for index, args in enumerate(ordered)]}
        (run / 'batches' / f'batch-{function.lower()}.json').write_text(json.dumps(packet, separators=(',', ':')), encoding='utf-8')
    (run / 'design.json').write_text(json.dumps({'rows_per_function': len(ordered),
        'phase': 'discovery_after_rejected_MIN_NORMAL_subtraction_hypothesis',
        'purpose': 'Map sign and residual exponent effects over binary and decimal ladders, with multiple base magnitudes and independent requested decimal units. No input-specific lookup or numerical production change.'}, indent=2), encoding='utf-8')
    print(len(ordered), 'per function')


if __name__ == '__main__':
    main()
