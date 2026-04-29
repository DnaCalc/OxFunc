# WORKSET - Function Array Support Systematic Sweep (W090)

## 1. Purpose
Own the systematic, bounded family-by-family review of array-valued
scalar-parameter behavior across supported OxFunc functions, continuing the
seed evidence from `W080` without claiming that every function can be swept in
one pass.

## 2. Why This Packet Exists
`W080` found and repaired several text-family array-spill gaps, and `W079`
showed the same class of risk in lookup selection arguments. Those findings are
enough to justify a broad review program, but the supported function invocation
space is too large for each passing case to become a prose artifact. `W090`
therefore owns compact inventory, sampling, replay, mismatch promotion, and
coverage telemetry for the broader array-support sweep.

## 3. Provenance
1. `docs/worksets/W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md`
2. `docs/worksets/W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md`
3. `docs/bugs/streams/BUG-FUNC-007_text_slice_array_position_and_count_spill_gap.md`
4. `docs/bugs/streams/BUG-FUNC-008_text_scalar_and_delimiter_array_support_gap.md`
5. `docs/bugs/streams/BUG-FUNC-016_text_search_replace_array_support_gap.md`
6. `docs/function-lane/W66_SCENARIO_MANIFEST_SEED.csv`
7. `docs/worksets/W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md`
8. `docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md`

## 4. Scope
In scope:
1. build an inventory of supported functions and arguments where metadata,
   coercion rules, source patterns, or adjacent Excel evidence suggest
   scalar-only behavior might actually lift or spill arrays,
2. classify risk by family and argument role before selecting batches,
3. define a compact replay matrix per family covering one-array arguments,
   multiple-array and broadcast contrasts, reference-vs-array inputs, blanks,
   omitted arguments, errors, optional/fallback positions, and spill shape,
4. run bounded batches through local OxFunc and Excel comparison when a batch is
   explicitly selected,
5. promote confirmed divergences into `BUG-FUNC-*` streams or narrower repair
   beads/worksets,
6. keep passing cases as aggregate coverage telemetry and representative
   anchors rather than heavy per-case prose,
7. reconcile `W051`, contracts, execution records, and bug streams when a
   confirmed issue changes current-surface truth.

Out of scope:
1. completing every supported function in a single pass,
2. broad random smart-fuzzer execution without the `W088` / `W089` artifact and
   batching discipline,
3. locale and alternate Excel-version sweeps,
4. repairing every discovered bug inside `W090` itself when a narrower bead or
   workset is the cleaner owner,
5. treating high-volume passing rows as completion evidence beyond compact
   coverage telemetry.

## 5. Initial Epic Lanes
1. supported-function and argument-role inventory
2. static risk classification from metadata, coercion declarations, and source
   implementation patterns
3. Excel replay matrix design and batch sizing
4. first post-`W080` family-batch selection
5. local-vs-Excel comparison and mismatch minimization
6. bug promotion and truth-surface reconciliation
7. compact roadmap/highlights trace for explored array-support regions

## 6. Closure Condition
`W090` may close only when:
1. a stable candidate inventory exists for the supported current-version
   function surface,
2. at least one non-text successor family tranche has been executed or
   explicitly deferred with a replacement tranche and rationale,
3. confirmed mismatches from executed tranches have been minimized and promoted
   to ordinary bug streams or narrower worksets,
4. pass-heavy exploration is summarized as aggregate coverage telemetry rather
   than per-case documentation,
5. `W051`, contract, workset, and bug-stream truth surfaces no longer overclaim
   the array-support regions actually examined,
6. remaining unswept regions have an explicit next-owner or next-tranche plan,
7. no claim is made that the full supported array-support surface has been
   reviewed unless the telemetry proves that exact scope.

## 7. Current Reading
1. execution_state: `planned`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - supported-function and argument-role inventory
   - first post-`W080` sweep-tranche selection
   - compact replay matrix and telemetry design
   - local-vs-Excel comparison execution for selected tranches
   - mismatch promotion and truth-surface reconciliation
