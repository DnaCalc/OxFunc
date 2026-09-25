"""Corpus for the rounding family (W111 G8-14): ROUND, ROUNDUP, ROUNDDOWN, MROUND.

Regions: money-style values with a trailing 5 at the rounding digit (the half-way cases
binary rounding gets wrong), random values across magnitudes 1e-12..1e17, digits from -8 to
16, integers, negative values, zero/negative zero, subnormals and huge values.

Usage: python smart-fuzzer/tools/w111/gen_round_batches.py <out_dir> [--seed N]
"""

import json
import random
import struct
import sys
from pathlib import Path


def bits(x: float) -> str:
    return "0x%016x" % struct.unpack("<Q", struct.pack("<d", float(x)))[0]


def batch(fn, rows, tag):
    return {"function": fn, "probes": [{"probe": {"id": f"{tag}-{fn}-{i:05d}", "args": [bits(a) for a in r]}}
                                       for i, r in enumerate(rows)]}


def digit_rows(r):
    rows = []
    for _ in range(700):  # half-way at the rounding digit
        d = r.randint(0, 6)
        whole = r.choice([0, 1, 2, 10, 99, 1234, 999999, r.randint(0, 10**r.randint(0, 9))])
        frac = r.randint(0, 10**d - 1) * 10 + 5
        v = float(f"{whole}.{frac:0{d + 1}d}") * r.choice([1, -1])
        rows.append((v, d))
    for _ in range(700):  # random magnitudes and digits
        v = r.choice([1, -1]) * 10 ** r.uniform(-12, 17)
        rows.append((v, r.randint(-8, 16)))
    for _ in range(200):  # negative digits on half-way integers
        d = r.randint(1, 6)
        v = (r.randint(0, 999) * 10 + 5) * 10 ** (d - 1) * r.choice([1, -1])
        rows.append((float(v), -d))
    for v in [0.0, -0.0, 5e-324, 1e-308, 0.5, 1.5, 2.5, -2.5, 0.49999999999999994, 1e15 + 0.5,
              4503599627370497.0, 1.7976931348623157e308, 123456789012345.67, 0.1 + 0.2]:
        for d in [0, 1, 2, 15, 16, -1, 308, -308]:
            rows.append((v, d))
    return rows


def mround_rows(r):
    rows = []
    for _ in range(700):
        m = r.choice([0.05, 0.1, 0.25, 0.5, 1, 3, 5, 10, 0.01, 0.2, 7, 100])
        k = r.randint(0, 5000)
        v = (k + 0.5) * m if r.random() < 0.6 else r.uniform(0, 5000) * m
        s = r.choice([1, -1])
        rows.append((s * v, s * m))
    for _ in range(300):
        rows.append((r.uniform(-1e6, 1e6), r.choice([1, -1]) * 10 ** r.uniform(-4, 4)))
    return rows


def main():
    out = Path(sys.argv[1])
    seed = int(sys.argv[sys.argv.index("--seed") + 1]) if "--seed" in sys.argv else 20260925
    out.mkdir(parents=True, exist_ok=True)
    for fn in ["ROUND", "ROUNDUP", "ROUNDDOWN"]:
        r = random.Random(f"{seed}-{fn}")
        (out / f"batch-round-{fn.lower()}.json").write_text(json.dumps(batch(fn, digit_rows(r), f"r{seed}")),
                                                            encoding="utf-8")
    r = random.Random(f"{seed}-MROUND")
    (out / "batch-round-mround.json").write_text(json.dumps(batch("MROUND", mround_rows(r), f"r{seed}")),
                                                 encoding="utf-8")


if __name__ == "__main__":
    main()
