# WORKSET - Deferred Provider Language Capability Baseline (W36)

## 1. Purpose
Own the extracted built-in language-service seam whose current-boundary behavior depends on an external language provider rather than a pure local kernel.

`TRANSLATE` is conceptually part of the broader external-services family, but it was extracted from `W041` because it was small enough to reconcile cleanly as its own focused seam packet and is not add-in-owned like `EUROCONVERT`.

This packet now remains the provenance/evidence owner for that narrow seam baseline only. Active deferred-current-version tracking for `TRANSLATE` sits in `W050`.

## 2. Provenance
Opened by `W31` scope reconciliation on `2026-03-19`.

Source artifacts:
1. `docs/function-lane/W31_SCOPE_RECONCILIATION.csv`
2. `docs/function-lane/W26_EXECUTION_RECORD.md`
3. `CURRENT_BLOCKERS.md`

## 3. Scope
Machine-readable inventory:
1. `docs/function-lane/W36_DEFERRED_PROVIDER_LANGUAGE_CAPABILITY_INVENTORY.csv`

Current total:
1. `1` deferred function.

Members:
1. `TRANSLATE`

Classification note:
1. `W036` no longer acts as the active current-version closure tracker for `TRANSLATE`.
2. `W050` now owns the question of whether `TRANSLATE` is deferred from the current OxFunc completion target.

## 4. Executed Work
1. pinned the native current-host provider-language baseline from `W26`,
2. defined a typed translate-provider request/result seam,
3. kept same-language passthrough local to OxFunc,
4. kept provider-state outcomes separate from library-context availability truth.

## 5. Status
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none in declared `W036` scope

Status note:
1. `complete` here means the declared `W036` seam-baseline packet is closed.
2. It does not mean `TRANSLATE` is counted as current-target complete; that classification now lives in `W050`.
