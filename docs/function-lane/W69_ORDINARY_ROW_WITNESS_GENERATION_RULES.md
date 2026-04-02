# W69 Ordinary Row Witness Generation Rules

This note defines the reusable generation rules for the `W069` full-coverage
program's ordinary extracted-surface tranche.

## 1. Scope
This ruleset applies to the tranche frozen in:
1. [W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_LEDGER.md](W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_LEDGER.md)
2. [W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv](W69_SUPPORTED_SURFACE_WITNESS_TRANCHE_REGISTER.csv)

The targeted row family is:
1. `metadata_status = function_meta_extracted`
2. `special_interface_kind = ordinary`
3. `category != Operators`

This excludes:
1. special-interface rows,
2. operator rows,
3. doc-modeled rows,
4. any row whose current closure path is dependency-gated on a retained live
   authority rather than ordinary generation.

## 2. Purpose
The generator rules exist so the ordinary rollout can be driven by a reusable
template instead of row-by-row hand authoring.

The template must:
1. preserve the `V1` identity and support facts,
2. project one `V2` witness row per supported ordinary surface,
3. keep generated help and signature fields reviewable,
4. keep evidence and formal refs explicit,
5. remain deterministic across runs.

## 3. Required Inputs
Every ordinary witness row should start from:
1. the `V1` snapshot row,
2. the row's current support basis in `V1`,
3. any existing contract or execution record tied to that surface,
4. any formal artifact already attached to the surface,
5. any existing replay/evidence id already pinned for that surface.

If a row does not yet have a richer witness seed, the generator may still emit a
minimal ordinary witness stub as long as the row remains clearly marked as
ordinary-generation output and not seam-heavy curated output.

## 4. Field Rules
### 4.1 Identity
1. `surface_stable_id` must be copied unchanged from `V1`.
2. `canonical_surface_name` must be copied unchanged from `V1`.
3. `metadata_status` must remain the V1 status for the row.

### 4.2 Help and Signature
1. `help_summary` should be a short functional explanation suitable for a
   downstream consumer.
2. `help_detail` may expand on the exact current-baseline behavior if the row has
   an existing evidence or contract seed.
3. `signature_display` must match the current admitted surface and not invent a
   second syntax.
4. `arg_specs` should be derived from the published surface signature and the
   current admitted slice.

### 4.3 Semantic Witness
1. `semantic_modes` should capture the row's admitted behavior class.
2. `witness_examples` should prefer concrete current-baseline examples.
3. If only minimal evidence exists, the generator may emit a single baseline
   example plus a validation or error lane example when both are grounded in the
   current row's support basis.
4. `admitted_slice_note` should describe the ordinary supported slice without
   claiming cross-repo seam closure.

### 4.4 Provenance And Trust
1. `evidence_refs` should include the strongest available current repo refs.
2. `formal_refs` should include the primary Lean artifact if one exists.
3. `contract_refs` should include the closest function-lane contract surface.
4. `current_support_basis` should explain why the row is supported in the parked
   baseline.

## 5. Determinism Rules
1. Rows must be emitted in stable `surface_stable_id` order.
2. The same input snapshot must produce the same witness rows.
3. The generator must not invent new identity fields or new support-status
   semantics.
4. Any missing row-specific enrichment must remain visible as a gap rather than
   being hidden behind fabricated prose.

## 6. Closure Note
This ruleset is intended to make the `T1` ordinary extracted tranche reusable.
Later tranches may reuse the same structure, but they are not required to use
the same evidence density or prose depth.

