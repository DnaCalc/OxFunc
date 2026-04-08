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

## 7. Follow-up Closure Notes
1. Operational hardening for FP-C registration is complete:
   - self-registration via `xlfRegister` in `xlAutoOpen`,
   - official SDK callback bridge (`XLCALL.H` + `XLCALL.CPP`) in harness build path.
2. SDK bootstrap is locked and scripted:
   - `tools/fp-probe/xll/EXCEL_XLL_SDK_LOCK.json`
   - `tools/fp-probe/xll/fetch-excel-xll-sdk.ps1`
   - `tools/fp-probe/xll/README.md`
   - includes controlled lock rotation path (`-UpdateLock`) for audited hash drift updates.
3. Foundation editorial handoff prompts are prepared:
   - now preserved behind `docs/HISTORY.md` and the `OxFunc_V1` tag.

## 8. Focused Follow-On - Numeric Comparison Families
Execution date:
1. `2026-04-08`

Environment:
1. Excel version/build: `16.0 (build 19822)`
2. Excel channel: `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`
3. Workbook compatibility descriptor:
   - `default|CalculationVersion=191029|CheckCompatibility=False|FileFormat=51`
4. Locale profile: `en-US`

Output artifacts:
1. `.tmp/fp-results-excel-e.csv` (`FP-E`)
2. `.tmp/w45-waveb-operator-compare-concat-results.csv` (refreshed `W45-B`)

Observation summary:
1. Excel `FP-E` rows captured: `15` (`observed=15`)
2. Scope covered:
   - `IF` / `IFS` empty-text condition coercion
   - ordinary compare operators
   - criteria/database numeric criteria matching
   - `SWITCH`
   - exact-match contrast for `MATCH`, `XMATCH`, `DELTA`
   - arithmetic-generated 15-significant-digit boundary rows

Key outcomes:
1. `IF("",1,2)` and `IFS("",1,TRUE,2)` both surfaced `#VALUE!`.
2. `W45-B` ordinary compare rows now include and pass the near-equality cases:
   - `0.1+0.2=0.3 -> TRUE`
   - `0.1+0.2<>0.3 -> FALSE`
   - `0.1+0.2<0.3 -> FALSE`
   - `0.1+0.2<=0.3 -> TRUE`
   - `0.1+0.2>0.3 -> FALSE`
   - `0.1+0.2>=0.3 -> TRUE`
   - `((123456789012345*10)+5)/1E25=((123456789012345*10)+4)/1E25 -> TRUE`
   - `((123456789012345*10)+5)/1E25<>((123456789012345*10)+4)/1E25 -> FALSE`
   - ordered comparisons on the same pair collapse to `FALSE/FALSE/TRUE/TRUE`
     for `>/<`/`>=`/`<=`.
3. Criteria/database numeric criteria matching and `SWITCH` share the tolerant
   near-equality lane on both the baseline `0.1+0.2` rows and the stronger
   arithmetic-generated boundary rows.
4. `MATCH`, `XMATCH`, and `DELTA` exact-match paths remained exact on both the
   baseline `0.1+0.2` rows and the arithmetic-generated boundary rows.
5. The earlier round-to-nearest helper model was disproved by the stronger
   boundary pair; the current local helper now follows the empirically pinned
   truncation-style 15-significant-digit comparison lane.

Current use:
1. this follow-on evidence reopens the previously promoted criteria/database and
   `SWITCH` rows under `W077`,
2. the family split is now bound into the corrected local runtime helper policy
   and the downstream reply handoff `HO-FN-008`.
