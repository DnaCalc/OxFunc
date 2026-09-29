"""Oracle-blind typed SLN heldout after the scalar/surface candidate freeze."""
import hashlib
import json
import random
from pathlib import Path
import gen_broad_typed_20260929 as g

out = g.ROOT / "smart-fuzzer/cache/w111-sln-20260929"
g.TRANCHE = "w111-sln-typed-heldout-20260929"
prior = json.loads((out/"typed.json").read_text())["cases"]
seen = {json.dumps(c["args"], sort_keys=True) for c in prior}
rng = random.Random(202609295)
values = [g.n(x) for x in (-21, -1.5, 0, .5, 2, 11, 1e-13, 1e100)]
values += [g.t(x) for x in ("", " ", "-2", "2.5", "abc")]
values += [g.b(True), g.b(False), g.blank(), g.missing(), g.e(), g.e("Div0"), g.e("Num")]

def emit(args, tag, fixtures=()):
    key = json.dumps(args, sort_keys=True)
    if key not in seen:
        seen.add(key)
        g.emit("SLN", tag, args, fixtures)

for i in range(225):
    emit([rng.choice(values) for _ in range(3)], f"typed-triple-{i}")
for pos in range(3):
    for i in range(10):
        args = [g.n(rng.randint(-99, 99)) for _ in range(3)]
        args[pos] = g.a([[rng.choice(values[:-7]) for _ in range(3)] for _ in range(2)])
        emit(args, f"array-{pos}-{i}")
    for i,value in enumerate([g.t("-2.5"),g.blank(),g.b(False),g.e("Div0")]):
        args = [g.n(37),g.n(-9),g.n(4)]; args[pos] = g.r(f"C{i+1}")
        emit(args, f"reference-{pos}-{i}", [g.fix(f"C{i+1}",value)])

for case in g.cases: case["case_id"] = case["case_id"].replace("w111typed-", "w111slnheldout-")
doc = {"schema_version":"oxfunc.smart_fuzzer.scenario_seed_case_set.v0", "tranche_id":g.TRANCHE,
       "generator_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),"seed":202609295,
       "excluded_prior_argument_tuples":len(prior),"cases":g.cases,
       "tranches":[{"tranche_id":g.TRANCHE,"case_ids":[c["case_id"] for c in g.cases]}]}
path = out/"typed-heldout.json"
path.write_text(json.dumps(doc,indent=2)+"\n",encoding="utf-8")
path.with_suffix(".jsonl").write_text("".join(json.dumps(case)+"\n" for case in g.cases),encoding="utf-8")
print(json.dumps({"path":str(path),"rows":len(g.cases)}))
