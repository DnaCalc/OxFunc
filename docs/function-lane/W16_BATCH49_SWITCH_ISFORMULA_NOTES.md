# W16 Batch 49 - SWITCH / ISFORMULA Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH49-SWITCH-ISFORMULA-20260316`

## Scope
1. `SWITCH`
2. `ISFORMULA`

## Pinned Semantics
1. `SWITCH` matches text case-insensitively and returns the first matching branch.
2. `SWITCH` does not match numeric text against numeric values.
3. `SWITCH` returns the optional default when no case matches, otherwise `#N/A`.
4. `SWITCH` is lazy over result branches; only the selected result is evaluated.
5. `ISFORMULA` returns `TRUE` or `FALSE` only for a real reference operand.
6. `ISFORMULA` returns `#VALUE!` for scalar current-surface operands such as `1`.

## Current Batch Position
1. This branch-only batch is intentionally isolated to the three owned files and does not alter shared dispatch or export surfaces.
2. `SWITCH` is modeled as a pure adapter-visible function family member.
3. `ISFORMULA` is modeled as a typed host-query over a reference operand.
4. Seam limit: without a promoted formula-artifact/query seam in shared OxFunc surfaces, `ISFORMULA` here is scoped to current surface semantics and does not claim deeper formula-identity inspection inside OxFunc itself.
5. Successor ownership split:
   - `SWITCH` remains an ordinary semantic-hardening residual under `W17`,
   - `ISFORMULA` is extracted to `W023` because it is not honestly a value-only pure function on the current boundary.

## Local Verification
1. Standalone Rust tests in [misc_switch_info_family.rs](/C:/Work/DnaCalc/OxFunc/crates/oxfunc_core/src/functions/misc_switch_info_family.rs) cover first-match, default, `#N/A`, type-sensitive comparison, laziness, and reference-only `ISFORMULA` behavior.
2. The Lean note in [MiscSwitchInfoFamily.lean](/C:/Work/DnaCalc/OxFunc/formal/lean/OxFunc/Functions/MiscSwitchInfoFamily.lean) records the intended metadata split: pure `SWITCH` versus workbook-state `ISFORMULA`.
