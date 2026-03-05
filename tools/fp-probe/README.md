# FP Probe Tooling

This folder contains the initial scaffolding for W2 floating-point empirical execution.

## Layout
1. `fp_probe_runner/`:
   - Rust CLI that validates a scenario manifest and writes queued execution rows.
2. `run-fp-probe.ps1`:
   - PowerShell wrapper around the runner.
3. `xll/FpEdgeHarnessContract.md`:
   - XLL function contract for special floating-point injection (`FP-C` lane).
4. `results/FLOATING_POINT_RESULTS_TEMPLATE.csv`:
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
This scaffold does not automate Excel yet. It prepares deterministic execution records and schema-compatible output files.
