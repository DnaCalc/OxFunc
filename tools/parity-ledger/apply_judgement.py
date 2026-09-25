"""Judge banked Excel answers with `sf judge-witnesses`, promote the reduced evidence, and
update docs/function-lane/EXCEL_PARITY_LEDGER.csv rows. The stopgap for `sf report`
(W112-P6) until that exists.

Several answer files for the same function (e.g. two corpus seeds) are merged: rows summed,
worst severity and worst ULP kept.

Usage:
  python tools/parity-ledger/apply_judgement.py --evidence-dir docs/function-lane/evidence/<run> \
      --build 20430 --note "W111 G8-01 fix, seeds 20260924+20260925" answers1.json answers2.json ...
"""

import argparse
import csv
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SF = ROOT / "smart-fuzzer/engine/target/release/sf"
LEDGER = ROOT / "docs/function-lane/EXCEL_PARITY_LEDGER.csv"
ORDER = ["last_bit", "numeric", "gross", "structural"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--evidence-dir", required=True)
    ap.add_argument("--build", required=True)
    ap.add_argument("--note", required=True)
    ap.add_argument("--as-of", default=None)
    ap.add_argument("answers", nargs="+")
    a = ap.parse_args()

    out = subprocess.run([str(SF), "judge-witnesses", *a.answers, "--max-misses", "100000", "--json"],
                         capture_output=True, text=True, encoding="utf-8", check=True).stdout
    reports = json.loads(out)
    merged = {}
    for r in reports:
        m = merged.setdefault(r["function_id"], {"rows_judged": 0, "rows_agree": 0, "severity": None,
                                                 "worst_ulp": None, "misses_by_severity": {}, "misses": []})
        m["rows_judged"] += r["rows_judged"]
        m["rows_agree"] += r["rows_agree"]
        if r["severity"] and (m["severity"] is None or ORDER.index(r["severity"]) > ORDER.index(m["severity"])):
            m["severity"] = r["severity"]
        if r["worst_ulp"] is not None:
            m["worst_ulp"] = max(m["worst_ulp"] or 0, r["worst_ulp"])
        for k, v in r["misses_by_severity"].items():
            m["misses_by_severity"][k] = m["misses_by_severity"].get(k, 0) + v
        m["misses"] += r["misses"]
        prov = r.get("capture_provenance")
        if prov:
            m["capture_provenance"] = prov

    ev = ROOT / a.evidence_dir
    ev.mkdir(parents=True, exist_ok=True)
    lines = []
    for fid, m in sorted(merged.items()):
        lines.append(f"{fid:<24} {m['rows_agree']:>6}/{m['rows_judged']:<6} worst={m['severity'] or 'none':<10} "
                     f"worst_ulp={m['worst_ulp'] if m['worst_ulp'] is not None else '-'} {m['misses_by_severity']}")
    (ev / "summary.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")
    (ev / "misses.json").write_text(json.dumps(
        {fid: {k: v for k, v in m.items() if k != "capture_provenance"} for fid, m in sorted(merged.items())},
        ensure_ascii=False, indent=1), encoding="utf-8")
    prov = next((m["capture_provenance"] for m in merged.values() if m.get("capture_provenance")), None)
    (ev / "capture_provenance.json").write_text(json.dumps(prov, indent=1), encoding="utf-8")

    rows = list(csv.DictReader(LEDGER.open(encoding="utf-8")))
    cols = list(rows[0].keys())
    ref = f"{a.evidence_dir}/misses.json"
    as_of = a.as_of or __import__("datetime").date.today().isoformat()
    for row in rows:
        m = merged.get(row["function_id"])
        if not m:
            continue
        prev = row["status"] + (("/" + row["severity"]) if row["severity"] else "")
        row["as_of"] = as_of
        row["excel_builds_judged"] = a.build
        row["rows_judged"] = str(m["rows_judged"])
        row["rows_agree"] = str(m["rows_agree"])
        if m["severity"] is None:
            row["status"], row["severity"], row["worst_ulp"] = "consistent", "", "0"
            verdict = "all rows agree"
        else:
            row["status"], row["severity"] = "divergent", m["severity"]
            row["worst_ulp"] = "" if m["worst_ulp"] is None else str(m["worst_ulp"])
            verdict = f"misses {m['misses_by_severity']}"
        note = f"{a.note}: {m['rows_agree']}/{m['rows_judged']} on build {a.build}, {verdict}; was {prev}"
        row["exact_domain_note"] = note + ("; earlier: " + row["exact_domain_note"] if row["exact_domain_note"] else "")
        refs = [ref] + [x for x in row["evidence_refs"].split(";") if x and x != ref]
        row["evidence_refs"] = ";".join(refs)
    with LEDGER.open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=cols, lineterminator="\n")
        w.writeheader()
        w.writerows(rows)
    print("\n".join(lines))


if __name__ == "__main__":
    main()
