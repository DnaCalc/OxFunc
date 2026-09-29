"""Independent typed/array/reference probes for the repaired bitwise kernels.
Uses the same audited Value2/token writer as the broad typed corpus.
Usage: python smart-fuzzer/tools/w111/gen_bitwise_typed_20260929.py
"""
import hashlib
import json
from pathlib import Path
import gen_broad_typed_20260929 as g

g.TRANCHE = "w111-bitwise-typed-20260929"
for fn in ("BITAND", "BITOR", "BITXOR", "BITLSHIFT", "BITRSHIFT"):
    g.variants(fn, [g.n(3), g.n(2)], positions=(0, 1))
    for pos in (0,1):
        for tag,v in [("fractional-number",g.n(3.5)),("fractional-text",g.t("3.5")),
                      ("nonnumeric-text",g.t("xyz")),("missing",g.missing())]:
            args=[g.n(3),g.n(2)]; args[pos]=v
            g.emit(fn,f"arg{pos+1}-{tag}",args)
        for tag,v in [("numeric-text",g.t("3")),("logical",g.b(True)),("blank",g.blank()),("empty-text",g.t(""))]:
            args=[g.n(3),g.n(2)]; args[pos]=g.r("C1")
            g.emit(fn,f"arg{pos+1}-reference-{tag}",args,[g.fix("C1",v)],"reference_coercion")
    g.emit(fn,"paired-array",[g.a([[g.n(1),g.n(2)],[g.n(3.5),g.e()]]),g.a([[g.n(1),g.n(-1)],[g.n(0),g.n(2)]])],axis="array_lift")
    g.emit(fn,"left-na-right-div0",[g.e(),g.e("Div0")],axis="error_precedence")
    g.emit(fn,"left-div0-right-na",[g.e("Div0"),g.e()],axis="error_precedence")

for c in g.cases: c["case_id"] = c["case_id"].replace("w111typed-","w111bittyped-")
out=g.ROOT/"smart-fuzzer/cache/w111-bitwise-20260929/typed.json"
doc={"schema_version":"oxfunc.smart_fuzzer.scenario_seed_case_set.v0","authority":"non_semantic_exploration_input",
     "generator":"smart-fuzzer/tools/w111/gen_bitwise_typed_20260929.py",
     "generator_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
     "token_writer_sha256":hashlib.sha256(Path(g.__file__).read_bytes()).hexdigest(),
     "tranche_id":g.TRANCHE,"comparison_policy":"exact_typed_bit_match_no_tolerance","cases":g.cases,
     "tranches":[{"tranche_id":g.TRANCHE,"case_ids":[c["case_id"] for c in g.cases]}],
     "summary":{"case_count":len(g.cases),"surfaces_covered":5}}
out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(doc,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
out.with_suffix(".jsonl").write_text("".join(json.dumps(c,ensure_ascii=False)+"\n" for c in g.cases),encoding="utf-8")
print(json.dumps({"path":str(out),**doc["summary"]}))
