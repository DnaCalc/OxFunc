# W48 Execution Record - Return Surface And Publication Hint Freeze

Status: `complete`
Workset: `W048`

## 1. Purpose
Freeze the first shared return-surface split for the already-covered OxFunc scope:
1. ordinary value
2. `ValueWithPresentation`
3. typed host/provider outcome projection

## 2. Packet Outputs
Artifacts produced or updated in this packet:
1. `docs/HISTORY.md`
2. `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md`
3. `docs/function-lane/W48_RETURN_SURFACE_CLASS_MAP.csv`
4. `docs/function-lane/W48_EXECUTION_RECORD.md`
5. `docs/function-lane/W48_OXFML_CONSUMER_RECONCILIATION.md`
6. `docs/function-lane/W48_CONSUMER_MISMATCH_LEDGER.csv`
7. `docs/upstream/NOTES_FOR_OXFML.md`

## 3. Current Result
The current shared split is now explicit:
1. ordinary value remains the default function return class,
2. `ValueWithPresentation` is the current publication-aware wrapper,
3. typed host/provider outcome projection remains a typed callback-boundary pattern rather than a new general published-value carrier.

## 4. Main Findings
1. `NOW`, `TODAY`, and `HYPERLINK` are enough to justify a real shared `ValueWithPresentation` class now; it is no longer only a local convenience wrapper.
2. `TRANSLATE` and `RTD` show the current third class clearly: typed upstream outcome families matter semantically even when the final worksheet-visible result is an ordinary value or worksheet error.
3. `IMAGE` should stay explicit as a sibling rich-value/publication pressure, not be misclassified as either ordinary value or presentation-only output.

## 5. Verification Basis
This packet freezes a shared reading from already exercised function packets and value-model artifacts rather than creating new function semantics.

Primary reviewed artifacts:
1. `crates/oxfunc_core/src/value.rs`
2. `formal/lean/OxFunc/ValueUniverse.lean`
3. `docs/function-lane/VALUE_UNIVERSE_PRELIM_SPEC.md`
4. `docs/function-lane/FUNCTION_SLICE_NOW_CONTRACT_PRELIM.md`
5. `docs/function-lane/FUNCTION_SLICE_TODAY_CONTRACT_PRELIM.md`
6. `docs/function-lane/FUNCTION_SLICE_HYPERLINK_IMAGE_VALUE_MODEL_PRELIM.md`
7. `docs/function-lane/FUNCTION_SLICE_TRANSLATE_PROVIDER_LANGUAGE_CONTRACT_PRELIM.md`
8. `docs/function-lane/FUNCTION_SLICE_RTD_CONTRACT_PRELIM.md`

## 6. Completeness Axes
1. execution_state: `complete`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - later witness-bearing runtime attachment remains follow-on work under `W069`, not an open current-phase freeze gap
