"""Retain compact, lossless bitwise oracle evidence; no Excel calls."""
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CACHE = ROOT / "smart-fuzzer/cache/w111-bitwise-20260929"
OUT = ROOT / "docs/function-lane/evidence/w111-broad-20260929/bitwise"
OUT.mkdir(parents=True, exist_ok=True)
manifest = {"schema_version":"w111.bitwise_evidence.v1", "artifacts":[], "numeric_phases":{},
            "scope_completeness":"scope_partial", "target_completeness":"target_partial",
            "integration_completeness":"partial",
            "open_lanes":["current-baseline shared numeric-text discrepancies (see PROMOTION_AUDIT.md)",
                          "remaining array precedence coverage", "current-phase review and promotion",
                          "alternate version and locale phases"]}

def retain(src, relative):
    dst = OUT / relative
    dst.parent.mkdir(parents=True, exist_ok=True)
    if src.suffix == ".jsonl":
        value = [json.loads(line) for line in src.read_text(encoding="utf-8-sig").splitlines() if line.strip()]
        dst.write_text("".join(json.dumps(row, ensure_ascii=False, separators=(",", ":"))+"\n" for row in value), encoding="utf-8")
    else:
        value = json.loads(src.read_text(encoding="utf-8-sig"))
        dst.write_text(json.dumps(value, ensure_ascii=False, separators=(",", ":"))+"\n", encoding="utf-8")
    manifest["artifacts"].append({"source":str(src.relative_to(ROOT)).replace("\\", "/"),
        "source_sha256":hashlib.sha256(src.read_bytes()).hexdigest(), "path":relative,
        "retained_sha256":hashlib.sha256(dst.read_bytes()).hexdigest()})
    return value

witnesses = []
for phase in ("discovery", "discriminator", "heldout"):
    rows = ingress_changed = 0
    folder = ROOT / "smart-fuzzer/runs/w111-broad-20260929/answers" if phase == "discovery" else CACHE / (phase+"-answers")
    for fn in ("bitand", "bitor", "bitxor", "bitlshift", "bitrshift"):
        name = f"answers-{fn}.json" if phase == "discovery" else f"answers-batch-{fn}.json"
        relative = f"{phase}/answers-{fn}.json"
        data = retain(folder / name, relative)
        rows += len(data["witnesses"])
        def source_changes(bits):
            n = int(bits, 16)
            return n == 0x8000000000000000 or (n & 0x7ff0000000000000 == 0 and n & 0xfffffffffffff != 0)
        ingress_changed += sum(any(source_changes(arg) for arg in row["args"]) for row in data["witnesses"])
        witnesses.append(str(OUT / relative))
    manifest["numeric_phases"][phase] = {"raw_rows":rows,
        "source_ingress_changed_rows":ingress_changed, "qualified_rows":rows-ingress_changed,
        "qualification":"Negative zero/subnormal source tuples remain banked but are excluded from same-input semantic evidence; see ../numeric-ingress.json"}
    if phase != "discovery":
        retain(CACHE / phase / "manifest.json", phase+"/input-manifest.json")
for version in (1, 2):
    retain(CACHE / f"freeze-v{version}.json", f"freeze-v{version}.json")
for phase, run in [("typed-discovery", "w111-bitwise-typed-20260929"),
                   ("typed-heldout", "w111-bitwise-typed-heldout-20260929")]:
    for file in ("manifest.json", "rollup.json", "cases/cases.jsonl", "outcomes/excel.jsonl", "outcomes/local.jsonl", "comparisons/comparisons.jsonl"):
        retain(ROOT / "smart-fuzzer/runs" / run / file, phase+"/"+file)
retain(CACHE / "typed-v2-local.jsonl", "typed-discovery/outcomes/local-final.jsonl")
result = subprocess.run(["cargo", "run", "-q", "--manifest-path", str(ROOT/"smart-fuzzer/engine/Cargo.toml"),
                         "--", "judge-witnesses", *witnesses, "--max-misses", "20", "--json"],
                        cwd=ROOT, capture_output=True, text=True, check=True)
(OUT / "final-numeric-judgement.json").write_text(result.stdout, encoding="utf-8")
judgements = json.loads(result.stdout)
assert all(report["rows_agree"] == report["rows_judged"] for report in judgements), "Retained numeric replay has semantic misses"
(OUT / "artifact-manifest.json").write_text(json.dumps(manifest, indent=2)+"\n", encoding="utf-8")
print(json.dumps({"evidence":str(OUT),"artifacts":len(manifest["artifacts"]),
                  "numeric_judgement":str(OUT / "final-numeric-judgement.json")}))
