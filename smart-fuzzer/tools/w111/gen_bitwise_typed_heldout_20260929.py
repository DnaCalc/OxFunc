"""Post-freeze independent bitwise typed probes; parent owns live Excel capture."""
import hashlib
import json
import random
from pathlib import Path
import gen_broad_typed_20260929 as g

g.TRANCHE = "w111-bitwise-typed-heldout-20260929"
rng = random.Random(202609293)
prior = json.loads((g.ROOT / "smart-fuzzer/cache/w111-bitwise-20260929/typed.json").read_text())
seen = {(case["function_id"], json.dumps(case["args"], sort_keys=True)) for case in prior["cases"]}

def emit(fn, args, tag):
    key = ("FUNC." + fn, json.dumps(args, sort_keys=True))
    if key not in seen:
        seen.add(key)
        g.emit(fn, tag, args)

values = [g.n(x) for x in (-2, -.5, 0, .5, 1, 4, 47, 48, 49, 53, 54, 2**47, 2**48-1, 2**48)]
values += [g.t(x) for x in ("", " ", "0", "4", "4.5", " 4 ", "1e1", "TRUE", "xyz")]
values += [g.b(False), g.b(True), g.blank(), g.missing(), g.e(), g.e("Div0"), g.e("Num")]
for fn in ("BITAND", "BITOR", "BITXOR", "BITLSHIFT", "BITRSHIFT"):
    for value in values:
        emit(fn, [g.missing(), value], "missing-left")
        emit(fn, [value, g.missing()], "missing-right")
    for i in range(90):
        emit(fn, [rng.choice(values), rng.choice(values)], f"typed-pair-{i}")
    # A missing scalar broadcasts across an independently selected typed grid.
    grid = g.a([[g.n(7), g.n(49), g.t("5")], [g.e("Div0"), g.n(-.5), g.b(False)]])
    emit(fn, [g.missing(), grid], "missing-array-right")
    emit(fn, [grid, g.missing()], "array-left-missing")

for case in g.cases:
    case["case_id"] = case["case_id"].replace("w111typed-", "w111bitheldout-")
out = g.ROOT / "smart-fuzzer/cache/w111-bitwise-20260929/typed-heldout.json"
doc = {"schema_version":"oxfunc.smart_fuzzer.scenario_seed_case_set.v0",
       "authority":"non_semantic_exploration_input", "tranche_id":g.TRANCHE,
       "generator":str(Path(__file__).relative_to(g.ROOT)),
       "generator_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
       "seed":202609293, "excluded_prior_args":len(prior["cases"]),
       "comparison_policy":"exact_typed_bit_match_no_tolerance", "cases":g.cases,
       "tranches":[{"tranche_id":g.TRANCHE,"case_ids":[c["case_id"] for c in g.cases]}],
       "summary":{"case_count":len(g.cases),"surfaces_covered":5}}
out.write_text(json.dumps(doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
out.with_suffix(".jsonl").write_text("".join(json.dumps(c, ensure_ascii=False)+"\n" for c in g.cases), encoding="utf-8")
print(json.dumps({"path":str(out), **doc["summary"]}))
