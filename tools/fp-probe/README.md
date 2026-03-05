# FP Probe Tooling

This folder contains the initial scaffolding for W2 floating-point empirical execution.

## Layout
1. `fp_probe_runner/`:
   - Rust CLI that validates a scenario manifest and writes queued execution rows.
2. `run-fp-probe.ps1`:
   - PowerShell wrapper around the runner.
3. `run-fp-excel-baseline.ps1`:
   - PowerShell COM runner for first local Excel baseline observations (`FP-A`, `FP-B` by default).
4. `xll/FpEdgeHarnessContract.md`:
   - XLL function contract for special floating-point injection (`FP-C` lane).
5. `results/FLOATING_POINT_RESULTS_TEMPLATE.csv`:
   - output schema template for result capture.

## Runner CLI
```
cargo run --manifest-path tools/fp-probe/fp_probe_runner/Cargo.toml -- --manifest <csv> --out <csv> [--mode dry-run|prepare]
```

Behavior:
1. Validates required manifest columns:
   - `scenario_id`
   - `lane`
   - `objective`
   - `status`
2. Emits one output row per scenario with queued status.
3. Uses `execution_status=queued` and `observed_class=pending_observation` for both modes.

## PowerShell Wrapper
```
powershell -File tools/fp-probe/run-fp-probe.ps1 -Manifest <csv> -Out <csv> [-Mode dry-run|prepare]
```

## Current Scope
1. Rust runner prepares deterministic schema-compatible output files from manifest rows.
2. Excel baseline script executes first local observation pass for formula/reference lanes.
3. XLL-driven special-value ingress (`FP-C`) remains pending harness implementation.

First Excel baseline script:
```powershell
powershell -File tools/fp-probe/run-fp-excel-baseline.ps1 -Manifest docs/function-lane/FLOATING_POINT_SCENARIO_MANIFEST_SEED.csv -Out .tmp/fp-results-excel-baseline.csv
```
