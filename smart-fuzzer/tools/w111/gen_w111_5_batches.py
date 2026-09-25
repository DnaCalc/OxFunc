"""W111-5 corpus generator: fresh, deterministic probe batches for the Unverified functions
with scalar numeric arguments, in the ProbeBatch shape Run-W109BulkBatch.ps1 answers.

Each batch covers the argument regions the kernel branches on plus edge rows (zero, negative
zero, subnormal, huge, negative, integer and fractional date serials, month-ends and the
31st, every basis/frequency/method, invalid enum values). Arguments are f64 bit patterns;
the bulk engine writes them through Range.Value2, never as formula text.

Usage: python smart-fuzzer/tools/w111/gen_w111_5_batches.py <out_dir> [--seed N]
"""

import json
import zlib
import random
import struct
import sys
from datetime import date, timedelta
from pathlib import Path

EPOCH = date(1899, 12, 30)


def bits(x: float) -> str:
    return "0x%016x" % struct.unpack("<Q", struct.pack("<d", float(x)))[0]


def serial(d: date) -> int:
    return (d - EPOCH).days


EDGE_NUMS = [0.0, -0.0, 5e-324, -5e-324, 2.2250738585072014e-308, 1e-300, 1e-15, 0.5, 1.0, -1.0,
             2.0, 1e15, 1e300, 1.7976931348623157e308, -1.7976931348623157e308, 0.1, 1 / 3]


class Gen:
    def __init__(self, seed):
        self.r = random.Random(seed)

    def date(self, lo=1990, hi=2060):
        r = self.r
        pick = r.random()
        y = r.randint(lo, hi)
        m = r.randint(1, 12)
        if pick < 0.35:  # month-end, incl. Feb and the 31st
            nxt = date(y + (m == 12), m % 12 + 1, 1)
            return nxt - timedelta(days=1)
        if pick < 0.45:
            return date(y, m, min(30, 28 + r.randint(0, 2)) if m != 2 else 28)
        return date(y, m, r.randint(1, 28))

    def serial(self, lo=1990, hi=2060):
        s = serial(self.date(lo, hi))
        if self.r.random() < 0.08:  # fractional serials are truncated by Excel
            return s + self.r.choice([0.25, 0.5, 0.9999])
        return s

    def num(self, lo, hi, log=False):
        r = self.r
        if log:
            import math
            return math.exp(r.uniform(math.log(lo), math.log(hi)))
        return r.uniform(lo, hi)


def batch(fn, rows):
    return {"function": fn, "probes": [{"probe": {"id": f"w1115-{fn}-{i:05d}", "args": [bits(a) for a in args]}}
                                       for i, args in enumerate(rows)]}


def coup_rows(g, n=1500):
    rows = []
    for _ in range(n):
        s = g.serial(1995, 2045)
        m = s + g.r.choice([1, 30, 92, 181, 365, 366, 1000, 3650, 10957]) + g.r.randint(0, 400)
        if g.r.random() < 0.3:  # maturity on a month-end
            dm = EPOCH + timedelta(days=int(m))
            m = serial(date(dm.year + (dm.month == 12), dm.month % 12 + 1, 1) - timedelta(days=1))
        f = g.r.choice([1, 2, 4])
        b = g.r.choice([0, 1, 2, 3, 4])
        rows.append((s, m, f, b))
    # invalid and boundary rows
    s0 = serial(date(2020, 7, 31))
    for f, b in [(3, 0), (2, 5), (2, -1), (0, 0), (2, 4.9), (4.5, 1)]:
        rows.append((s0, s0 + 800, f, b))
    rows += [(s0, s0, 2, 0), (s0 + 10, s0, 2, 0), (0, 400, 2, 0), (-1, 400, 2, 0), (s0, 2958465, 2, 1),
             (s0, 2958466, 2, 1)]
    return rows


def main():
    out = Path(sys.argv[1])
    seed = int(sys.argv[sys.argv.index("--seed") + 1]) if "--seed" in sys.argv else 20260924
    out.mkdir(parents=True, exist_ok=True)
    g = Gen(seed)
    batches = {}

    for fn in ["COUPDAYBS", "COUPDAYS", "COUPDAYSNC", "COUPNCD", "COUPNUM", "COUPPCD"]:
        batches[fn] = coup_rows(Gen(seed + zlib.crc32(fn.encode()) % 1000))

    rows = []
    for _ in range(1200):
        i = g.serial(1990, 2040)
        s = i + g.r.randint(1, 4000)
        rows.append((i, s, g.num(0.0001, 0.25), g.r.choice([100, 1000, 1e6, 0.01]), g.r.choice([0, 1, 2, 3, 4])))
    rows += [(40000, 40000, 0.05, 1000, 0), (40001, 40000, 0.05, 1000, 0), (40000, 40100, 0, 1000, 0),
             (40000, 40100, -0.01, 1000, 0), (40000, 40100, 0.05, 0, 0), (40000, 40100, 0.05, 1000, 5)]
    batches["ACCRINTM"] = rows

    rows = []
    for _ in range(1200):
        iss = g.serial(1990, 2035)
        s = iss + g.r.randint(1, 2000)
        m = s + g.r.randint(1, 6000)
        rows.append((s, m, iss, g.num(0, 0.2), g.num(0.0001, 0.3), g.r.choice([0, 1, 2, 3, 4])))
    rows += [(40100, 40500, 40000, 0.05, 0, 0), (40100, 40500, 40000, -0.01, 0.05, 0),
             (40100, 40500, 40200, 0.05, 0.05, 0), (40100, 40100, 40000, 0.05, 0.05, 0)]
    batches["PRICEMAT"] = rows

    for fn in ["TBILLEQ", "TBILLPRICE"]:
        rows = []
        for _ in range(1200):
            s = g.serial(1990, 2050)
            rows.append((s, s + g.r.choice([1, 2, 7, 30, 91, 182, 183, 200, 364, 365, 366, 367]) + g.r.randint(0, 3),
                         g.num(0.0001, 0.5) if g.r.random() < 0.9 else g.r.choice([0, -0.01, 1, 1.5, 2])))
        batches[fn] = rows

    # TBILLYIELD shares the T-bill one-year limit. Added after the first capture, so it
    # draws from its own generator and leaves every earlier batch byte-identical.
    ty, rows = Gen(seed + zlib.crc32(b"TBILLYIELD") % 1000), []
    for _ in range(1200):
        st = ty.serial(1990, 2050)
        rows.append((st, st + ty.r.choice([1, 30, 91, 182, 183, 364, 365, 366, 367]) + ty.r.randint(0, 2),
                     ty.num(1, 99.99) if ty.r.random() < 0.95 else ty.r.choice([0, -1, 100, 150])))
    batches["TBILLYIELD"] = rows

    rows = []
    for _ in range(1200):
        rows.append((g.num(-500, 500), g.r.choice([1, 2, 4, 8, 16, 32, 3, 10, 12, 100, 0.5, 1.9, 0, -2])))
    rows += [(1.02, 16), (1.1, 32), (0, 16), (-0.0, 16), (1e15, 16), (1.5, 1e10)]
    batches["DOLLARDE"] = rows
    batches["DOLLARFR"] = list(rows)  # same kernel shape, mirror function

    rows = []
    for _ in range(1500):
        rows.append((g.serial(1900, 2100), g.serial(1900, 2100), g.r.choice([0, 1])))
    rows += [(serial(date(2020, 2, 29)), serial(date(2021, 2, 28)), 0), (60, 61, 0), (59, 61, 1)]
    batches["DAYS360"] = rows

    rows = [(x,) for x in EDGE_NUMS]
    for _ in range(1500):
        rows.append((g.r.choice([-1, 1]) * g.num(1e-300, 1e300, log=True),))
    batches["ASINH"] = rows
    rows = [(x,) for x in [1.0, 1.0000000000000002, 1.5, 2.0, 0.9999, 0.0, -1.0, 1e15, 1e300, 1.7976931348623157e308]]
    for _ in range(1500):
        rows.append((1 + g.num(1e-16, 1e300, log=True),))
    batches["ACOSH"] = rows

    for fn in ["NORMDIST", "LOGNORM.DIST"]:
        rows = []
        for _ in range(1500):
            x = g.num(-40, 40) if fn == "NORMDIST" else g.num(1e-6, 1e6, log=True)
            rows.append((x, g.num(-5, 5), g.num(0.01, 10, log=True), g.r.choice([0, 1])))
        rows += [(0, 0, 1, 1), (0, 0, 0, 1), (0, 0, -1, 0), (1, 0, 1, 0), (-0.0, 0, 1, 1)]
        batches[fn] = rows

    rows = []
    for _ in range(1500):
        a = g.serial(1900, 2100)
        rows.append((a, a + g.r.randint(-4000, 4000), g.r.choice([0, 1, 2, 3, 4])))
    rows += [(40000, 40000, 0), (40000, 40365, 5), (40000, 40365, -1)]
    batches["YEARFRAC"] = rows

    rows = []
    for _ in range(1200):
        a = g.serial(1990, 2060)
        rows.append((a, a + g.r.randint(-800, 800)))
    batches["NETWORKDAYS"] = rows
    rows = []
    for _ in range(1200):
        a = g.serial(1990, 2060)
        rows.append((a, a + g.r.randint(-800, 800), g.r.choice([1, 2, 3, 4, 5, 6, 7, 11, 12, 13, 14, 15, 16, 17, 8, 0])))
    batches["NETWORKDAYS.INTL"] = rows
    rows = []
    for _ in range(1200):
        rows.append((g.serial(1990, 2060), g.r.randint(-600, 600) + g.r.choice([0, 0, 0.5])))
    batches["WORKDAY"] = rows
    rows = []
    for _ in range(1200):
        rows.append((g.serial(1990, 2060), g.r.randint(-600, 600), g.r.choice([1, 2, 3, 4, 5, 6, 7, 11, 12, 13, 14, 15, 16, 17, 9])))
    batches["WORKDAY.INTL"] = rows

    for fn in ["KURT", "SKEW"]:
        rows = []
        for _ in range(1500):
            k = g.r.choice([3, 4, 4, 5, 6, 8, 10])
            scale = g.num(1e-6, 1e6, log=True)
            rows.append(tuple(g.r.gauss(0, 1) * scale + g.num(-10, 10) for _ in range(k)))
        rows += [(1, 1, 1, 1), (1, 2, 3), (1, 2), (0, 0, 0, 1)]
        batches[fn] = rows

    rows = [(x,) for x in [0, 1, 21, 100, 1234.5, 0.25, -5, 1e15, 99999999.99]]
    for _ in range(600):
        rows.append((round(g.num(-1e9, 1e9), g.r.choice([0, 2, 5])),))
    batches["BAHTTEXT"] = rows

    for fn, rows in batches.items():
        (out / f"batch-w1115-{fn.lower()}.json").write_text(json.dumps(batch(fn, rows)), encoding="utf-8")
        print(fn, len(rows))


if __name__ == "__main__":
    main()
