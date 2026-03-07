# Function Lane Integrity Checks

Purpose:
1. verify that correlation ledger links resolve to concrete artifacts,
2. prevent drift between contract, formal, runtime, tests, and evidence references.

## Script
1. `run-correlation-integrity-check.ps1`

## Command
```powershell
powershell -File tools/function-lane-check/run-correlation-integrity-check.ps1
```

## Checks
1. Lean module path exists.
2. Lean theorem/lemma/def ids exist in the module.
3. Rust module path exists.
4. Rust test function ids exist in the module.
5. Evidence ids in ledger exist in registry (except `TBD-*` placeholders).
6. Dual version-scope fields are non-empty.
