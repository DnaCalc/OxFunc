# WORKSET - Semantic Witness Snapshot V2 Plan (W69)

## 1. Purpose
Turn the current OxFunc library-context snapshot from a catalog/profile artifact into a semantic witness artifact that downstream systems can query for help, signature, diagnostics, replay correlation, and formal-evidence provenance.

This packet exists to define the next high-leverage OxFunc product step after non-deferred current-version closure:
1. keep the current `V1` snapshot as the pinned catalog/profile export,
2. add a `V2` semantic witness layer that exposes structured semantic evidence rather than only row metadata,
3. align that `V2` layer with the preferred runtime `LibraryContextProvider` / immutable `LibraryContextSnapshot` direction.

## 2. Why This Packet Exists
Current OxFunc now has:
1. a published `V1` snapshot with `534` rows,
2. `517` supported rows,
3. `0` active non-deferred backlog rows,
4. a current shared freeze reading for the seam-relevant non-deferred surface,
5. a parked non-deferred Rust/runtime/formal surface.

What OxFunc still does not expose is one machine-readable downstream artifact that answers:
1. what a function means in practical semantic terms,
2. how to explain it to a user,
3. which exact replay/evidence rows justify the current support claim,
4. which Lean artifact is the primary formal home,
5. what admitted-slice boundaries and orthogonal validation limits remain.

`V1` tells a consumer what a row is.
`V2` should tell a consumer why the row is trustworthy and how to use it.

## 3. Product Thesis
The smart next move is to make OxFunc the authoritative executable semantic oracle for Excel functions rather than only an implementation repo.

`Semantic Witness Snapshot V2` means:
1. a downstream consumer can ask OxFunc not only for catalog identity and arity,
2. but also for structured help, signature, argument, evidence, and formalization payloads,
3. all aligned to the same stable `surface_stable_id`,
4. with explicit snapshot generation and provenance.

This is accretive because every already-closed function becomes more valuable without changing the Rust evaluator itself.

## 4. V1 vs V2
Current `V1` snapshot provides:
1. stable ids,
2. canonical names,
3. arity bounds,
4. category,
5. coarse semantic and seam profile fields,
6. snapshot provenance.

`V1` does not yet provide:
1. structured help prose,
2. argument names and descriptions,
3. formatted signatures,
4. semantic witness examples,
5. explicit evidence-id links,
6. explicit formal artifact links,
7. admitted-slice boundary summaries,
8. orthogonal validation status.

`V2` should add those missing layers without regressing the stable `V1` identity/model surface.

## 5. Concrete Example - VLOOKUP
Current `V1` row for `VLOOKUP` can tell a consumer:
1. `surface_stable_id = FUNC.VLOOKUP`
2. `arity = 3..4`
3. `category = Lookup and reference functions`
4. `kernel_signature_class = LookupMatch`
5. `metadata_status = function_meta_curated`

That is useful, but it still leaves downstream consumers to invent or hardcode:
1. signature text,
2. argument labels,
3. help summary,
4. semantic examples,
5. evidence references,
6. formal references.

`V2` should let OxFunc publish one row-shaped witness package such as:

```text
SemanticWitnessEntry:
  surface_stable_id: "FUNC.VLOOKUP"
  canonical_surface_name: "VLOOKUP"
  signature_display: "VLOOKUP(lookup_value, table_array, col_index_num, [range_lookup])"
  help_summary: "Looks for a value in the first column of a table and returns a value from a specified column."
  arg_specs:
    - lookup_value
    - table_array
    - col_index_num
    - range_lookup
  semantic_modes:
    - exact_match
    - approximate_match_sorted
    - wildcard_text_lookup
  witness_examples:
    - exact match success
    - exact match not found -> #N/A
    - approximate match picks last-not-greater row
    - invalid column index -> error
  evidence_refs:
    - W68-LOOKUP-LOGICAL-RESIDUALS-BL-20260401
  formal_refs:
    - formal/lean/OxFunc/Functions/VhlookupFamily.lean
  contract_refs:
    - docs/function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md
  admitted_slice_note:
    - current-baseline exact/approximate/wildcard/index-validation lanes are closed
  orthogonal_validation_status:
    - locale/version sweeps pending
```

The gain is that OneCalc or any future consumer would no longer need three separate private truths:
1. hardcoded help text,
2. separate test/evidence lookup tables,
3. separate formalization or trust ledgers.

## 6. Proposed V2 Payload Families
`V2` should add four new payload families.

### 6.1 Help Payload
Per row:
1. `help_summary`
2. `help_detail`
3. `category_display`
4. `behavior_notes`

### 6.2 Signature Payload
Per row:
1. `signature_display`
2. `arg_specs`
3. `arg_required`
4. `arg_type_hint`
5. `arg_behavior_note`

### 6.3 Semantic Witness Payload
Per row:
1. `semantic_modes`
2. `witness_examples`
3. `admitted_slice_note`
4. `known_orthogonal_validation_lanes`
5. `current_support_basis`

### 6.4 Provenance And Trust Payload
Per row:
1. `evidence_refs`
2. `execution_record_refs`
3. `formal_refs`
4. `contract_refs`
5. `snapshot_generation`
6. `source_commit_*`

## 7. Runtime Shape Direction
`V2` should not be designed as a larger CSV first and only later mapped into runtime shape.

Preferred rule:
1. design `V2` as a `LibraryContextEntry` runtime witness shape first,
2. define CSV/JSON export projections from that runtime shape,
3. keep `V1` CSV as the pinned interchange artifact,
4. add `V2` as either:
   - a second export artifact, or
   - a runtime-provider-backed export family with a deterministic serialized form.

That matches the direction already recorded in:
1. `W044`
2. `W049`
3. `OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md`

## 8. Design Rules
1. `V2` must be keyed by the existing `surface_stable_id`.
2. `V2` must not invent a second identity system.
3. `V2` must separate stable facts from prose/enrichment.
4. `V2` help text must be OxFunc-owned, not downstream-invented.
5. `V2` evidence refs must point to current repo artifacts, not only free text.
6. `V2` must not weaken the current `V1` stability guarantees.
7. `V2` must preserve the distinction between:
   - support status,
   - admitted-slice evidence,
   - orthogonal validation phases.

## 9. Initial Deliverables
This packet should aim for:
1. one `SemanticWitnessEntry` schema note,
2. one mapping from `V1` fields into `V2`,
3. one first witness artifact for a bounded seed set,
4. one downstream-consumer reading guide,
5. one runtime-model alignment note showing how the witness payload attaches to `LibraryContextSnapshot`.

## 9A. Full-Coverage Target
`W069` now owns the full supported-surface witness rollout target for the parked
non-deferred baseline.

Full-coverage target:
1. every currently supported non-deferred `V1` row should eventually have a
   `V2` witness row keyed by the same `surface_stable_id`,
2. the target surface is the current `517` supported rows from the parked
   baseline,
3. the `17` deferred rows tracked in `W050` remain out of scope until
   intentionally reopened,
4. rows whose witness rollout depends on retained live seam authorities such as
   `W043` or `W049` remain in-scope for `W069`, but their final closure may be
   dependency-gated by those retained worksets.

Current coverage accounting:
1. current witness-covered rows: `10`
2. current remaining supported rows without `V2` witness coverage: `507`
3. current covered set:
   - `FUNC.GROUPBY`
   - `FUNC.HLOOKUP`
   - `FUNC.HYPERLINK`
   - `FUNC.IF`
   - `FUNC.IMAGE`
   - `FUNC.LET`
   - `FUNC.OP_IMPLICIT_INTERSECTION`
   - `FUNC.SUM`
   - `FUNC.VLOOKUP`
   - `FUNC.XLOOKUP`

Current gap partition:
1. `docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_GAP_LEDGER.md`
2. `docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_GAP_INVENTORY.csv`
3. `docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_LEDGER.md`
4. `docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv`

Frozen tranche counts for the remaining `507` rows:
1. `201` ordinary extracted non-operator rows
2. `267` ordinary curated non-operator rows
3. `22` operator rows
4. `5` special extracted rows
5. `12` special curated rows

The frozen W069 support-surface ledgers that define that split are:
1. `docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_GAP_LEDGER.md`
2. `docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_GAP_INVENTORY.csv`
3. `docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_LEDGER.md`
4. `docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv`
5. `docs/function-lane/W69_SEAM_HEAVY_SUPPORTED_SURFACE_LEDGER.md`
6. `docs/function-lane/W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv`
7. `docs/function-lane/W69_SEAM_HEAVY_WITNESS_AUTHORING_RULES.md`
8. `docs/function-lane/W69_OPERATOR_AND_MODELED_WITNESS_CONVENTIONS.md`
9. `docs/function-lane/W69_FINAL_SUPPORTED_SURFACE_COVERAGE_LEDGER.md`

## 9B. Full-Coverage Rollout Shape
The widened `W069` program should proceed in explicit stages rather than as one
undifferentiated “finish the rest” lane.

Stage set:
1. coverage accounting and tranche register
   - derive the exact remaining supported-surface witness gap set
   - freeze the tranche order for the remaining `507` supported rows
2. ordinary extracted-surface rollout
   - generalize generator-backed witness emission for large ordinary
     `function_meta_extracted` families
   - drain the bulk of non-seam-heavy supported rows in tranche waves
3. curated and seam-heavy supported rollout
   - cover callable/helper, rich-value, presentation-aware, grouped, and other
     retained supported rows that need more curated witness payloads
4. operator and modeled-surface rollout
   - widen witness conventions beyond `OP_IMPLICIT_INTERSECTION` to the rest of
     the supported operator surface and any remaining modeled rows
5. final coverage reconciliation and publication
   - prove that every supported row is either:
     - witness-covered, or
     - explicitly dependency-blocked on a retained live authority
   - publish the full-surface reading rule and coverage ledger
   - reconcile the ledger against the frozen tranche partition above

## 10. Suggested Seed Set
Do not start with all `517` supported rows.
Start with a deliberately mixed seed:
1. `VLOOKUP`
2. `SUM`
3. `IF`
4. `XLOOKUP`
5. `LET`
6. `IMAGE`
7. `RTD`
8. `HYPERLINK`
9. `GROUPBY`
10. `OP_IMPLICIT_INTERSECTION`

Reason:
1. lookup
2. scalar aggregate
3. logical control
4. modern lookup
5. helper/callable
6. rich value
7. external-provider seam
8. presentation seam
9. grouped aggregation
10. operator / compatibility surface

If the model works for that seed, it will probably generalize honestly.

## 10A. First Seeded Family Selection
The first bounded seeded family for actual `V2` witness rollout should be the shared lookup-selection family:
1. `HLOOKUP`
2. `VLOOKUP`

Why this family should go first:
1. it already has a stable `V1` identity/profile surface,
2. it has a dedicated closure packet and replay bundle under `W068`,
3. it has a clear shared Lean substrate in `formal/lean/OxFunc/Functions/VhlookupFamily.lean`,
4. it is semantically rich enough to exercise signature/help/evidence/formal refs without immediately pulling in richer provider or callable seams,
5. `VLOOKUP` is already the concrete explanatory example used earlier in this plan.

The first-seed source surfaces for this family are:
1. identity/profile seed:
   - `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
2. contract/help-seed inputs:
   - `docs/function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md`
3. runtime evidence and closure provenance:
   - `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`
   - `docs/HISTORY.md`
   - `tools/w68-probe/run-w68-lookup-logical-baseline.ps1`
4. formal refs:
   - `formal/lean/OxFunc/Functions/VhlookupFamily.lean`
5. runtime implementation anchors:
   - `crates/oxfunc_core/src/functions/vhlookup_family.rs`
   - `crates/oxfunc_core/src/functions/surface_dispatch.rs`
6. first concrete witness seed artifact:
   - `docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_SEED_HVLOOKUP.json`
7. first bounded generator path:
   - `tools/w69-probe/run-w69-hvlookup-witness-generator.ps1`

The first-seed witness lane should exclude `IFS` initially.
Reason:
1. `IFS` shares the `W068` packet but not the same lookup-selection semantic substrate,
2. starting with `HLOOKUP` and `VLOOKUP` gives one cleaner family-shaped witness rollout before mixing lazy pair-scanning control flow into the same seed.

## 10B. First Mixed-Seed Tranche
After the bounded `HLOOKUP` / `VLOOKUP` family seed, the first widened mixed
tranche should be:
1. `FUNC.SUM`
2. `FUNC.IF`
3. `FUNC.XLOOKUP`
4. `FUNC.LET`
5. `FUNC.IMAGE`
6. `FUNC.HYPERLINK`
7. `FUNC.GROUPBY`
8. `FUNC.OP_IMPLICIT_INTERSECTION`

Why this tranche:
1. it covers ordinary scalar aggregation (`SUM`),
2. logical control flow (`IF`),
3. modern lookup (`XLOOKUP`),
4. callable/helper formation (`LET`),
5. rich-value publication (`IMAGE`),
6. presentation-aware output (`HYPERLINK`),
7. grouped aggregation (`GROUPBY`),
8. modeled operator semantics (`OP_IMPLICIT_INTERSECTION`).

Current exclusion from this first widened tranche:
1. `FUNC.RTD`
   - intentionally excluded until the retained `W043` RTD seam lane is
     narrowed further for witness-facing rollout.

First widened artifact surfaces:
1. artifact:
   - `docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_SEED_MIXED_TRANCHE_A.json`
2. generator:
   - `tools/w69-probe/run-w69-mixed-tranche-a-witness-generator.ps1`

## 11. Execution Sequence
1. define the `SemanticWitnessEntry` schema and stability tiers,
2. define `V1` -> `V2` field mapping,
3. define evidence/formal/contract reference projection rules,
4. write one concrete bounded witness export for the seed set,
5. validate that downstream help/signature UI can be driven entirely from that witness payload,
6. only then widen to the broader supported surface.

Schema anchor note:
1. the first live `SemanticWitnessEntry` schema and stability tiers now live in `docs/function-lane/OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md` Section 4.6 and Section 4.7,
2. the first explicit `V1` plus `W049` to `V2` projection rule now lives in `docs/function-lane/OXFUNC_DOWNSTREAM_METADATA_AND_HELP_CONTRACT.md` Section 4.8,
3. the first bounded seed artifact now lives in `docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_SEED_HVLOOKUP.json`,
4. downstream readers should cite the contract for schema ownership and the seed artifact for the first concrete witness projection rather than restating the schema in packet-local notes.

## 11A. First Bridge Shape
The first honest bridge into `V2` is:
1. `V1` export row as the stable identity and structural seed,
2. `W049` runtime model as the runtime attachment and immutable snapshot carrier,
3. OxFunc-curated witness enrichment as the semantic/help/provenance layer.

That means:
1. `V1` still owns:
   - `surface_stable_id`
   - `canonical_surface_name`
   - category/profile-bearing row facts
   - snapshot provenance fields
2. `W049` still owns:
   - `LibraryContextProvider`
   - immutable `LibraryContextSnapshot`
   - generation semantics
   - runtime grouping rather than CSV mirroring
3. `V2` owns:
   - structured help/signature payloads
   - semantic witness examples and modes
   - admitted-slice and orthogonal-validation summaries
   - machine-readable provenance refs

Generator rule:
1. generate `SemanticWitnessEntry` from `V1` plus `W049` plus enrichment surfaces,
2. do not author `V2` as a second unrelated catalog,
3. do not duplicate ownership of identity/profile facts that already belong to `V1`.

First serialized projection note:
1. the first bounded generator-backed `V2` artifact should remain a deterministic
   JSON seeded-family projection,
2. it should carry one witness-snapshot header plus an ordered `entries` array,
3. it should sort entries by `surface_stable_id`,
4. it should copy snapshot provenance from the `V1` rows used for generation,
5. it should not attempt to serialize a second runtime identity layer distinct
   from the retained `W049` attachment model.

## 11B. Current Execution Encoding
Live `W069` execution now runs through `.beads/` under:
1. workset rollout:
   - `oxf-jbk` `W069 Semantic Witness Snapshot V2 rollout`
2. lane epics:
   - `oxf-jbk.1` schema and stability tiers
   - `oxf-jbk.2` witness-generation pipeline
   - `oxf-jbk.3` runtime attachment and consumer mapping
   - `oxf-jbk.4` seeded family rollout
3. first execution child beads:
   - `oxf-jbk.1.1` audit current schema/seed/contract against required payload families
   - `oxf-jbk.1.2` finalize bounded field ownership and stability tiers
   - `oxf-jbk.2.1` define deterministic `V1` + `W049` + enrichment input map
   - `oxf-jbk.2.2` implement the first deterministic `HLOOKUP` / `VLOOKUP` witness generator
   - `oxf-jbk.3.1` narrow `W049` witness-bearing runtime attachment
   - `oxf-jbk.3.2` specify seeded-slice projection and generation semantics
   - `oxf-jbk.4.1` refresh the `HLOOKUP` / `VLOOKUP` seed through the generator
   - `oxf-jbk.4.2` choose the first mixed-seed tranche and downstream reading-guide shape
   - `oxf-jbk.4.3` populate the first mixed-seed tranche beyond `HLOOKUP` / `VLOOKUP`

Current ready path:
1. `oxf-jbk.1.1` is the first ready bead.
2. Later beads remain dependency-blocked until schema audit and field-ownership tightening land.

## 11C. First Schema Audit Readout
The first live audit bead (`oxf-jbk.1.1`) found that the current `V2` schema and
the `HLOOKUP` / `VLOOKUP` seed are viable, but not yet sufficiently tightened
for the first generator-backed slice.

The bounded gap set is:
1. help/signature requiredness and derivation rules are not yet explicit for the
   first generator-backed slice,
2. minimum semantic witness payload expectations are not yet locked for a seeded
   supported row,
3. minimum provenance coverage is not yet locked for a seeded supported row,
4. the field-by-field bridge from `V1` and `W049` into curated witness
   enrichment is not yet fully explicit.

The current supported-surface gap inventory is now pinned in:
1. `docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_GAP_LEDGER.md`
2. `docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_GAP_INVENTORY.csv`
3. `docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_LEDGER.md`
4. `docs/function-lane/W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv`
5. `docs/function-lane/W69_ORDINARY_ROW_WITNESS_GENERATION_RULES.md`
6. `docs/function-lane/W69_SEAM_HEAVY_SUPPORTED_SURFACE_LEDGER.md`
7. `docs/function-lane/W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv`
8. `docs/function-lane/W69_OPERATOR_AND_MODELED_WITNESS_CONVENTIONS.md`
9. `docs/function-lane/W69_FINAL_SUPPORTED_SURFACE_COVERAGE_LEDGER.md`
8. `docs/function-lane/W69_SEAM_HEAVY_WITNESS_AUTHORING_RULES.md`

That ledger reconciles the full supported surface as:
1. `517` supported non-deferred rows,
2. `10` witness-covered rows,
3. `507` remaining supported rows without `V2` witness coverage.

Those gaps are already covered by the current bead graph:
1. `oxf-jbk.1.2`
2. `oxf-jbk.2.1`
3. `oxf-jbk.3.1`

No additional execution lane was discovered by the audit.

## 11D. First Generator Validation Readout
The first generator-backed `HLOOKUP` / `VLOOKUP` seed validation confirms that
the bounded family survives the deterministic generation path.

Validated results:
1. `tools/w69-probe/run-w69-hvlookup-witness-generator.ps1` now regenerates
   `docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_SEED_HVLOOKUP.json`
   directly from the retained `V1` export plus bounded family templates and
   retained contract/evidence/formal refs.
2. The generated artifact remains family-bounded and ordered by
   `surface_stable_id`:
   - `FUNC.HLOOKUP`
   - `FUNC.VLOOKUP`
3. The regenerated rows preserve the required copied fields from `V1`,
   including:
   - `snapshot_generation`
   - `source_commit_short`
   - `source_commit_full`
   - `source_tree_state`
4. The generated rows still carry the required first-slice provenance coverage:
   - `catalog_export`
   - `contract_artifact`
   - `native_excel_replay`
   - `runtime_test`
   - `formal_artifact`
5. The current first seeded family therefore survives the generator path without
   reopening removed packet-local artifacts or creating a second catalog-owner
   surface.
6. The first bulk ordinary extracted tranche is now seeded in:
   - `docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_TRANCHE_T1_ORDINARY_EXTRACTED.json`
   - `tools/w69-probe/run-w69-ordinary-tranche-t1-generator.ps1`
   - it covers `201` ordinary extracted non-operator rows.
7. The second bulk ordinary curated tranche is now seeded in:
   - `docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_TRANCHE_T2_ORDINARY_CURATED.json`
   - `tools/w69-probe/run-w69-ordinary-tranche-t2-generator.ps1`
   - it covers `267` ordinary curated non-operator rows.

## 11E. First Downstream Reading Guide Shape
Downstream consumers should read `V2` witness rollout in three layers:
1. identity and support overlay
   - join on `surface_stable_id`
   - keep `V1` plus `W050` / `W051` as the owner of identity, counts, deferred
     status, and supported-surface overlay truth
2. witness enrichment
   - when a `V2` witness row exists, use it for:
     - `signature_display`
     - `arg_specs`
     - `help_summary`
     - `help_detail`
     - `semantic_modes`
     - `witness_examples`
     - `admitted_slice_note`
     - `orthogonal_validation_status`
     - `current_support_basis`
     - `provenance_refs`
3. fallback behavior
   - when no `V2` witness row exists yet, continue to read the row from `V1`
     only and do not invent downstream-private help or witness content.

Mixed-tranche reading rule:
1. witness presence is an enrichment signal, not a second support-status
   system,
2. seam-heavy or modeled rows in the mixed tranche should still surface their
   `metadata_status` and `special_interface_kind` alongside witness payloads,
3. downstream consumers should therefore be able to render one mixed witness UI
   without pretending that every supported row already has `V2` payload
   coverage.

Seam-heavy reading rule:
1. the retained seam-heavy tranche is dependency-gated by `W041`, `W043`, and
   `W049`,
2. the seam-heavy ledger and inventory remain the canonical row-level anchor for
   that gated remainder set,
3. downstream readers should follow the seam-heavy authoring rules rather than
   the ordinary-row template when rendering those rows.

Current widened-tranche result:
1. the first mixed artifact is now generated as an adjacent bounded `V2`
   projection rather than widening the `HLOOKUP` / `VLOOKUP` family artifact in
   place,
2. the generated artifact currently contains:
   - `FUNC.GROUPBY`
   - `FUNC.HYPERLINK`
   - `FUNC.IF`
   - `FUNC.IMAGE`
   - `FUNC.LET`
   - `FUNC.OP_IMPLICIT_INTERSECTION`
   - `FUNC.SUM`
   - `FUNC.XLOOKUP`
3. the ordinary seeded tranche output is now represented by both the `T1`
   extracted artifact and the `T2` curated artifact, neither of which should be
   read as final full-surface coverage.

Downstream reading-guide rule:
1. cite the frozen gap, tranche, seam-heavy, operator, and final coverage
   ledgers above as the canonical `W069` support-surface anchors,
2. treat the first tranche artifact as seeded witness output, not as the final
   supported-surface publication,
3. use the plan plus the function-lane index to navigate from the seed artifacts
   to the frozen ledgers in the order above.

## 12. Gate Criteria
This packet can only be reported `scope_complete` when:
1. one explicit `V2` schema exists,
2. at least one bounded witness artifact exists,
3. the artifact includes structured help, signature, evidence refs, and formal refs,
4. the runtime `LibraryContextSnapshot` direction is reflected in the design,
5. at least one mixed seed set is populated and reviewable,
6. downstream consumers can tell clearly what `V2` adds beyond `V1`,
7. every currently supported non-deferred row is either:
   - covered by a generated or curated `V2` witness row, or
   - explicitly dependency-blocked on a retained live authority such as `W043`
     or `W049` with that dependency recorded in the bead graph and in the final
     coverage ledger.
8. the frozen tranche ledger and row-level tranche register reconcile exactly to
   the current supported-surface remainder set.

## 13. Risks
1. bloating the snapshot with prose before the schema is stable,
2. mixing stable identity fields with unstable narrative fields,
3. treating evidence refs as optional decoration instead of core trust payload,
4. letting downstream consumers invent private help text before OxFunc owns it,
5. trying to widen to all `517` rows before the model is proven on a seed set.

## 14. Status
1. execution_state: `in_progress`
2. scope_completeness: `scope_partial`
3. target_completeness: `target_partial`
4. integration_completeness: `partial`
5. open_lanes:
   - reopen `W069` under a new full-coverage bead graph for the remaining
     `507` supported rows
   - freeze the full supported-surface tranche register
   - generalize ordinary extracted-surface witness generation
   - drain curated and seam-heavy supported rows
   - widen operator and modeled-surface witness coverage
   - publish final supported-surface coverage reconciliation
