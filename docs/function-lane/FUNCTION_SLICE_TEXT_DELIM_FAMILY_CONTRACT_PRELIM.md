# Function Slice Contract (Preliminary) - Text Delimiter Family

Status: `provisional`
Workset: `W24`
Primary Functions: `TEXTAFTER`, `TEXTBEFORE`

## 1. Scope
1. close the admitted current-baseline scalar slice for `TEXTAFTER` and `TEXTBEFORE`,
2. bind the already-integrated runtime and Lean substrate to a replayable native worksheet packet,
3. make the bounded delimiter, case-mode, and fallback behavior explicit.

## 2. Admitted Current-Baseline Slice
1. scalar text input and delimiter,
2. `instance_num` positive or negative, but not zero,
3. `match_mode` limited to `0` or `1`,
4. `match_end` limited to `0` or `1`,
5. documented empty-delimiter polarity,
6. explicit `if_not_found` fallback in the sixth argument position,
7. current observed ASCII-only case-folding in `match_mode = 1`.

## 3. Semantics
1. `TEXTAFTER` returns the text after the selected delimiter occurrence.
2. `TEXTBEFORE` returns the text before the selected delimiter occurrence.
3. Positive `instance_num` counts from the start; negative `instance_num` counts from the end.
4. Empty-delimiter handling follows Excel's directional polarity:
   - `TEXTAFTER(text, "", positive)` returns the whole text,
   - `TEXTAFTER(text, "", negative)` returns `""`,
   - `TEXTBEFORE(text, "", positive)` returns `""`,
   - `TEXTBEFORE(text, "", negative)` returns the whole text.
5. `match_end = 1` adds a synthetic terminal delimiter when the delimiter is otherwise absent.
6. No-match without `if_not_found` returns `#N/A`.
7. Invalid `instance_num`, `match_mode`, and `match_end` return `#VALUE!`.

## 4. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety: `safe_pure`
5. arg_preparation_profile: `values_only_pre_adapter`
6. coercion_lift_profile: `custom`
7. kernel_signature_class: `text_to_text`
8. fec_dependency_profile: `none`
9. surface_fec_dependency_profile: `ref_only`

## 5. Evidence Basis
1. Rust runtime kernel and unit tests in `crates/oxfunc_core/src/functions/text_delim_family.rs`
2. Lean metadata/binding in `formal/lean/OxFunc/Functions/TextDelimFamily.lean`
3. Native worksheet packet in `docs/function-lane/W24_BATCH03_TEXT_DELIM_SCENARIO_MANIFEST_SEED.csv`
4. Runtime harness in `tools/w24-probe/run-w24-batch03-text-delim-baseline.ps1`
5. Packet execution record in `docs/function-lane/W24_BATCH03_TEXT_DELIM_EXECUTION_RECORD.md`

## 6. Scope Boundary
1. The admitted slice is bounded to the currently pinned scalar cases.
2. No claim is made here about broader locale-sensitive collation or Unicode case-fold parity beyond the observed ASCII baseline.
