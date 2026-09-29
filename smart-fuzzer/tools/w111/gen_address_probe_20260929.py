"""ADDRESS coordinate/absolute-relative boundary probes, numeric Value2 inputs.

Usage: python gen_address_probe_20260929.py OUT [--heldout --seed N]
Discovery emphasizes discriminating integer-boundary/rounding hypotheses. Heldout
is generated only after the production candidate freezes, with separate random
coordinates and selector inputs. The caller records the candidate source hash.
"""
import argparse
import json
import math
import random
import struct
from pathlib import Path


def bits(x):
    return '0x%016x' % struct.unpack('<Q', struct.pack('<d', float(x)))[0]


def make(heldout, seed):
    rows = []
    if not heldout:
        for axis, maximum in [(0, 1048576), (1, 16384)]:
            values = [-2**31, -maximum-1, -maximum, -maximum+1, -maximum+2,
                      -2, -1, -.9, -0.0, 0, .9, 1, 2,
                      maximum-2, maximum-1, maximum, maximum+1, 2**31]
            for boundary in [-maximum, -maximum+1, 0, 1, maximum-1, maximum]:
                values += [boundary-.25, boundary+.25,
                           math.nextafter(float(boundary), -math.inf),
                           math.nextafter(float(boundary), math.inf)]
            for x in values:
                for mode in [1, 2, 3, 4]:
                    for style in [0, 1]:
                        xy = [1, 1]
                        xy[axis] = x
                        rows.append((*xy, mode, style))
        for x, y in [(0, 0), (-1, -1), (-1048575, -16383),
                     (-1048576, -16384), (1048575, 16383), (1048576, 16384)]:
            for mode in [0, .9, 1, 1.9, 2, 2.9, 3, 3.9, 4, 4.9, 5]:
                for style in [-1, 0, .1, 1]:
                    rows.append((x, y, mode, style))
    else:
        rng = random.Random(seed)
        for _ in range(4096):
            row = rng.uniform(-1049000, 1049000)
            col = rng.uniform(-17000, 17000)
            if rng.randrange(3) == 0:
                row = rng.choice([-1048576, -1048575, -1, 0, 1, 1048575, 1048576]) + rng.uniform(-1, 1)
            if rng.randrange(3) == 0:
                col = rng.choice([-16384, -16383, -1, 0, 1, 16383, 16384]) + rng.uniform(-1, 1)
            mode = rng.choice([1, 2, 3, 4, 5]) + rng.choice([0, 0, rng.random()])
            style = rng.choice([0, 1, -1, .5])
            rows.append((row, col, mode, style))
    seen, probes = set(), []
    for row in rows:
        args = tuple(bits(x) for x in row)
        if args in seen:
            continue
        seen.add(args)
        probes.append({'probe': {'id': f'address-{seed}-{"heldout" if heldout else "boundary"}-{len(probes):05d}', 'args': list(args)}})
    return {'function': 'ADDRESS', 'probes': probes}


def rounding(seed):
    rows = []
    for axis, integers in [(0, [1, 2, 3, 10, 100, 256, 1024, 16383, 16384, 1048575, 1048576]),
                           (1, [1, 2, 3, 10, 100, 256, 1024, 16383, 16384])]:
        for integer in integers:
            for sign in [-1, 1]:
                for direction in [-math.inf, math.inf]:
                    x = float(sign * integer)
                    for step in range(1, 513):
                        x = math.nextafter(x, direction)
                        if step <= 64 or step in [96, 128, 192, 256, 384, 512]:
                            args = [1, 1, 4, 0]
                            args[axis] = x
                            rows.append(tuple(args))
    for integer in [1, 2, 3, 4, 5]:
        for direction in [-math.inf, math.inf]:
            x = float(integer)
            for step in range(1, 65):
                x = math.nextafter(x, direction)
                rows.append((1, 1, x, 0))
    return {'function': 'ADDRESS', 'probes': [{'probe': {'id': f'address-{seed}-rounding-{i:05d}',
            'args': [bits(x) for x in row]}} for i, row in enumerate(rows)]}


def floor_threshold(seed):
    rows = []
    for axis in [0, 1, 2]:
        anchors = [0, 1, 2, 3, 10, 100, 1000, 16383] if axis != 2 else [0, 1, 2, 3, 4, 5]
        if axis == 0:
            anchors += [1048575, 1048576]
        for anchor in anchors:
            for sign in [-1, 1]:
                for exponent in range(-14, 0):
                    for factor in [.25, .5, .9, 1, 1.1, 1.5, 2, 5, 9]:
                        delta = factor * 10.0**exponent
                        for side in [-1, 1]:
                            args = [1, 1, 4, 0]
                            args[axis] = sign * anchor + side * delta
                            rows.append(tuple(args))
    seen, probes = set(), []
    for row in rows:
        argbits = tuple(bits(x) for x in row)
        if argbits in seen: continue
        seen.add(argbits)
        probes.append({'probe': {'id': f'address-{seed}-floor-threshold-{len(probes):05d}', 'args': list(argbits)}})
    return {'function': 'ADDRESS', 'probes': probes}


def floor_bits(seed):
    rows = []
    for axis, anchors in [(0, [-1048575, -16383, -1000, -10, -2, -1, 0, 1, 2, 3, 10, 1000, 16383, 1048575, 1048576]),
                          (1, [-16383, -10, -1, 0, 1, 2, 10, 16383, 16384]),
                          (2, [0, 1, 2, 3, 4, 5])]:
        for anchor in anchors:
            threshold = float(anchor) - 2.0**(-22 if anchor > 0 else -23)
            for direction in [-math.inf, math.inf]:
                x = threshold
                for step in range(33):
                    args = [1, 1, 4, 0]
                    args[axis] = x
                    rows.append(tuple(args))
                    x = math.nextafter(x, direction)
            for ratio in [.95, .99, 1.01, 1.05]:
                args = [1, 1, 4, 0]
                args[axis] = float(anchor) - ratio * 2.0**(-22 if anchor > 0 else -23)
                rows.append(tuple(args))
    seen, probes = set(), []
    for row in rows:
        argbits = tuple(bits(x) for x in row)
        if argbits in seen: continue
        seen.add(argbits)
        probes.append({'probe': {'id': f'address-{seed}-floor-bits-{len(probes):05d}', 'args': list(argbits)}})
    return {'function': 'ADDRESS', 'probes': probes}


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('out', type=Path)
    ap.add_argument('--heldout', action='store_true')
    ap.add_argument('--rounding', action='store_true')
    ap.add_argument('--floor-threshold', action='store_true')
    ap.add_argument('--floor-bits', action='store_true')
    ap.add_argument('--seed', type=int, default=20260929)
    a = ap.parse_args()
    p = floor_bits(a.seed) if a.floor_bits else floor_threshold(a.seed) if a.floor_threshold else rounding(a.seed) if a.rounding else make(a.heldout, a.seed)
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(p, separators=(',', ':')), encoding='utf-8')
    print(len(p['probes']), a.out)


if __name__ == '__main__':
    main()
