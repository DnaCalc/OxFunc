# W69 Operator And Modeled Witness Conventions

This note defines the witness-authoring conventions for the operator tranche in
`W069` and for the `doc_modeled` operator seed already carried by
`OP_IMPLICIT_INTERSECTION`.

## 1. Scope
This ruleset applies to:
1. the `T3` operator tranche frozen in
   [W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_LEDGER.md](W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_LEDGER.md),
2. the `doc_modeled` operator seed `FUNC.OP_IMPLICIT_INTERSECTION`,
3. any later operator rows that are rolled into the `V2` witness surface.

## 2. Purpose
Operator rows should not be forced through the ordinary function witness
template.

The witness payload for these rows must:
1. preserve the `V1` identity and support facts,
2. keep the operator form explicit,
3. keep context sensitivity and compatibility behavior visible,
4. keep the `doc_modeled` operator seed aligned with the same operator family
   conventions,
5. remain reviewable without pretending that operator syntax is just ordinary
   function-call syntax.

## 3. Shared Operator Rules
1. `surface_stable_id` must be copied unchanged from `V1`.
2. `metadata_status` must remain the V1 status for the row.
3. `signature_display` should render the operator shape rather than a function
   call wrapper whenever the surface is operator-shaped.
4. `help_summary` should name the operator behavior in one short sentence.
5. `help_detail` should explain the operator's admitted slice and any context
   sensitivity.
6. `semantic_modes` should include the operator class, such as:
   - reference formation
   - implicit intersection projection
   - arithmetic projection
   - comparison projection
   - spill/reference shaping
7. `witness_examples` should prefer context-visible examples rather than generic
   function-call examples.
8. `current_support_basis` should explain the parked-baseline support claim and
   whether the row is `function_meta_extracted`, `function_meta_curated`, or
   `doc_modeled`.

## 4. Modeled Operator Rule
`FUNC.OP_IMPLICIT_INTERSECTION` remains the model row for the operator family
and uses the same operator witness family with `metadata_status = doc_modeled`.

For this row:
1. the `@` surface and legacy `_xlfn.SINGLE` compatibility story should remain
   explicit,
2. the witness payload should reflect context-sensitive scalarization,
3. the support note should distinguish between operator semantics and syntax
   compatibility.

## 5. Determinism Rules
1. Operator rows must be emitted in stable `surface_stable_id` order.
2. The same input snapshot must produce the same operator witness rows.
3. Operator witness rows must not be disguised as ordinary function-call rows.
4. If a row needs dependency-gated treatment, the gate must stay explicit in the
   witness payload.

## 6. Closure Note
This ruleset is the operator companion to the seam-heavy and ordinary-row
authoring notes. It gives the `W069` operator tranche a shared witness shape
without collapsing it into the ordinary extracted template.

The current seeded operator/model artifact already covers the parked operator
surface together with the `doc_modeled` `FUNC.OP_IMPLICIT_INTERSECTION` row.
Any later operator/model generator run should therefore emit a zero-entry
confirmation artifact unless new operator rows are intentionally added to the
parked baseline.
