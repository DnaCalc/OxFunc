# Floating-Point Characterization Execution Record

Status: `active`
Workset: `W2`
Conformance row seed: `FDEF-027`

## 1. Purpose
Track execution readiness and observed progress for the W2 floating-point characterization matrix.

This file is the operational companion to:
1. `FLOATING_POINT_BEHAVIOR_RESEARCH_NOTES.md`
2. `FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv`

## 2. Current Baseline
1. Scenario seed manifest created (`FP2-001..FP2-026`).
2. Lanes covered in seed:
   - `FP-A` formula-only
   - `FP-B` reference-chain
   - `FP-C` interop ingress (currently harness-blocked)
   - `FP-D` persistence/text round-trip
3. Interop lane status:
   - blocked pending XLL/UDF harness setup for special-value injection.

## 3. Execution Metadata Contract
Each executed scenario row must capture:
1. scenario id,
2. Excel app version/channel,
3. workbook Compatibility Version,
4. locale profile,
5. observed result classification,
6. captured artifacts/paths,
7. runner/tool revision.
8. model source (`excel` or `lean-runtime`) for comparative rows.

## 3.1 Baseline Run Policy (Current)
1. Use local installed Excel for all worksheet-observation runs.
2. Record exact Excel build and channel on every run result row.
3. Start with workbook Compatibility Version default and record that value explicitly.
4. Run only `en-US` locale in this phase; multi-locale expansion is a later workset action.

## 4. Next Actions
1. Execute all non-harness scenarios (`FP-A`, `FP-B`, `FP-D`) for one baseline build/channel.
2. Execute comparable Lean-runtime observations for relevant scenarios.
3. Populate Lean-vs-Excel deviation ledger with explicit relation classes.
4. Generate first observed results matrix and mismatch log.
5. Define minimal interop harness design for `FP-C` scenarios.
6. Propose promotion candidates for `EMP-*` where behavior is stable and high-impact.

## 5. Gate Tracking
### G1 - Scenario Closure
1. Status: `closed` (seed manifest exists and is reproducible).

### G2 - Observation Closure
1. Status: `open`.
2. Blockers: Excel and Lean comparative runs not yet captured.

### G3 - Characterization Closure
1. Status: `open`.
2. Blockers: normalized policy map and deviation ledger not yet assembled.

### G4 - Promotion Closure
1. Status: `open`.
2. Blockers: no promoted findings yet.

## 6. Tooling Integration Artifacts
1. Probe tooling root:
   - `../../tools/fp-probe/`
   - Rust runner manifest: `../../tools/fp-probe/fp_probe_runner/Cargo.toml`
2. XLL harness contract:
   - `../../tools/fp-probe/xll/FpEdgeHarnessContract.md`
3. Results template:
   - `../../tools/fp-probe/results/FLOATING_POINT_RESULTS_TEMPLATE.csv`
4. Runtime requirements:
   - `FLOATING_POINT_PROBE_RUNTIME_REQUIREMENTS.md`
5. Deviation ledger:
   - `FLOATING_POINT_LEAN_EXCEL_DEVIATION_LEDGER.csv`
