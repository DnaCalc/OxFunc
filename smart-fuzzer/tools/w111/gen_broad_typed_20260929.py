"""Build deterministic typed structural probes for the 2026-09-29 campaign.

Usage: python smart-fuzzer/tools/w111/gen_broad_typed_20260929.py [output.json]
Run with Run-ArraySupportTranche.ps1 -CaseSetPath <output.json> -RunId <unique>.
This is exploration input, not a semantic authority or a parity claim. Numbers
are written through Value2 fixtures; text/logical/error expressions preserve
direct-argument provenance. CHOOSE materializes arrays from those typed inputs.
Reference probes map every argument target explicitly in the local resolver.
"""
import copy
import csv
import hashlib
import json
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
TRANCHE = "w111-broad-typed-20260929"
def n(v): return {"kind": "number", "value": v}
def t(v): return {"kind": "text", "value": v}
def b(v): return {"kind": "logical", "value": v}
def e(v="NA"): return {"kind": "error", "code": v}
def blank(): return {"kind": "empty_cell"}
def missing(): return {"kind": "missing_arg"}
def a(rows): return {"kind": "array", "rows": rows}
def r(target): return {"kind": "reference", "reference_kind": "Area" if ":" in target else "A1", "target": target}
def fix(target, value): return {"target": target, "value": value}
ERR = {"NA": "#N/A", "Value": "#VALUE!", "Div0": "#DIV/0!", "Num": "#NUM!", "Ref": "#REF!", "Name": "#NAME?", "Null": "#NULL!"}
cases = []

def emit(fn, tag, args, fixtures=(), axis="typed_argument"):
    args = copy.deepcopy(args)
    fixtures = copy.deepcopy(list(fixtures))
    next_row = 100
    def token(v):
        nonlocal next_row
        k = v["kind"]
        if k in ("number", "empty_cell"):
            target = f"A{next_row}"
            next_row += 1
            fixtures.append(fix(target, copy.deepcopy(v)))
            return target
        if k == "text": return '"' + v["value"].replace('"', '""') + '"'
        if k == "logical": return "TRUE" if v["value"] else "FALSE"
        if k == "error": return ERR[v["code"]]
        if k == "missing_arg": return ""
        if k == "reference": return v["target"]
        if k == "array":
            rows = v["rows"]
            assert rows and rows[0] and all(len(row) == len(rows[0]) for row in rows)
            assert all(cell["kind"] != "empty_cell" for row in rows for cell in row), "blank arrays need reference fixtures"
            indices, tokens, index = [], [], 1
            for row in rows:
                indices.append(",".join(str(i) for i in range(index, index + len(row))))
                tokens.extend(token(cell) for cell in row)
                index += len(row)
            return "CHOOSE({" + ";".join(indices) + "}," + ",".join(tokens) + ")"
        raise ValueError(k)
    formula = "=" + fn + "(" + ",".join(token(v) for v in args) + ")"
    cases.append({
        "schema_version": "oxfunc.smart_fuzzer.scenario_seed_case.v0", "run_id": "assigned_by_runner",
        "tranche_id": TRANCHE, "case_id": f"w111typed-{len(cases):05d}-{fn.lower().replace('.', '_')}-{tag}",
        "function_id": "FUNC." + fn, "canonical_surface_name": fn, "case_tag": tag,
        "axis": axis, "expected_probe_class": "structural_exploration", "formula_text": formula,
        "args": args, "cell_fixture": fixtures, "formula_cell": "J20", "category": "typed_structural",
        "blocked_or_deferred_lanes": [], "known_deviation_tags": [],
    })

def variants(fn, args, positions=(0,), arrays=True, blanks=True):
    emit(fn, "valid-control", args)
    for pos in positions:
        for tag, value in [("na", e()), ("value-error", e("Value")), ("empty-text", t("")),
                           ("numeric-text", t("2")), ("logical", b(True))]:
            changed = copy.deepcopy(args); changed[pos] = value
            emit(fn, f"arg{pos+1}-{tag}", changed)
        if blanks:
            changed = copy.deepcopy(args); changed[pos] = blank()
            emit(fn, f"arg{pos+1}-blank-cell", changed)
        if arrays:
            changed = copy.deepcopy(args); changed[pos] = a([[args[pos], args[pos]]])
            emit(fn, f"arg{pos+1}-row-array", changed, axis="array_lift")

def main():
    # Text, information, conversion and logical surfaces: direct typed input
    # distinguishes coercion from typed cell-reference treatment.
    specs = {
        "ASC": [t("ABC１２３")], "DBCS": [t("ABC123")], "JIS": [t("ABC123")],
        "ENCODEURL": [t("a b/?x=é&z=1")], "CLEAN": [t("a\t b\n")], "TRIM": [t(" a  b ")],
        "LOWER": [t("AbCÉ")], "UPPER": [t("AbCé")], "PROPER": [t("one-TWO three")],
        "LEN": [t("A😀B")], "LENB": [t("ASCII")], "CODE": [t("A")], "UNICODE": [t("😀")],
        "CHAR": [n(65)], "UNICHAR": [n(128512)],
        "LEFT": [t("abcdef"), n(2)], "LEFTB": [t("abcdef"), n(2)],
        "RIGHT": [t("abcdef"), n(2)], "RIGHTB": [t("abcdef"), n(2)],
        "MID": [t("abcdef"), n(2), n(3)], "MIDB": [t("abcdef"), n(2), n(3)],
        "FIND": [t("b"), t("Abc")], "FINDB": [t("b"), t("Abc")],
        "SEARCH": [t("B"), t("abc")], "SEARCHB": [t("B"), t("abc")],
        "REPLACE": [t("abcdef"), n(2), n(3), t("X")], "REPLACEB": [t("abcdef"), n(2), n(3), t("X")],
        "REPT": [t("ab"), n(3)], "SUBSTITUTE": [t("a-b-a"), t("a"), t("x")],
        "EXACT": [t("Ab"), t("ab")], "CONCAT": [t("a"), t("b")], "CONCATENATE": [t("a"), t("b")],
        "TEXTJOIN": [t("-"), b(True), a([[t("a"), t(""), t("b")]])],
        "TEXTAFTER": [t("a-b-c"), t("-")], "TEXTBEFORE": [t("a-b-c"), t("-")],
        "TEXTSPLIT": [t("a,b,c"), t(",")],
        "REGEXTEST": [t("abc123"), t("[0-9]+")], "REGEXEXTRACT": [t("abc123"), t("[0-9]+")],
        "REGEXREPLACE": [t("abc123"), t("[0-9]+"), t("X")],
        "NUMBERVALUE": [t("1.234,5"), t(","), t(".")], "COMPLEX": [n(2), n(3)],
        "ADDRESS": [n(2), n(3)], "ARABIC": [t("XIV")], "ROMAN": [n(14)],
        "BASE": [n(31), n(16)], "DECIMAL": [t("1F"), n(16)],
        "BIN2DEC": [t("101")], "BIN2HEX": [t("101")], "BIN2OCT": [t("101")],
        "OCT2DEC": [t("17")], "OCT2HEX": [t("17")], "OCT2BIN": [t("17")],
        "HEX2DEC": [t("1F")], "HEX2OCT": [t("1F")], "HEX2BIN": [t("1F")],
        "DEC2BIN": [n(5)], "DEC2OCT": [n(15)], "DEC2HEX": [n(31)],
        "T": [t("abc")], "N": [n(3)], "TYPE": [n(3)], "ERROR.TYPE": [e()],
        "ISBLANK": [n(3)], "ISERR": [n(3)], "ISERROR": [n(3)], "ISNA": [n(3)],
        "ISLOGICAL": [n(3)], "ISNONTEXT": [n(3)], "ISNUMBER": [n(3)], "ISTEXT": [n(3)],
        "ISEVEN": [n(3)], "ISODD": [n(3)], "NOT": [b(False)],
        "AND": [b(True), b(False)], "OR": [b(True), b(False)], "XOR": [b(True), b(False)],
        "IF": [b(True), n(7), n(9)], "IFERROR": [e(), n(9)], "IFNA": [e(), n(9)],
        "IFS": [b(False), n(7), b(True), n(9)], "SWITCH": [n(2), n(1), t("a"), n(2), t("b"), t("other")],
    }
    for fn, args in specs.items():
        variants(fn, args, arrays=fn not in {"TYPE", "ERROR.TYPE", "TEXTJOIN"})
    for fn in ("LEFT", "RIGHT", "LEFTB", "RIGHTB"):
        emit(fn, "omitted-count", [t("abc")]); emit(fn, "missing-count", [t("abc"), missing()])
    for fn in ("TEXTAFTER", "TEXTBEFORE"):
        for instance in (-2, -1, 0, 1, 2, 3):
            emit(fn, f"instance-{instance}", [t("a-b-c"), t("-"), n(instance)])
        emit(fn, "not-found-fallback", [t("abc"), t("-"), missing(), missing(), missing(), t("absent")])
    for fn in ("FIND", "SEARCH"):
        for find in ("*", "?", "~*", "", "z"):
            emit(fn, "pattern-" + str(len(cases)), [t(find), t("a*b?c"), n(1)])
    for fn in ("REPT", "CHAR", "UNICHAR", "ROMAN"):
        emit(fn, "negative-domain", ([t("a"), n(-1)] if fn == "REPT" else [n(-1)]))

    # Shape and optional-argument axes. The scalar publication of a 1x1 result
    # remains a separately classified downstream seam in the existing runner.
    matrix = a([[n(3), n(1)], [n(2), n(4)]])
    array_specs = {
        "TAKE": [matrix, n(1)], "DROP": [matrix, n(1)], "CHOOSEROWS": [matrix, n(2)],
        "CHOOSECOLS": [matrix, n(2)], "TOCOL": [matrix], "TOROW": [matrix], "TRANSPOSE": [matrix],
        "SORT": [matrix], "SORTBY": [matrix, a([[n(2)], [n(1)]])], "UNIQUE": [matrix],
        "HSTACK": [matrix, a([[n(8)], [n(9)]])], "VSTACK": [matrix, a([[n(8), n(9)]])],
        "EXPAND": [matrix, n(3), n(3), t("pad")], "WRAPROWS": [a([[n(1), n(2), n(3)]]), n(2), t("pad")],
        "WRAPCOLS": [a([[n(1), n(2), n(3)]]), n(2), t("pad")],
        "FILTER": [matrix, a([[b(True)], [b(False)]])], "SEQUENCE": [n(2), n(3), n(-1), n(2)],
        "MDETERM": [matrix], "MMULT": [matrix, matrix],
        "MINVERSE": [matrix], "MUNIT": [n(2)], "ARRAYTOTEXT": [matrix], "VALUETOTEXT": [n(3)],
        "CHOOSE": [n(2), t("a"), t("b"), t("c")],
    }
    for fn, args in array_specs.items():
        variants(fn, args, arrays=False, blanks=False)
    for fn in ("TAKE", "DROP"):
        emit(fn, "missing-rows-columns", [matrix, missing(), n(1)])
        emit(fn, "last-row", [matrix, n(-1)])
        emit(fn, "empty-count", [matrix, n(0)])

    # Numeric fixtures in true reference arguments stay references locally;
    # no global one-size conversion of reference arguments into value arrays.
    mixed = a([[n(1)], [t("2")], [b(True)], [blank()], [n(4)]])
    refs = [fix("A1:A5", mixed)]
    for fn in ("SUM", "PRODUCT", "AVERAGE", "AVERAGEA", "COUNT", "COUNTA", "COUNTBLANK", "MIN", "MAX", "MINA", "MAXA", "MEDIAN", "STDEV.S", "STDEV.P", "VAR.S", "VAR.P", "DEVSQ", "SUMSQ"):
        emit(fn, "mixed-reference", [r("A1:A5")], refs, "reference_coercion")
        with_error = copy.deepcopy(mixed); with_error["rows"][3][0] = e("Div0")
        emit(fn, "error-reference", [r("A1:A5")], [fix("A1:A5", with_error)], "reference_error")
        if fn != "COUNTBLANK":
            emit(fn, "direct-mixed-values", [n(1), t("2"), b(True), n(4)], axis="direct_vs_reference")
    for fn in ("AGGREGATE", "SUBTOTAL"):
        for selector in (1, 2, 3, 4, 5, 9):
            args = [n(selector), n(6), r("A1:A5")] if fn == "AGGREGATE" else [n(selector), r("A1:A5")]
            emit(fn, f"selector-{selector}-mixed", args, refs, "reference_coercion")

    keys = a([[t("alpha")], [t("Beta")], [t("alpha")], [t("gamma")]])
    values = a([[n(10)], [n(20)], [n(30)], [n(40)]])
    criteria_fixtures = [fix("A1:A4", keys), fix("B1:B4", values)]
    for criterion in ("alpha", "*a*", "<>alpha", "missing", "~*"):
        for fn in ("COUNTIF", "SUMIF", "AVERAGEIF"):
            args = [r("A1:A4"), t(criterion)] + ([] if fn == "COUNTIF" else [r("B1:B4")])
            emit(fn, "criterion-" + str(len(cases)), args, criteria_fixtures, "reference_criteria")
        for fn in ("COUNTIFS", "SUMIFS", "AVERAGEIFS", "MINIFS", "MAXIFS"):
            args = ([] if fn == "COUNTIFS" else [r("B1:B4")]) + [r("A1:A4"), t(criterion)]
            emit(fn, "criterion-" + str(len(cases)), args, criteria_fixtures, "reference_criteria")
    for fn in ("MATCH", "XMATCH", "XLOOKUP"):
        for needle in (t("alpha"), t("BETA"), t("missing"), e()):
            args = [needle, r("A1:A4")]
            args += [r("B1:B4"), t("absent")] if fn == "XLOOKUP" else [n(0)]
            emit(fn, "needle-" + str(len(cases)), args, criteria_fixtures, "lookup")
    table = a([[n(1), t("a")], [n(2), t("b")], [n(3), t("c")]])
    for fn in ("VLOOKUP", "HLOOKUP"):
        grid = table if fn == "VLOOKUP" else a([[n(1), n(2), n(3)], [t("a"), t("b"), t("c")]])
        target = "A1:B3" if fn == "VLOOKUP" else "A1:C2"
        for needle in (n(2), n(9), t("2"), e()):
            emit(fn, "needle-" + str(len(cases)), [needle, r(target), n(2), b(False)], [fix(target, grid)], "lookup")
    for fn in ("INDEX", "ROWS", "COLUMNS", "AREAS"):
        args = [r("A1:B3")] + ([n(2), n(2)] if fn == "INDEX" else [])
        emit(fn, "explicit-grid-reference", args, [fix("A1:B3", table)], "reference")
    # Plain value-cell introspection is supported; formula-state and sheet
    # identity probes are deliberately not manufactured by this local resolver.
    for fn in ("ISFORMULA", "FORMULATEXT"):
        emit(fn, "plain-value-cell", [r("A1")], [fix("A1", n(7))], "reference")

    database = a([[t("kind"), t("amount")], [t("a"), n(2)], [t("b"), n(4)], [t("a"), n(6)]])
    for fn in ("DAVERAGE", "DCOUNT", "DCOUNTA", "DGET", "DMAX", "DMIN", "DPRODUCT", "DSTDEV", "DSTDEVP", "DSUM", "DVAR", "DVARP"):
        for criterion in ("a", "b", "missing"):
            fixtures = [fix("A1:B4", database), fix("D1:D2", a([[t("kind")], [t(criterion)]]))]
            emit(fn, "database-" + criterion, [r("A1:B4"), t("amount"), r("D1:D2")], fixtures, "database_criteria")

    ledger = {row["function_id"]: row["status"] for row in csv.DictReader((ROOT / "docs/function-lane/EXCEL_PARITY_LEDGER.csv").open(encoding="utf-8-sig"))}
    # Stable priority order: fresh evidence for Unverified first.
    cases.sort(key=lambda c: (ledger.get(c["function_id"]) != "unverified", c["function_id"], c["case_id"]))
    for case in cases: case["ledger_status_at_generation"] = ledger.get(case["function_id"], "absent")
    assert len({c["case_id"] for c in cases}) == len(cases)
    counts = Counter(c["canonical_surface_name"] for c in cases)
    unverified = sorted(fn for fn in counts if ledger.get("FUNC." + fn) == "unverified")
    doc = {"schema_version": "oxfunc.smart_fuzzer.scenario_seed_case_set.v0", "authority": "non_semantic_exploration_input",
           "generator": str(Path(__file__).relative_to(ROOT)), "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
           "tranche_id": TRANCHE, "comparison_policy": "exact_typed_bit_match_no_tolerance", "cases": cases,
           "tranches": [{"tranche_id": TRANCHE, "case_ids": [c["case_id"] for c in cases]}],
           "skipped": ["locale_provider", "volatile", "callable", "cross_sheet", "formula_state", "rich_value", "reference_identity"],
           "summary": {"case_count": len(cases), "surfaces_covered": len(counts), "previously_unverified": unverified, "by_function": dict(sorted(counts.items()))}}
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "smart-fuzzer/cache/w111-broad-typed-20260929.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    out.with_suffix(".jsonl").write_text("".join(json.dumps(c, ensure_ascii=False) + "\n" for c in cases), encoding="utf-8")
    print(json.dumps({"path": str(out), **doc["summary"]}, ensure_ascii=False))

if __name__ == "__main__": main()
