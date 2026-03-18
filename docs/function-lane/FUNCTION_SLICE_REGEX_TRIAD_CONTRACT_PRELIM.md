# Function Slice Contract (Preliminary) - Regex Triad

Status: `provisional`
Workset: `W24`
Primary Functions: `REGEXEXTRACT`, `REGEXREPLACE`, `REGEXTEST`

## 1. Scope
1. close the admitted current-baseline pure regex trio independently from the mixed `NUMBERVALUE` / `TRANSLATE` family note,
2. bind the integrated runtime and Lean substrate to a replayable native worksheet packet,
3. preserve the current bounded local regex language without overclaiming the full Excel regex surface.

## 2. Admitted Current-Baseline Slice
1. deterministic local regex subset only:
   - literals,
   - `.`,
   - character classes,
   - ASCII-aware ranges,
   - `\d`, `\w`, `\s`,
   - postfix quantifiers `*`, `+`, `?`
2. `REGEXEXTRACT`
   - first-match extraction only,
   - no-match returns `#N/A`,
   - bounded to the scalar text result surface.
3. `REGEXREPLACE`
   - literal replacement text only,
   - occurrence omitted means the first or only match under the admitted current packet,
   - positive occurrence `n` replaces the `n`th match.
4. `REGEXTEST`
   - logical yes/no result only,
   - `case_sensitivity = 0` or omitted means case-sensitive,
   - `case_sensitivity = 1` means current ASCII-only case-insensitive matching.

## 3. Explicitly Out Of Slice
1. grouping, alternation, anchors, lookarounds, backreferences, and capture-group extraction.
2. Unicode-aware case folding beyond the current ASCII baseline.
3. array-return modes and richer extraction surfaces.
4. the mixed-family `NUMBERVALUE` locale-default lanes and `TRANSLATE` provider lanes.

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
1. Rust runtime kernel and unit tests in `crates/oxfunc_core/src/functions/number_regex_translate_family.rs`
2. Lean metadata/binding in `formal/lean/OxFunc/Functions/NumberRegexTranslateFamily.lean`
3. Native worksheet packet in `docs/function-lane/W24_BATCH10_REGEX_SCENARIO_MANIFEST_SEED.csv`
4. Runtime harness in `tools/w24-probe/run-w24-batch10-regex-baseline.ps1`
5. Packet execution record in `docs/function-lane/W24_BATCH10_REGEX_EXECUTION_RECORD.md`

## 6. Scope Boundary
1. The closure is bounded to the admitted current-baseline regex trio above.
2. `NUMBERVALUE` and `TRANSLATE` are not part of this closure packet.
3. Broader Excel regex semantics remain separate future work rather than implied support.
