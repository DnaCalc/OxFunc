"""Exact-bit SLN probing; this generator does not call Excel."""
import argparse
import hashlib
import json
import math
import random
import struct
from pathlib import Path
import gen_broad_typed_20260929 as g

def bits(v): return "0x%016x" % struct.unpack("<Q", struct.pack("<d", v))[0]
def decode(v): return struct.unpack("<d", struct.pack("<Q", v))[0]

ap = argparse.ArgumentParser(description=__doc__)
ap.add_argument("--phase", choices=("discriminator", "heldout"), default="discriminator")
args = ap.parse_args()
out = g.ROOT / "smart-fuzzer/cache/w111-sln-20260929"
out.mkdir(parents=True, exist_ok=True)
rows = {}
prior = set()
if args.phase == "heldout":
    for source in [g.ROOT / "smart-fuzzer/runs/w111-broad-20260929/answers/answers-sln.json", out / "discriminator-answers.json"]:
        prior.update(tuple(row["args"]) for row in json.loads(source.read_text(encoding="utf-8-sig"))["witnesses"])

def add(values, axis):
    assert all(math.isfinite(v) and (v == 0 or abs(v) >= 2**-1022) and bits(v) != "0x8000000000000000" for v in values)
    key = tuple(map(bits, values))
    if key not in prior: rows.setdefault(key, axis)

if args.phase == "discriminator":
    maximum = decode(0x7fefffffffffffff)
    pairs = [(0., 0.), (1., 1.), (-1., -1.), (3., 7.), (7., 3.), (-3., 7.), (3., -7.),
             (2**-1022, 0.), (-2**-1022, 0.), (2**-1022, math.nextafter(2**-1022, math.inf)),
             (maximum, -maximum), (-maximum, maximum), (maximum, maximum),
             (maximum, math.nextafter(maximum, 0)), (1e200, -1e200), (1e-200, 0.), (-1e-200, 0.),
             (math.nextafter(1., math.inf), 1.), (1., math.nextafter(1., math.inf))]
    lives = [0., 1., -1., 2., -2., 10., -10., 1e-11, -1e-11, 1e-12, -1e-12, 1e-13, -1e-13,
             2**-1022, -2**-1022, 1e-200, -1e-200, 1e200, -1e200, maximum, -maximum]
    for pair in pairs:
        for life in lives: add((*pair, life), "sign-zero-tiny-life-overflow-underflow-cancellation")
else:
    rng = random.Random(202609294)
    for _ in range(6500):
        # Normal binary64 operands over the full exponent range. No Value2
        # changed-ingress inputs or previously observed exact tuples enter.
        values = [decode((rng.getrandbits(1)<<63) | (rng.randrange(1,2047)<<52) | rng.getrandbits(52)) for _ in range(3)]
        add(values, "independent-full-range-normal")
    for _ in range(750):
        value = rng.uniform(-1e10, 1e10)
        add((value, math.nextafter(value, math.inf), rng.choice([-1.,1.])*10**rng.uniform(-250,250)), "adjacent-cancellation")

doc = {"function":"SLN", "probes":[{"probe":{"id":f"w111sln-{args.phase}-{i:05d}","args":list(row)},"probe_region":axis} for i,(row,axis) in enumerate(rows.items())]}
path = out / (args.phase + ".json")
path.write_text(json.dumps(doc, indent=2)+"\n", encoding="utf-8")
(out/(args.phase+"-manifest.json")).write_text(json.dumps({"phase":args.phase,"rows":len(rows),"excluded_prior_tuples":len(prior),"generator_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),"batch_sha256":hashlib.sha256(path.read_bytes()).hexdigest(),"ingress":"normal binary64 plus positive zero only"},indent=2)+"\n",encoding="utf-8")
print(json.dumps({"path":str(path),"rows":len(rows)}))

if args.phase == "discriminator":
    g.TRANCHE = "w111-sln-typed-20260929"
    g.variants("SLN", [g.n(30),g.n(7.5),g.n(10)], positions=(0,1,2))
    for pos in range(3):
        for value in [g.missing(),g.t("abc"),g.t("-3.5"),g.n(-3),g.e("Div0")]:
            inputs = [g.n(30),g.n(7.5),g.n(10)]; inputs[pos] = value
            g.emit("SLN", f"arg{pos+1}-typed-{len(g.cases)}", inputs)
    g.emit("SLN", "rectangular-broadcast", [g.a([[g.n(10),g.n(20)],[g.n(-10),g.e()]]),g.n(2),g.a([[g.n(2),g.n(-2)]])])
    g.emit("SLN", "all-explicit-missing", [g.missing(),g.missing(),g.missing()])
    for case in g.cases: case["case_id"] = case["case_id"].replace("w111typed-", "w111sln-")
    typed = {"schema_version":"oxfunc.smart_fuzzer.scenario_seed_case_set.v0", "tranche_id":g.TRANCHE,"cases":g.cases,"tranches":[{"tranche_id":g.TRANCHE,"case_ids":[c["case_id"] for c in g.cases]}]}
    (out/"typed.json").write_text(json.dumps(typed,indent=2)+"\n",encoding="utf-8")
    (out/"typed.jsonl").write_text("".join(json.dumps(case)+"\n" for case in g.cases),encoding="utf-8")
    print(json.dumps({"path":str(out/"typed.json"),"rows":len(g.cases)}))
