"""Distinguish decimal remainder from binary subtraction in ROUNDUP's increment."""
import json
import math
import struct
from pathlib import Path


def bits(number):
    return '0x' + struct.pack('>d', number).hex()


def main():
    run = Path('smart-fuzzer/runs/w111-roundup-tiny-difference-20260929')
    (run / 'batches').mkdir(parents=True, exist_ok=True)
    rows = set()
    def add(number, count):
        if math.isfinite(number) and abs(number) >= 2**-1022:
            rows.add((bits(number), bits(float(count))))
    # Small normal values with a residual both below and above minimum normal.
    # Compare source neighbors and requested counts that share the same down
    # value but have distinct normal/subnormal increment units.
    for exponent in [-307, -305, -300, -298, -295, -294, -293, -290]:
        for base in [1., 2.2250738585072, 7.875]:
            center = base * 10.0**exponent
            for delta in [0., 2**-1022/4, 2**-1022, 2**-1021, 10.0**exponent * 5e-14]:
                number = center + delta
                for neighbor in [math.nextafter(number, 0.), number, math.nextafter(number, math.inf)]:
                    for count in [-exponent, -exponent+7, -exponent+13, 307, 308, 323]:
                        for sign in [-1., 1.]:
                            add(sign * neighbor, count)
    # Large-unit controls isolate the increment decision from the magnitude of
    # the decimal unit itself; ordinary medium values test a global tolerance.
    for number in [1.0000000000000502e-298, 1.00000000000005e-290, 1.00000000000005, 1.00000000000005e100]:
        for count in [-308, -100, -1, 0, 1, 7, 14, 290, 298, 305, 307, 308, 309, 323, 65536]:
            for sign in [-1., 1.]:
                add(sign * number, count)
    ordered = sorted(rows)
    for function in ['ROUNDUP', 'ROUNDDOWN']:
        packet = {'function': function, 'probes': [{'probe': {
            'id': f'w111roundtinydiff-{function}-{i:05d}', 'args': list(args)}}
            for i, args in enumerate(ordered)]}
        (run / 'batches' / f'batch-{function.lower()}.json').write_text(json.dumps(packet, separators=(',', ':')), encoding='utf-8')
    (run / 'design.json').write_text(json.dumps({'phase': 'discovery_from_one_frozen_heldout_failure',
        'rows_per_function': len(ordered), 'source': 'w111roundheldout-ROUNDUP-04118',
        'hypotheses': ['decimal remainder alone', 'initial binary value minus down with subnormal subtraction flushed', 'original source minus down with subnormal subtraction flushed'],
        'no_runtime_change_before_capture': True}, indent=2), encoding='utf-8')
    print(len(ordered), 'per function')


if __name__ == '__main__':
    main()
