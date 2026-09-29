"""Oracle-blind boundary and held-out batches for the five bitwise functions.

Usage: python gen_bitwise_20260929.py OUT --phase discriminator|heldout [--seed N]
Every numeric input is exact bits for Run-W109BulkBatch Value2 capture. A heldout
phase excludes the discovery corpus and any discriminator batches in OUT's parent.
"""
import argparse
import hashlib
import json
import math
import random
import struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
FUNCTIONS = ["BITAND", "BITOR", "BITXOR", "BITLSHIFT", "BITRSHIFT"]
MAX = 2**48 - 1
def bits(x): return "0x%016x" % struct.unpack("<Q", struct.pack("<d", float(x)))[0]

def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("out", type=Path)
    ap.add_argument("--phase", choices=["discriminator", "heldout"], required=True)
    ap.add_argument("--seed", type=int, default=202609291)
    args = ap.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    manifest = {"generator": Path(__file__).name, "phase": args.phase, "seed": args.seed,
                "comparison_policy": "exact_typed_bit_match_no_tolerance", "batches": []}
    for fn in FUNCTIONS:
        seen = set()
        if args.phase == "heldout":
            paths = [ROOT / f"smart-fuzzer/runs/w111-broad-20260929/answers/answers-{fn.lower()}.json"]
            for p in paths:
                if p.exists():
                    seen.update(tuple(w["args"]) for w in json.loads(p.read_text(encoding="utf-8-sig"))["witnesses"])
            disc = args.out.parent / "discriminator" / f"batch-{fn.lower()}.json"
            if disc.exists():
                seen.update(tuple(w["probe"]["args"]) for w in json.loads(disc.read_text())["probes"])
        rows = {}
        excluded_input_plumbing = set()
        def add(n, s, tag):
            key = (bits(n), bits(s))
            if args.phase == "heldout" and any(
                (0 < abs(float(v)) < math.ldexp(1.0,-1022)) or bits(v) == "0x8000000000000000"
                for v in (n,s)
            ):
                # Root's separate Value2 readback observed these source bits
                # becoming +0 in cell storage. They cannot judge this numeric
                # kernel against the unchanged source tuple; never normalize.
                excluded_input_plumbing.add(key)
                return
            if key not in seen: rows.setdefault(key, tag)
        if args.phase == "discriminator":
            operands = [-1, -0.5, -0.0, 0, math.ldexp(1.0,-1022), 0.1, 0.9, 1, 1.5, 2, 3,
                        2**24-1, 2**24, 2**47, MAX, math.nextafter(MAX, math.inf), 2**48]
            if "SHIFT" in fn:
                shifts = [-1e300, -2**32, -54, -53.99, -53, -52.99, -49, -48.99, -48, -47.99,
                          -1.5, -0.999, -0.0, 0, 0.999, 1.5, 47.99, 48, 48.99, 49, 52.99, 53, 53.99, 54, 2**32, 1e300]
                for n in operands:
                    for s in shifts: add(n, s, "integer-validation-count-order-zero-shortcut-48-boundary")
            else:
                for n in operands:
                    for s in operands: add(n, s, "operand-integer-and-domain-admission")
        else:
            rng = random.Random(f"{args.seed}/{fn}")
            for _ in range(4500):
                n = rng.randrange(2**48)
                if rng.randrange(3) == 0: n += rng.choice([0.125, 0.25, 0.5, 0.75])
                if rng.randrange(10) == 0: n = rng.choice([-n, math.ldexp(1.0,-1022), 2**48+rng.randrange(128)])
                if "SHIFT" in fn:
                    s = rng.choice([rng.uniform(-55,55), rng.randint(-56,56), rng.choice([-1,1])*10**rng.uniform(2,300)])
                else:
                    s = rng.randrange(2**48) + rng.choice([0,0,0,0.125,0.5])
                    if rng.randrange(10) == 0: s *= -1
                add(n,s,"independent-random")
            if "SHIFT" in fn:
                # Distinct half/fractional neighbours at the overflow and count boundaries.
                for k in range(-55,56):
                    for n in [0, 1, 2, 3, 7, 2**17-1, 2**31+1, 2**47-1, MAX]:
                        for s in [math.nextafter(float(k),-math.inf),math.nextafter(float(k),math.inf)]:
                            add(n,s,"adjacent-count-boundary")
        doc = {"function": fn, "probes": [{"probe": {"id": f"w111bit-{args.phase}-{fn}-{i:05d}",
              "args": list(values)}, "probe_region": tag} for i,(values,tag) in enumerate(rows.items())]}
        path = args.out / f"batch-{fn.lower()}.json"
        path.write_text(json.dumps(doc,indent=2)+"\n",encoding="utf-8")
        manifest["batches"].append({"function":fn,"path":path.name,"rows":len(rows),
                                    "sha256":hashlib.sha256(path.read_bytes()).hexdigest(),"excluded_prior_tuples":len(seen),
                                    "excluded_input_plumbing_tuples":len(excluded_input_plumbing),
                                    "input_plumbing_exclusion_reason":"subnormal and negative-zero source bits become positive zero in live Value2 storage; no input or expectation normalization"})
    (args.out / "manifest.json").write_text(json.dumps(manifest,indent=2)+"\n",encoding="utf-8")
    print(json.dumps(manifest))

if __name__ == "__main__": main()
