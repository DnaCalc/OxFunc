"""Fresh broad numeric-input campaign; no Excel or production mutation.

Usage: python gen_w111_broad_20260929.py OUT [--seed 20260929] [--rows 1200]

Each file is a ProbeBatch for Run-W109BulkBatch.ps1. All numeric arguments are
IEEE-754 bits for Range.Value2 transport. Per-function RNGs make additions leave
existing corpora unchanged. This is discovery evidence, not a closure corpus.
An independent --seed supplies a later held-out corpus after a candidate freezes.
Scalar numeric inputs deliberately leave text, reference, array and locale lanes
open; returned values may be numbers, text, logicals or worksheet errors.
"""

import argparse
import hashlib
import json
import math
import random
import struct
import zlib
from pathlib import Path


MAX = float.fromhex("0x1.fffffffffffffp1023")
MIN = float.fromhex("0x1p-1022")
EDGE = [0.0, -0.0, 5e-324, -5e-324, MIN, -MIN, 1e-300, -1e-300,
        1e-16, -1e-16, 0.5, -0.5, 1.0, -1.0, 2.0, -2.0,
        math.nextafter(1.0, 0), math.nextafter(1.0, 2),
        1e15, -1e15, 2**52 - 1, 2**52, 2**53, 1e100, -1e100, MAX, -MAX]
DATE_EDGE = [-1, 0, 0.5, 1, 31, 58, 59, 59.99, 60, 60.5, 61, 365, 366,
             1462, 36526, 43890, 44561, 2958464, 2958465, 2958466]


def bits(x):
    return "0x%016x" % struct.unpack("<Q", struct.pack("<d", float(x)))[0]


class Corpus:
    def __init__(self, seed, n):
        self.seed, self.n, self.batches = seed, n, {}

    def rng(self, fn):
        return random.Random(self.seed + zlib.crc32(fn.encode("ascii")))

    def add(self, fn, draw, edges=()):
        r = self.rng(fn)
        rows = [tuple(x) for x in edges]
        rows.extend(tuple(draw(r, i)) for i in range(self.n))
        assert all(math.isfinite(float(x)) for row in rows for x in row), fn
        self.batches[fn] = rows


def signed_log(r, low=-300, high=300):
    return r.choice([-1, 1]) * 10.0**r.uniform(low, high)


def broad_num(r, i):
    if i % 5 == 0:
        return signed_log(r)
    if i % 5 == 1:
        return r.randint(-1000000, 1000000) + r.choice([0, .5, .1, .999999999999])
    return r.uniform(-1e8, 1e8)


def make(seed, n):
    c = Corpus(seed, n)

    # Still Unverified in the 2026-09-25 ledger. ADDRESS's text output remains
    # bounded even for invalid row/column inputs; COMPLEX uses its default i.
    c.add("ADDRESS", lambda r, i: tuple([r.randint(1, 1048576), r.randint(1, 16384)] +
          ([r.choice([1, 2, 3, 4])] if i % 3 else []) +
          ([r.choice([0, 1])] if i % 3 == 2 else [])),
          [(a, b, m, st) for a, b in [(1, 1), (0, 1), (-1, 1), (1048576, 16384),
          (1048577, 1), (1, 16385), (1.9, 2.9)] for m in [0, 1, 2, 3, 4, 5]
          for st in [0, 1]])
    c.add("COMPLEX", lambda r, i: (broad_num(r, i), broad_num(r, i + 1)),
          [(a, b) for a in EDGE for b in [0, -0.0, 1, -1, .1]])
    c.add("IFS", lambda r, i: (r.choice([0, 1, -1, 0.5]), broad_num(r, i),
          r.choice([0, 1, -1]), broad_num(r, i + 1)),
          [(a, 42, b, 17) for a in [0, -0.0, 1, -1, MIN, -MIN] for b in [0, 1]])
    c.add("SWITCH", lambda r, i: (r.choice([0, 1, -1, .1]), 0, broad_num(r, i),
          1, broad_num(r, i + 1), broad_num(r, i + 2)),
          [(x, 1, 10, -1, 20, 30) for x in EDGE])

    # Cheap scalar maps and predicates, including DAZ and large-input handling.
    for fn in ["ABS", "INT", "EVEN", "ODD", "SIGN", "SQRT", "ISNUMBER", "ISLOGICAL",
               "ISNONTEXT", "ISTEXT", "ISBLANK", "ISERR", "ISERROR", "ISNA", "ISREF",
               "ISEVEN", "ISODD", "N", "T", "TYPE", "NOT", "ERROR.TYPE"]:
        c.add(fn, lambda r, i: (broad_num(r, i),), [(x,) for x in EDGE])
    for fn in ["DEGREES", "RADIANS"]:
        c.add(fn, lambda r, i: (broad_num(r, i),), [(x,) for x in EDGE])
    for fn in ["DELTA", "GESTEP"]:
        c.add(fn, lambda r, i: (broad_num(r, i),) if i % 3 == 0 else
              (float(r.randint(-20, 20)), float(r.randint(-20, 20))),
              [(x,) for x in EDGE] + [(x, x) for x in EDGE])
    for fn in ["MOD", "QUOTIENT"]:
        c.add(fn, lambda r, i: (broad_num(r, i), r.choice([-1, 1]) *
              10.0**r.uniform(-10, 10)), [(a, b) for a in EDGE for b in [-2, -1, 0, .1, 1, 2]])
    c.add("TRUNC", lambda r, i: (broad_num(r, i),) if i % 4 == 0 else
          (broad_num(r, i), r.choice([-300, -20, -10, -2, -1, 0, 1, 2, 10, 20, 300])),
          [(x, d) for x in EDGE for d in [-308, -1, 0, 1, 15, 308]])
    for fn in ["CEILING", "FLOOR", "CEILING.PRECISE", "FLOOR.PRECISE", "ISO.CEILING"]:
        c.add(fn, lambda r, i: (broad_num(r, i), r.choice([-1, 1]) *
              10.0**r.uniform(-6, 6)), [(a, b) for a in EDGE for b in [-2, -.1, 0, .1, 2]])
    for fn in ["CEILING.MATH", "FLOOR.MATH"]:
        c.add(fn, lambda r, i: (broad_num(r, i),) if i % 4 == 0 else
              (broad_num(r, i), r.uniform(-100, 100), r.choice([-1, 0, 1, 2])),
              [(a, b, m) for a in EDGE for b in [-1, 0, 1] for m in [0, 1]])
    # No FACT/FACTDOUBLE/PERMUT huge-edge loops: bounded domains plus admission.
    c.add("FACT", lambda r, i: (r.uniform(-2, 173),), [(x,) for x in [-1, -.1, 0, 1, 2, 169, 170, 171, 172]])
    c.add("FACTDOUBLE", lambda r, i: (r.uniform(-3, 302),), [(x,) for x in [-3, -2, -1, -.1, 0, 1, 298, 299, 300, 301]])
    for fn in ["PERMUT", "PERMUTATIONA"]:
        c.add(fn, lambda r, i: (r.uniform(-2, 172), r.uniform(-2, 35)),
              [(a, b) for a in [-1, 0, 1, 5, 170, 171] for b in [-1, 0, 1, 2, 20]])

    # Exactly represented integer and truncation boundaries around 48-bit words.
    for fn in ["BITAND", "BITOR", "BITXOR"]:
        c.add(fn, lambda r, i: (r.randrange(2**48) + r.choice([0, 0, .5]),
              r.randrange(2**48) + r.choice([0, 0, .25])),
              [(a, b) for a in [-1, 0, .9, 1, 2**47, 2**48 - 1, 2**48]
               for b in [-1, 0, 1, 3, 2**48 - 1]])
    for fn in ["BITLSHIFT", "BITRSHIFT"]:
        c.add(fn, lambda r, i: (r.randrange(2**48), r.randint(-54, 54) + r.choice([0, .5])),
              [(a, b) for a in [-1, 0, .9, 1, 2**47, 2**48 - 1, 2**48]
               for b in [-54, -53, -48, -1, 0, 1, 47, 48, 53, 54]])
    for fn, lo, hi in [("DEC2BIN", -513, 513), ("DEC2OCT", -(2**29) - 1, 2**29 + 1),
                       ("DEC2HEX", -(2**39) - 1, 2**39 + 1)]:
        c.add(fn, lambda r, i, lo=lo, hi=hi: (r.uniform(lo, hi),) if i % 3 == 0 else
              (r.uniform(lo, hi), r.choice([-1, 0, 1, 5, 10, 11, 5.5])),
              [(x, p) for x in [lo, lo + 1, -1, 0, 1, hi - 1, hi] for p in [1, 10, 11]])
    for fn, radix in [("BIN2DEC", 2), ("BIN2HEX", 2), ("BIN2OCT", 2),
                       ("OCT2DEC", 8), ("OCT2BIN", 8), ("OCT2HEX", 8),
                       ("HEX2DEC", 10), ("HEX2BIN", 10), ("HEX2OCT", 10)]:
        c.add(fn, lambda r, i, radix=radix, fn=fn: tuple([float("".join(str(r.randrange(radix))
              for _ in range(r.randint(1, 10))))] + ([r.randint(1, 10)] if
              not fn.endswith("DEC") and i % 3 else [])),
              [(x,) for x in [-1, 0, 1, 9, 10, 1111111111, 9999999999, 1e10]])
    c.add("BASE", lambda r, i: (r.randrange(2**53), r.randint(2, 36), r.randint(0, 20)),
          [(a, b, p) for a in [-1, 0, 1, 2**53 - 1, 2**53] for b in [1, 2, 10, 36, 37] for p in [0, 4]])
    c.add("DECIMAL", lambda r, i: (r.randint(0, 999999999), r.choice([2, 8, 10, 16, 36])),
          [(x, b) for x in [-1, 0, 1, 123456789, 1e15] for b in [1, 2, 10, 16, 36, 37]])

    # Date serial / enum boundaries use small outputs and no holiday loops.
    for fn in ["DAY", "MONTH", "YEAR", "HOUR", "MINUTE", "SECOND", "ISOWEEKNUM"]:
        c.add(fn, lambda r, i: (r.uniform(0, 2958466),), [(x,) for x in DATE_EDGE])
    for fn in ["WEEKDAY", "WEEKNUM"]:
        c.add(fn, lambda r, i: (r.uniform(0, 2958466), r.choice([1, 2, 3, 11, 12, 13, 14, 15, 16, 17, 21])),
              [(d, t) for d in DATE_EDGE for t in [0, 1, 2, 3, 11, 17, 21, 22]])
    c.add("DATE", lambda r, i: (r.randint(0, 9999), r.randint(-24, 36), r.randint(-60, 90)),
          [(y, m, d) for y in [-1, 0, 99, 100, 1900, 1904, 2000, 9999, 10000]
           for m, d in [(1, 1), (2, 28), (2, 29), (2, 30), (12, 31), (13, 1)]])
    c.add("TIME", lambda r, i: (r.uniform(-2, 32769), r.uniform(-2, 32769), r.uniform(-2, 32769)),
          [(h, m, s) for h in [-1, 0, 23, 24, 32767, 32768] for m in [-1, 0, 59, 60] for s in [-1, 0, 59, 60]])
    for fn in ["EDATE", "EOMONTH"]:
        c.add(fn, lambda r, i: (r.uniform(1, 2958000), r.uniform(-1200, 1200)),
              [(d, m) for d in DATE_EDGE for m in [-13, -1, 0, 1, 12, 120]])
    c.add("DAYS", lambda r, i: (r.uniform(0, 2958466), r.uniform(0, 2958466)),
          [(a, b) for a in DATE_EDGE for b in [0, 1, 59, 60, 61, 2958465]])

    # Direct numeric varargs: short rows expose reduction/rounding structures.
    # Numerical cancellation and mixed magnitudes are deliberate; all arities
    # are small. No array/reference semantics is inferred from this campaign.
    def varargs(r, i):
        k = r.choice([1, 2, 3, 4, 5, 8, 16])
        scale = 10.0**r.uniform(-8, 8)
        if i % 4 == 0:
            return tuple(r.randint(-1000, 1000) for _ in range(k))
        if i % 4 == 1:
            return tuple(r.uniform(-10, 10) * scale for _ in range(k))
        if i % 4 == 2:
            return tuple(r.uniform(-10, 10) + scale for _ in range(k))
        return tuple(10.0**r.uniform(-10, 10) for _ in range(k))
    reduction_edges = [(0,), (-0.0,), (1,), (1, 2), (1, 1, 1), (1, 2, 3, 4),
                       (1e16, 1, -1e16), (1e16, -1e16, 1), (MIN, MIN, MIN),
                       (0, -0.0, 0), (1e150, -1e150, 1), (.1, .2, .3, .4)]
    for fn in ["SUM", "PRODUCT", "SUMSQ", "AVERAGE", "AVERAGEA", "COUNT", "COUNTA",
               "MIN", "MINA", "MAX", "MAXA", "MEDIAN", "GEOMEAN", "HARMEAN",
               "AVEDEV", "DEVSQ", "VAR", "VAR.P", "VAR.S", "VARA", "VARPA", "VARP",
               "STDEV", "STDEV.P", "STDEV.S", "STDEVA", "STDEVPA", "STDEVP",
               "AND", "OR", "XOR"]:
        c.add(fn, varargs, reduction_edges)
    for fn in ["GCD", "LCM"]:
        c.add(fn, lambda r, i: tuple(r.randint(-1, 10000) + r.choice([0, .5])
              for _ in range(r.choice([1, 2, 3, 5]))),
              [(0,), (0, 0), (1, 0), (-1, 2), (1.9, 2.9), (12, 18, 36), (2**53, 1)])
    c.add("STANDARDIZE", lambda r, i: (broad_num(r, i), r.uniform(-1e6, 1e6),
          10.0**r.uniform(-12, 12)), [(a, 0, s) for a in EDGE for s in [-1, 0, 1, 2]])
    # Two inexpensive finance kernels that are under-represented in old data.
    c.add("SLN", lambda r, i: (r.uniform(-1e6, 1e6), r.uniform(-1e6, 1e6), r.uniform(-100, 100)),
          [(100, 0, 0), (100, 0, -1), (0, 0, 1), (1, 1, 1), (1e300, -1e300, 1)])
    c.add("SYD", lambda r, i: (r.uniform(0, 1e6), r.uniform(0, 1e6), r.uniform(-1, 100), r.uniform(-1, 100)),
          [(100, 0, 10, p) for p in [-1, 0, .5, 1, 10, 11]] + [(100, 0, 0, 1)])
    return c


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("out", type=Path)
    ap.add_argument("--seed", type=int, default=20260929)
    ap.add_argument("--rows", type=int, default=1200)
    ap.add_argument("--only", help="Comma-separated function names for a selected wave")
    args = ap.parse_args()
    corpus = make(args.seed, args.rows)
    selected = set(args.only.upper().split(",")) if args.only else None
    if selected and (selected - corpus.batches.keys()):
        ap.error("Unknown functions: " + ",".join(sorted(selected - corpus.batches.keys())))
    args.out.mkdir(parents=True, exist_ok=True)
    manifest = {"generator": Path(__file__).name, "seed": args.seed, "random_rows_per_function": args.rows,
                "scope": "numeric_scalar_arguments_only", "comparison_policy": "exact_typed_bit_match_no_tolerance",
                "phase": "discovery", "batches": []}
    for fn, rows in corpus.batches.items():
        if selected and fn not in selected:
            continue
        path = args.out / ("batch-w111broad-" + fn.lower() + ".json")
        payload = {"function": fn, "probes": [{"probe": {"id": f"w111broad-{args.seed}-{fn}-{i:05d}",
                   "args": [bits(v) for v in row]}} for i, row in enumerate(rows)]}
        data = json.dumps(payload, separators=(",", ":")).encode()
        path.write_bytes(data)
        manifest["batches"].append({"function": fn, "rows": len(rows), "path": path.name,
                                     "sha256": hashlib.sha256(data).hexdigest()})
        print(fn, len(rows))
    (args.out / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    print("TOTAL", len(manifest["batches"]), sum(x["rows"] for x in manifest["batches"]))


if __name__ == "__main__":
    main()
