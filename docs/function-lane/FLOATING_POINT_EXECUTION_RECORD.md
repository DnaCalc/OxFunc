# Floating-Point Characterization Execution Record

Status: `active`
Workset: `W2`
Conformance row seed: `FDEF-027`

## 1. Purpose
Track execution status and reproducible evidence for W2 floating-point characterization.

## 2. Executed Baseline (Current)
Execution date:
1. `2026-03-05`

Environment:
1. Excel version/build: `16.0 (build 19725)`
2. Excel channel: `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`
3. Workbook compatibility descriptor:
   - `default|CalculationVersion=191029|CheckCompatibility=False|FileFormat=51`
4. Locale profile: `en-US`
5. Lean toolchain: `leanprover/lean4:v4.28.0`

Output artifacts:
1. `.tmp/fp-results-excel-abd.csv` (`FP-A`, `FP-B`, `FP-D`)
2. `.tmp/fp-results-excel-c.csv` (`FP-C`, Rust XLL ingress)
3. `.tmp/fp-results-excel-all.csv` (merged Excel rows)
4. `.tmp/fp-results-lean.csv` (Lean comparative rows)
5. `.tmp/fp-artifacts/*` (workbook/CSV per-scenario artifacts for persistence lanes)

## 3. Observation Summary
1. Excel rows captured: `26` (`observed=26`)
   - `FP-A=10`, `FP-B=6`, `FP-C=6`, `FP-D=4`
2. Lean comparable rows captured: `10` (`observed=10`)
3. Deviation ledger stitched:
   - relation status counts: `aligned=1`, `divergent=9`

## 4. Key Characterization Outcomes
1. Formula-surface signed zero:
   - `FP-A` paths normalize display to `0`.
2. Interop (`FP-C`) special values via Rust XLL:
   - `+inf`, `-inf`, `qNaN`, `sNaN` normalize to `#NUM!`.
   - `-0` from UDF is observable as `-0`.
3. Divide-by-zero/invalid operations at worksheet surface:
   - `0/0`, `1/0`, `-1/0` surfaced as `#DIV/0!`.
4. Tiny/subnormal candidates:
   - Excel surface and `Value2` in tested scenarios were `0`.
   - Lean runtime preserved nonzero subnormal bit patterns for comparable cases.

## 5. Gate Tracking
### G1 - Scenario Closure
1. Status: `closed`.
2. Evidence: explicit manifest + executable op fields in `FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv`.

### G2 - Observation Closure
1. Status: `closed` (baseline build/channel + default compatibility + locale).
2. Evidence:
   - Excel: `.tmp/fp-results-excel-all.csv`
   - Lean: `.tmp/fp-results-lean.csv`

### G3 - Characterization Closure
1. Status: `closed-provisional`.
2. Evidence:
   - divergence ledger populated:
     - `FLOATING_POINT_LEAN_EXCEL_DEVIATION_LEDGER.csv`
   - policy notes updated:
     - `FLOATING_POINT_BEHAVIOR_RESEARCH_NOTES.md`

### G4 - Promotion Closure
1. Status: `closed-provisional`.
2. Evidence:
   - promotion candidates and explicit deferral rationale documented in research notes.

## 6. Next Expansion Actions (Post-Baseline)
1. Re-run matrix across additional Excel channels/builds.
2. Add non-default workbook compatibility template runs.
3. Add multi-locale follow-up beyond `en-US`.
4. Promote finalized candidates into Foundation `EMP-*` registry after editorial review.
