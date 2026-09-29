# Function Slice - Reference Metadata And Formula Visibility Contract (Prelim)

Status: `provisional`
Workset: `W040`

## 1. Scope
This slice covers the admitted current-baseline packet for:
1. `ADDRESS`
2. `AREAS`
3. `FORMULATEXT`
4. `SHEET`
5. `SHEETS`

## 2. Why These Functions Are Grouped
1. They expose reference identity or workbook/grid metadata rather than only computing over scalar values.
2. They pressure the seam between prepared references and host-supplied sheet/formula metadata.
3. They are narrower than the broader `W023` host/database packet but still not purely scalar.

## 3. Admitted Current-Baseline Slice
1. `ADDRESS`
   - row/column to text address conversion,
   - absolute-mode switching,
   - A1 vs R1C1 text mode,
   - explicit `sheet_text`, including quoted sheet names with spaces.
2. `AREAS`
   - single-area reference cardinality,
   - first-class same-sheet multi-area reference cardinality.
3. `FORMULATEXT`
   - formula visibility for a referenced formula cell,
   - `#N/A` when the referenced cell stores a plain value rather than a formula.
4. `SHEET`
   - omitted-argument current-sheet number,
   - sheet number by reference,
   - sheet number by sheet-name text,
   - `#N/A` for missing sheet-name text.
5. `SHEETS`
   - omitted-argument workbook sheet count,
   - single-sheet reference count,
   - 3D sheet-span count.

## 4. Semantic Class Axes
1. `ADDRESS`
   - `determinism_class`: `deterministic`
   - `volatility_class`: `nonvolatile`
   - `host_interaction_class`: `none`
   - `thread_safety_class`: `safe_pure`
   - `arg_preparation_profile`: `values_only_pre_adapter`
   - `surface_fec_dependency_profile`: `none`
2. `AREAS`
   - `determinism_class`: `deterministic`
   - `volatility_class`: `nonvolatile`
   - `host_interaction_class`: `none`
   - `thread_safety_class`: `safe_pure`
   - `arg_preparation_profile`: `refs_visible_in_adapter`
   - `surface_fec_dependency_profile`: `ref_only`
3. `FORMULATEXT`
   - `determinism_class`: `deterministic`
   - `volatility_class`: `nonvolatile`
   - `host_interaction_class`: `workbook_state`
   - `thread_safety_class`: `host_serialized`
   - `arg_preparation_profile`: `refs_visible_in_adapter`
   - `surface_fec_dependency_profile`: `ref_only`
4. `SHEET`
   - `determinism_class`: `deterministic`
   - `volatility_class`: `nonvolatile`
   - `host_interaction_class`: `workbook_state`
   - `thread_safety_class`: `host_serialized`
   - `arg_preparation_profile`: `refs_visible_in_adapter`
   - `surface_fec_dependency_profile`: `composite`
5. `SHEETS`
   - `determinism_class`: `deterministic`
   - `volatility_class`: `nonvolatile`
   - `host_interaction_class`: `workbook_state`
   - `thread_safety_class`: `host_serialized`
   - `arg_preparation_profile`: `refs_visible_in_adapter`
   - `surface_fec_dependency_profile`: `composite`

## 5. Seam Reading
1. `ADDRESS`
   - pure text/reference rendering once the scalar args are admitted,
   - does not require workbook topology when `sheet_text` is supplied explicitly.
2. `AREAS`
   - requires the prepared reference to preserve first-class multi-area shape,
   - does not need additional host metadata beyond the reference structure itself.
3. `FORMULATEXT`
   - requires host/grid access to the stored formula text for the referenced cell,
   - cannot be reduced to value-only evaluation.
4. `SHEET`
   - requires sheet identity/order metadata,
   - omitted form also depends on the caller sheet.
5. `SHEETS`
   - requires workbook sheet-topology metadata,
   - 3D reference counting depends on sheet-span identity rather than only value evaluation.

## 6. First-Pass OxFml <-> OxFunc Interface Reading
1. OxFml should preserve structured references and sheet-span identity for `AREAS`, `SHEET`, and `SHEETS`.
2. OxFml or the host-facing layer must provide stored-formula text access for `FORMULATEXT`.
3. OxFunc can own:
   - admitted scalar argument validation,
   - address rendering policy,
   - area counting over preserved multi-area references,
   - result projection once host metadata is supplied.
4. OxFunc should not own:
   - workbook sheet-order discovery,
   - current-sheet identity discovery,
   - stored formula retrieval from the grid.

## 7. Exact Current OxFunc Callback Surface
The current first-pass OxFunc-side callback/context surface is:
1. `query_formula_text(reference: ReferenceLike) -> EvalValue`
   - used by `FORMULATEXT`,
   - host responsibility: return stored formula text for the referenced cell, or the corresponding worksheet error/result classification.
2. `query_sheet_index(spec: SheetIdentitySpec) -> EvalValue`
   - used by `SHEET`,
   - where `SheetIdentitySpec` is one of:
     - `CurrentSheet`
     - `Reference(reference)`
     - `SheetNameText(text)`
3. `query_sheet_count(spec: SheetCountSpec) -> EvalValue`
   - used by `SHEETS`,
   - where `SheetCountSpec` is one of:
     - `Workbook`
     - `Reference(reference)`

## 8. Current Ownership Split
1. `ADDRESS`
   - OxFunc-owned once scalar args are admitted.
   - No host callback required.
2. `AREAS`
   - OxFunc-owned once the reference preserves multi-area structure.
   - No extra host callback required beyond ref-visible argument preparation.
3. `FORMULATEXT`
   - OxFunc owns ref admission and worksheet result projection.
   - Host/OxFml side owns stored-formula retrieval.
4. `SHEET`
   - OxFunc owns argument admission and selection of `CurrentSheet` vs `Reference` vs `SheetNameText`.
   - Host/OxFml side owns sheet-order/topology truth and current-sheet identity.
5. `SHEETS`
   - OxFunc owns argument admission and selection of `Workbook` vs `Reference`.
   - Host/OxFml side owns workbook sheet-count and 3D span-count truth.

## 9. Evidence Posture
1. `spec_anchor`:
   - `FDEF-061` in `EXCEL_FUNCTION_DEFINITION_PRELIM_CONFORMANCE.csv`
2. `empirical_anchor`:
   - `W40-REFMETA-BL-20260321`
3. exact current OxFunc-side implementation artifacts:
   - `crates/oxfunc_core/src/functions/reference_metadata_family.rs`
   - `crates/oxfunc_core/src/host_info.rs`
   - `formal/lean/OxFunc/Functions/ReferenceMetadataFamily.lean`
   - `formal/lean/OxFunc/HostInfoSeam.lean`

## 10. Evidence Target
Initial packet evidence is the native Excel baseline in:
1. `docs/function-lane/W40_SCENARIO_MANIFEST_SEED.csv`
2. `tools/w40-probe/run-w40-reference-metadata-baseline.ps1`
3. `.tmp/w40-reference-metadata-results.csv`

## 11. Artifact Bindings
1. workset: `docs/worksets/W040_REFERENCE_METADATA_AND_FORMULA_VISIBILITY_FUNCTIONS.md`
2. execution record: `docs/function-lane/W40_EXECUTION_RECORD.md`
3. scope reconciliation: `docs/function-lane/W40_SCOPE_RECONCILIATION.csv`
4. Rust: `crates/oxfunc_core/src/functions/reference_metadata_family.rs`; `crates/oxfunc_core/src/host_info.rs`
5. Lean: `formal/lean/OxFunc/Functions/ReferenceMetadataFamily.lean`; `formal/lean/OxFunc/HostInfoSeam.lean`


## 12. W111 ADDRESS empirical refinement (2026-09-29)

Status remains `in_progress`: `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Open lanes: implicit numeric-text grammar, locale/workbook-date/current-year
preparation context, cross-repository seam assessment, and final claim audit.

The [retained ADDRESS evidence](evidence/w111-broad-20260929/address/README.md)
refines this packet on Excel 16.0 build 20430, Workbook Compatibility Version 2.
Numeric coordinates/mode use the evidenced tolerant-floor conversion; signed
relative R1C1 coordinates have strict worksheet-size bounds and zero suppresses
the bracketed offset. Scalar errors coerce left to right before numeric domain
validation. Omitted and blank selectors differ, and omitted first/second
coordinates have distinct behavior. All five arguments broadcast.

The text renderer preserves raw UTF-16 units. Sheet names use position-sensitive
character classes, reference-token ambiguity rules, apostrophe doubling and
observed original/escaped length limits. A bounded append model governs long
prefixes and integer fragments. Numeric sheet values use the evidenced
15-significant-digit decimal primitive followed by ADDRESS-specific width and
scientific rounding rules. These behaviors are exercised through production
dispatch, including 5,330 admitted independent heldout cases after candidate
freeze. The Lean rendering substrate and explicit classifier adapter binding
are aligned with this admitted slice.

The earlier host-independent ADDRESS description in sections 4–8 is valid for
already prepared numeric/logical arguments and sheet text, but it does not yet
account for all text-to-number preparation. Exact-ingress observations show
locale-specific grouping/currency/digit/date/time parsing, and dates with no
year depend on the current host year. That known seam gap prevents a full
function claim. Runtime metadata and OxFml contracts must only change after
cross-repository impact assessment; this note does not silently change them.
