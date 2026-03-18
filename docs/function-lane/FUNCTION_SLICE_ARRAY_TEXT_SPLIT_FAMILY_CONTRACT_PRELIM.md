# Function Slice Contract (Preliminary) - Array Text Split Family

Status: `provisional`
Workset: `W24`
Primary Functions: `ARRAYTOTEXT`, `TEXTSPLIT`

## 1. Scope
1. close the admitted current-baseline slice for `ARRAYTOTEXT` and `TEXTSPLIT`,
2. bind the integrated runtime and Lean substrate to a replayable native worksheet packet,
3. keep spill-publication claims narrower than the core array semantics when the packet uses scalarized `ARRAYTOTEXT(TEXTSPLIT(...),1)` witnesses.

## 2. Admitted Current-Baseline Slice
1. `ARRAYTOTEXT`
   - scalar inputs promoted to `1x1` arrays,
   - concise format `0`,
   - strict format `1`,
   - row-major concise rendering,
   - strict rendering with braces, comma/semicolon separators, quoted text cells, and unquoted logical/error/number cells.
2. `TEXTSPLIT`
   - scalar text input,
   - column delimiter only or column plus row delimiter,
   - multi-delimiter arrays,
   - `ignore_empty`,
   - ASCII-only case-insensitive `match_mode = 1`,
   - default `#N/A` padding and explicit `pad_with`.

## 3. Semantics
1. `ARRAYTOTEXT(...,0)` renders a concise row-major string.
2. `ARRAYTOTEXT(...,1)` renders a strict array literal-like string.
3. `TEXTSPLIT` returns an array payload whose witness is captured in this packet through `ARRAYTOTEXT(TEXTSPLIT(...),1)`.
4. Default `TEXTSPLIT` padding is `#N/A`.
5. Explicit `pad_with` replaces the default padding cells.

## 4. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety: `safe_pure`
5. arg_preparation_profile: `values_only_pre_adapter`
6. coercion_lift_profile: `custom`
7. fec_dependency_profile: `none`
8. surface_fec_dependency_profile: `ref_only`

## 5. Evidence Basis
1. Rust runtime kernel and unit tests in `crates/oxfunc_core/src/functions/array_text_split_family.rs`
2. Lean metadata/binding in `formal/lean/OxFunc/Functions/ArrayTextSplitFamily.lean`
3. Native worksheet packet in `docs/function-lane/W24_BATCH04_ARRAY_TEXT_SPLIT_SCENARIO_MANIFEST_SEED.csv`
4. Runtime harness in `tools/w24-probe/run-w24-batch04-array-text-split-baseline.ps1`
5. Packet execution record in `docs/function-lane/W24_BATCH04_ARRAY_TEXT_SPLIT_EXECUTION_RECORD.md`

## 6. Scope Boundary
1. The packet proves the admitted core array semantics for the seeded baseline.
2. It does not claim broader spill-host publication closure beyond the scalar witness path used here.
3. It does not claim deeper locale-sensitive or Unicode collation closure for delimiter matching beyond the observed ASCII baseline.
