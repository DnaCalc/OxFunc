# OxFunc Downstream Metadata And Help Contract

Status: `active`
Date: 2026-03-29

## 1. Purpose
Define the downstream metadata, help, and signature contract that `DNA OneCalc` and other consuming hosts should read from OxFunc.

This document answers three questions:
1. Which fields in the current snapshot export are safe for downstream consumers to treat as stable now?
2. What is the preferred first OneCalc-facing payload shape for function help, argument help, and signature help metadata?
3. What is the preferred first `SemanticWitnessEntry` schema and stability model for `V2` witness payloads?
4. How should downstream consumers align with the longer-term runtime provider or immutable snapshot direction?

This is an OxFunc-owned contract note. It does not override Foundation doctrine or OxFml language-service ownership.

## 2. Authoritative Source Chain
Downstream consumers should read in this order:
1. this document for field stability and help-payload shape,
2. `OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md` for surface-labeling and admission-category rules,
3. `OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md` for field definitions and current export mechanics,
4. `W050` and `W051` for deferred/not-complete overlay truth,
5. `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md` for the preferred long-term runtime shape.

## 3. Snapshot Export Field Stability Classification

### 3.1 Stable Fields
The following fields are safe for downstream consumers to treat as stable identity and structural facts now. Their meaning and presence will not change without an explicit snapshot-generation bump and downstream notification.

| Field | Downstream Use | Stability Basis |
|-------|---------------|-----------------|
| `snapshot_id` | Snapshot family identity | Fixed per export family |
| `snapshot_generation` | Generation identity and pinning | Explicit per emission |
| `source_commit_short` | Provenance | Mechanical |
| `source_commit_full` | Provenance | Mechanical |
| `source_tree_state` | Provenance quality | Mechanical |
| `lane_id` | Lane identity | Fixed `oxfunc` |
| `entry_kind` | Row kind discrimination | Locked vocabulary: `built_in_function`, `built_in_operator` |
| `surface_stable_id` | Primary stable identity key | Locked pattern `FUNC.<NAME>` or `OP.<NAME>` |
| `canonical_surface_name` | Display name and parse seed | Current catalog truth |
| `xlcall_builtin_symbol` | XLL interop identity | `XLCALL.H` derived |
| `xlcall_builtin_code` | XLL interop identity | `XLCALL.H` derived |
| `arity_min` / `arity_max` | Arity bounds | Current catalog truth |
| `category` | Display grouping | Current catalog truth |
| `metadata_status` | Profile depth indicator | Locked vocabulary: `function_meta_extracted`, `function_meta_curated`, `catalog_only`, `doc_modeled` |

### 3.2 Usable-But-Provisional Fields
The following fields carry real current-tree information and are safe to consume for display, completion, and planning, but their exact vocabulary, granularity, or coverage may evolve without a breaking-change gate.

| Field | Downstream Use | Provisionality Reason |
|-------|---------------|----------------------|
| `registration_source_kind` | Row origin discrimination | Vocabulary may grow |
| `version_marker` | Version-gating display | Extraction coverage incomplete |
| `interesting` | Planning/priority hint | Planning flag, not semantic |
| `determinism_class` | Scheduling hint | Vocabulary stable but per-row extraction ongoing |
| `volatility_class` | Recalc hint | Same as above |
| `host_interaction_class` | Host-coupling hint | Same as above |
| `thread_safety_class` | Concurrency hint | Same as above |
| `arg_preparation_profile` | Adapter shape hint | Vocabulary may refine |
| `coercion_lift_profile` | Admission family hint | Family refs, not per-row contracts |
| `kernel_signature_class` | Kernel shape hint | Vocabulary may refine |
| `fec_dependency_profile` | Dependency hint | Vocabulary may refine |
| `surface_fec_dependency_profile` | Pipeline dependency hint | Vocabulary may refine |
| `special_interface_kind` | Seam-heavy signal | Vocabulary actively growing |
| `admission_interface_kind` | Admission shape hint | Not yet locked shared vocabulary |
| `preparation_owner` | Formation responsibility hint | Not yet locked shared vocabulary |
| `runtime_boundary_kind` | Runtime seam shape hint | Not yet locked shared vocabulary |

### 3.3 Current-Tree-Hint-Only Fields
The following fields are informational pointers into OxFunc internals. Downstream consumers may display them for debugging or tracing but should not build hard coupling against their values.

| Field | Downstream Use | Why Hint-Only |
|-------|---------------|---------------|
| `name_resolution_table_ref` | Localization table pointer | Points to current seed, not locked localization library |
| `semantic_trait_profile_ref` | Profile family ref | Family-level, not dereferenceable per-row |
| `gating_profile_ref` | Gating family ref | Family-level, not dereferenceable per-row |
| `arity_shape_note` | Free-form shape note | Unstable free text |
| `interface_contract_ref` | Seam contract pointer | Points to current workset/prelim docs |
| `source_catalog_ref` | Source row family | Internal generation detail |

## 4. Help And Signature Payload Shape

### 4.1 Current State
No OxFunc-backed help or signature payload retrieval is frozen yet. This is an acknowledged active seam gap.

Current available truth for downstream help surfaces:
1. `canonical_surface_name` and `surface_stable_id` from the snapshot export,
2. `arity_min` / `arity_max` for basic arity display,
3. `category` for grouping,
4. `metadata_status` to determine profile depth,
5. `special_interface_kind` and `admission_interface_kind` for seam-category labeling,
6. `determinism_class` / `volatility_class` / `host_interaction_class` for behavioral classification.

### 4.2 Preferred First OneCalc-Facing Payload Shape
The preferred first payload shape for each help surface is:

#### Function Help
```
FunctionHelpPayload:
  surface_stable_id: string          # from snapshot, stable
  canonical_surface_name: string     # from snapshot, stable
  category: string                   # from snapshot, stable
  arity_min: integer                 # from snapshot, stable
  arity_max: integer | null          # from snapshot, stable
  determinism_class: string          # from snapshot, usable
  volatility_class: string           # from snapshot, usable
  host_interaction_class: string     # from snapshot, usable
  special_interface_kind: string     # from snapshot, usable
  admission_category: string         # from labeling policy, see OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md
  implementation_status_label: string # from labeling policy
  help_summary: string | null        # NOT YET AVAILABLE from OxFunc
  help_detail: string | null         # NOT YET AVAILABLE from OxFunc
```

#### Argument Help
```
ArgumentHelpPayload:
  surface_stable_id: string          # parent function
  arg_index: integer                 # 0-based
  arg_name: string | null            # NOT YET AVAILABLE from OxFunc
  arg_description: string | null     # NOT YET AVAILABLE from OxFunc
  arg_required: boolean | null       # derivable from arity_min vs index
  arg_type_hint: string | null       # NOT YET AVAILABLE from OxFunc
```

#### Signature Help Metadata
```
SignatureHelpMetadata:
  surface_stable_id: string          # from snapshot, stable
  canonical_surface_name: string     # from snapshot, stable
  arity_min: integer                 # from snapshot, stable
  arity_max: integer | null          # from snapshot, stable
  arg_preparation_profile: string    # from snapshot, usable
  special_interface_kind: string     # from snapshot, usable
  admission_interface_kind: string   # from snapshot, usable
  arg_names: [string] | null         # NOT YET AVAILABLE from OxFunc
  signature_display: string | null   # NOT YET AVAILABLE from OxFunc
```

### 4.3 What Is Available Now vs What Requires Upstream Work
Available now from the snapshot export:
1. identity, arity, category, behavioral classification, and seam-category fields.
2. enough to populate completion lists, basic tooltips, and surface-labeling UI.

Requires upstream OxFunc or OxFml work:
1. `help_summary` and `help_detail` prose content per function,
2. per-argument names, descriptions, and type hints,
3. `signature_display` formatted signature strings,
4. localized help content.

### 4.4 Interim OneCalc Guidance
Until OxFunc publishes structured help payloads:
1. populate function help from snapshot export stable and usable fields,
2. populate completion lists from `canonical_surface_name`, `category`, `arity_min`/`arity_max`, and `admission_category` from the labeling policy,
3. show `special_interface_kind` and `admission_interface_kind` as visible metadata in help and scenario UI rather than hiding them,
4. do not invent local prose help content that would become a private second truth,
5. do not claim help coverage beyond what the snapshot export and labeling policy provide.

### 4.5 Alignment With Runtime Provider Direction
The preferred long-term direction is:
1. `LibraryContextProvider` / immutable `LibraryContextSnapshot` as the runtime substrate (see `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md`),
2. help and signature payloads should eventually be fields on `LibraryContextEntry` in the runtime model rather than separate retrieval surfaces,
3. the current CSV export remains the pinned interchange artifact for bounded integration, test pinning, and debugging,
4. downstream consumers should design against an immutable snapshot-shaped help/catalog source even if the first implementation is CSV-backed.

Transition rule:
1. OneCalc should consume the snapshot export through a local adapter that projects stable and usable fields into the payload shapes above,
2. when the runtime provider model materializes, OneCalc should swap the adapter backing from CSV to runtime snapshot without changing the consumer-facing payload shape,
3. help prose fields (`help_summary`, `help_detail`, `arg_name`, `arg_description`) should be treated as nullable until OxFunc populates them.

### 4.6 Semantic Witness Entry V2 Schema
The first explicit `V2` witness contract is:

```text
SemanticWitnessEntry:
  witness_schema_version: string
  surface_stable_id: string
  canonical_surface_name: string
  category: string | null
  metadata_status: string
  signature_display: string | null
  arg_specs: [SemanticWitnessArgSpec] | null
  help_summary: string | null
  help_detail: string | null
  semantic_modes: [string] | null
  witness_examples: [SemanticWitnessExample] | null
  admitted_slice_note: string | null
  orthogonal_validation_status: [string] | null
  current_support_basis: string | null
  provenance_refs: [SemanticWitnessRef]
  snapshot_generation: string
  source_commit_short: string
  source_commit_full: string
  source_tree_state: string

SemanticWitnessArgSpec:
  arg_index: integer
  arg_name: string
  arg_required: boolean
  arg_type_hint: string | null
  arg_behavior_note: string | null

SemanticWitnessExample:
  example_id: string
  summary: string
  outcome_note: string | null
  evidence_ref_hint: string | null

SemanticWitnessRef:
  ref_kind: string
  ref_value: string
  provenance_category: string
```

The current locked `provenance_category` vocabulary for this first contract is:
1. `native_excel_replay`
2. `runtime_test`
3. `formal_artifact`
4. `contract_artifact`
5. `execution_record`
6. `catalog_export`
7. `seam_or_handoff`

### 4.7 Semantic Witness Stability Tiers
`SemanticWitnessEntry` fields are grouped into four stability tiers:

1. `tier_a_identity_stable`
   - durable identity keys and schema presence
   - fields:
     - `witness_schema_version`
     - `surface_stable_id`
     - `canonical_surface_name`
2. `tier_b_structural_stable`
   - structural fields whose shape is intended to stay stable across bounded `V2` revisions
   - fields:
     - `category`
     - `metadata_status`
     - `snapshot_generation`
     - `source_commit_short`
     - `source_commit_full`
     - `source_tree_state`
3. `tier_c_curated_semantic`
   - OxFunc-curated semantic/help content that is expected to grow and refine without identity churn
   - fields:
     - `signature_display`
     - `arg_specs`
     - `help_summary`
     - `help_detail`
     - `semantic_modes`
     - `witness_examples`
     - `admitted_slice_note`
     - `orthogonal_validation_status`
     - `current_support_basis`
4. `tier_d_live_provenance`
   - current-tree references that are designed to remain machine-readable but may change as packets, exports, or formal artifacts evolve
   - fields:
     - `provenance_refs`

Downstream rule:
1. treat tiers `A` and `B` as the first hard-coupling surface for `V2`,
2. treat tier `C` as consumer-facing but OxFunc-curated semantic content,
3. treat tier `D` as traceable provenance intended for diagnostics, trust UI, and auditability rather than immutable ABI coupling.

### 4.8 V1 And W049 To V2 Projection Rule
The first `V2` witness generator must project from existing OxFunc truth surfaces rather than inventing a second ownership layer.

Projection rule:
1. `V1` remains the owner of row identity, category, and current profile-bearing metadata facts.
2. `W049` remains the owner of the preferred runtime attachment model:
   - `LibraryContextProvider`
   - immutable `LibraryContextSnapshot`
   - snapshot-generation semantics
   - runtime entry grouping rather than CSV mirroring
3. `V2` adds witness-bearing enrichment keyed by the same `surface_stable_id` and pinned to the same snapshot generation.

The intended first projection path is:

```text
V1 export row
  -> stable identity and structural seed
  -> runtime attachment through the W049 provider/snapshot model
  -> witness enrichment from OxFunc-owned help, contract, evidence, and formal refs
  -> SemanticWitnessEntry
```

Field-family ownership for the first bridge is:

| V2 field family | Current owner | Bridge rule |
|-----------------|---------------|-------------|
| `surface_stable_id`, `canonical_surface_name`, `category`, `metadata_status` | `V1` export | copy directly from the current snapshot export |
| `snapshot_generation`, `source_commit_*`, `source_tree_state` | `V1` export | copy directly from the current snapshot export |
| runtime attachment to immutable snapshot | `W049` runtime model | project into a witness-bearing runtime entry keyed by `surface_stable_id` plus snapshot generation; do not mirror CSV columns one-for-one |
| `signature_display`, `arg_specs`, `help_summary`, `help_detail` | `V2` curated layer | add as nullable witness enrichment owned by OxFunc |
| `semantic_modes`, `witness_examples`, `admitted_slice_note`, `orthogonal_validation_status`, `current_support_basis` | `V2` curated layer | add as curated semantic payload keyed to the current supported reading |
| `provenance_refs` | mixed current OxFunc evidence surfaces | emit machine-readable refs to contracts, execution records, formal artifacts, runtime tests, replay artifacts, and export provenance |

No-duplication rule:
1. `V2` must not restate or fork `V1` identity/profile ownership.
2. `V2` must not treat the CSV as the runtime ABI; the runtime carrier stays the `W049` provider/snapshot direction.
3. When a fact exists in both `V1` and a witness row, `V1` remains the primary owner unless OxFunc explicitly promotes that fact into a later runtime-owned witness contract.
4. `V2` should therefore be generated from `V1` plus `W049` plus enrichment surfaces, not authored as an unrelated second catalog.

### 4.8A First Generator-Backed Slice Rules
For the first generator-backed `HLOOKUP` / `VLOOKUP` slice, each field is now
classified as copied, generated, required curated enrichment, or optional
curated enrichment.

| Field | First-slice rule | Owner / source | Stability tier |
|-------|------------------|----------------|----------------|
| `witness_schema_version` | required generated constant | `V2` generator version constant | `tier_a_identity_stable` |
| `surface_stable_id` | required copied | `V1` export | `tier_a_identity_stable` |
| `canonical_surface_name` | required copied | `V1` export | `tier_a_identity_stable` |
| `category` | required copied | `V1` export | `tier_b_structural_stable` |
| `metadata_status` | required copied | `V1` export | `tier_b_structural_stable` |
| `signature_display` | required curated enrichment | OxFunc witness layer | `tier_c_curated_semantic` |
| `arg_specs` | required curated enrichment | OxFunc witness layer | `tier_c_curated_semantic` |
| `help_summary` | required curated enrichment | OxFunc witness layer | `tier_c_curated_semantic` |
| `help_detail` | optional curated enrichment | OxFunc witness layer | `tier_c_curated_semantic` |
| `semantic_modes` | required curated enrichment | OxFunc witness layer | `tier_c_curated_semantic` |
| `witness_examples` | required curated enrichment | OxFunc witness layer | `tier_c_curated_semantic` |
| `admitted_slice_note` | required curated enrichment | OxFunc witness layer | `tier_c_curated_semantic` |
| `orthogonal_validation_status` | optional curated enrichment | OxFunc witness layer | `tier_c_curated_semantic` |
| `current_support_basis` | required curated enrichment | OxFunc witness layer | `tier_c_curated_semantic` |
| `provenance_refs` | required generated mixed provenance set | `V1` + contract + evidence + runtime + formal surfaces | `tier_d_live_provenance` |
| `snapshot_generation` | required copied | `V1` export | `tier_b_structural_stable` |
| `source_commit_short` | required copied | `V1` export | `tier_b_structural_stable` |
| `source_commit_full` | required copied | `V1` export | `tier_b_structural_stable` |
| `source_tree_state` | required copied | `V1` export | `tier_b_structural_stable` |

First-slice minimum payload rule:
1. a supported seeded row must emit every required field above,
2. `help_detail` and `orthogonal_validation_status` may be `null` or omitted
   from generator-backed authoring only when the seeded row has no bounded extra
   detail beyond the required fields,
3. `arg_specs` must include all surfaced arguments in positional order,
4. `witness_examples` must contain at least one success lane and one validation
   or error lane for the first seeded supported family slice.

First-slice provenance coverage rule:
1. every generated seeded row must include at least one `catalog_export` ref,
2. at least one `contract_artifact` ref,
3. at least one replay-bearing ref from `native_excel_replay` or
   `execution_record`,
4. at least one `runtime_test` ref,
5. at least one `formal_artifact` ref.

Optional provenance for the first slice:
1. `seam_or_handoff` refs are optional and only appear when the seeded family
   genuinely depends on a retained seam packet or handoff surface.

### 4.8B First Seeded Slice Field-Source Map
The first generator-backed `HLOOKUP` / `VLOOKUP` slice now has an explicit
field-source map with no unowned fields.

| Field family | Owning source surface(s) | First-slice extraction rule |
|-------------|---------------------------|-----------------------------|
| `witness_schema_version` | `V2` generator config | emit the generator's current schema/version constant |
| `surface_stable_id`, `canonical_surface_name`, `category`, `metadata_status` | `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv` | copy directly from the current `V1` row keyed by `surface_stable_id` |
| `snapshot_generation`, `source_commit_short`, `source_commit_full`, `source_tree_state` | `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv` | copy directly from the same `V1` row and snapshot header fields |
| `signature_display` | `docs/function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md` plus generator family templates | emit from the bounded `HLOOKUP` / `VLOOKUP` family template anchored in the retained contract |
| `arg_specs` | `docs/function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md` plus generator family templates | emit ordered argument specs from the same bounded family template, including requiredness and current-baseline behavior notes |
| `help_summary`, `help_detail` | `docs/function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md` | derive bounded consumer-facing help from the retained current-baseline contract text; do not invent downstream-only help |
| `semantic_modes`, `admitted_slice_note`, `current_support_basis` | `docs/function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md`, `docs/worksets/W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md`, `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md` | derive current admitted-slice and support-basis wording from the retained contract and parked-baseline reading |
| `orthogonal_validation_status` | `docs/worksets/W069_SEMANTIC_WITNESS_SNAPSHOT_V2_PLAN.md` and parked-baseline doctrine | emit the current orthogonal validation posture for the seeded slice; presently `locale_version_sweeps_pending` |
| `witness_examples` | `tools/w68-probe/run-w68-lookup-logical-baseline.ps1`, `.tmp/w68-lookup-logical-results.csv`, `docs/function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md` | project seeded success and validation/error examples from the retained replay bundle and bounded contract |
| `provenance_refs.catalog_export` | `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv` | emit one catalog-export ref for the exact `surface_stable_id` row |
| `provenance_refs.contract_artifact` | `docs/function-lane/FUNCTION_SLICE_LOOKUP_AND_LOGICAL_RESIDUALS_CONTRACT_PRELIM.md` | emit the retained family contract as the current contract ref |
| `provenance_refs.native_excel_replay` | `docs/function-lane/FUNCTION_LANE_EVIDENCE_ID_REGISTRY.md`, `tools/w68-probe/run-w68-lookup-logical-baseline.ps1` | emit the seeded replay evidence id `W68-LOOKUP-LOGICAL-RESIDUALS-BL-20260401` and the retained probe ref |
| `provenance_refs.runtime_test` | `crates/oxfunc_core/src/functions/vhlookup_family.rs` | emit the retained runtime implementation anchor for the shared lookup family |
| `provenance_refs.formal_artifact` | `formal/lean/OxFunc/Functions/VhlookupFamily.lean` | emit the retained formal substrate ref for the shared lookup-selection family |

Unowned-field result:
1. there are no first-slice fields whose source surface remains unidentified,
2. the current remaining work is generatorization and runtime attachment, not
   source-surface discovery.

### 4.8C First Runtime Attachment Rule
The first bounded runtime attachment rule for `V2` witness payloads is:
1. `W049` owns the runtime entry and immutable snapshot attachment model,
2. `W069` owns the witness payload attached to that runtime entry,
3. runtime consumers join witness-bearing rows on:
   - `surface_stable_id`
   - `LibraryContextSnapshotRef`
4. the witness payload does not create a second runtime identity system,
5. rows without current witness payloads remain valid runtime entries without
   witness attachment.

First-slice interpretation:
1. the seeded `HLOOKUP` / `VLOOKUP` slice is not a second catalog,
2. it is witness enrichment attached to the retained runtime entry shape,
3. serialized `V2` artifacts are projections of that attachment model rather
   than the owner of runtime identity.

### 4.8D First Serialized Projection Rule
The first deterministic serialized `V2` artifact for a bounded seeded slice is a
JSON document with:
1. one witness-snapshot header,
2. one bounded seeded-family marker,
3. an ordered `entries` array of `SemanticWitnessEntry` rows.

First bounded header shape:
1. `witness_snapshot_id`
2. `witness_schema_version`
3. `source_snapshot_ref`
4. `seed_family`
5. `entries`

Deterministic generation rule:
1. the header must be emitted in the same field order on every run,
2. `entries` must be sorted by `surface_stable_id` ascending,
3. each entry must carry the copied `snapshot_generation` and
   `source_commit_*` / `source_tree_state` facts from the current `V1` export
   row used for generation,
4. the serialized artifact must not inline a second runtime identity object or a
   CSV-mirroring runtime object model,
5. the serialized artifact is therefore the deterministic projection of the
   witness payload attached to snapshot entries, not a replacement for the
   `W049` runtime carrier.

First-slice publication rule:
1. the first generator-backed artifact may remain family-bounded
   (`HLOOKUP` / `VLOOKUP`) rather than pretending to be a full-surface `V2`,
2. later mixed-seed tranches may either:
   - widen the same JSON family artifact shape, or
   - publish an adjacent tranche artifact with the same header/entry rules,
3. downstream consumers should treat the first bounded artifact as a
   deterministic witness projection format, not yet the final all-surface
   packaging decision.

### 4.9 First Seed Artifact
The first bounded `V2` seed artifact now lives at:
1. `docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_SEED_HVLOOKUP.json`

Current role:
1. it is the first concrete `SemanticWitnessEntry` projection exercise for the shared `HLOOKUP` / `VLOOKUP` family,
2. it demonstrates that the `V2` schema can be populated from the retained `V1` export, current contract surfaces, replay evidence, runtime source, and Lean formal refs without reopening removed packet-local files,
3. it is a seed artifact, not yet the full generator or the full supported-surface rollout.

### 4.10 First Schema Audit Findings
The first `W069` schema/seed audit against the required `V2` payload families found
that the current contract and `HLOOKUP` / `VLOOKUP` seed are directionally sound
but still leave four bounded gaps before the first generator-backed slice is
honestly locked.

Gap set:
1. help/signature payload ownership is present, but the first slice does not yet
   say which help/signature fields are required, optional, or derived for
   generator-backed emission.
2. semantic witness payload shape is present, but the first slice does not yet
   state the minimum required witness content for a supported seeded row:
   semantic modes, witness examples, admitted-slice note, and support-basis
   text are all present in the seed but not yet tightened as slice-level rules.
3. provenance payload shape is present, but the first slice does not yet lock the
   minimum provenance coverage rule for a generator-backed seeded row:
   catalog export, contract, replay/evidence, runtime, and formal refs should
   all be present for the first supported family slice.
4. stable-vs-enrichment ownership is present at the family level, but the first
   slice does not yet finish the field-by-field bridge between:
   - copied `V1` facts,
   - `W049` runtime attachment facts,
   - OxFunc-curated witness enrichment facts.

Existing bead mapping for these gaps:
1. `oxf-jbk.1.2`
   - finalize bounded field ownership, requiredness, and stability-tier rules
     for the first generator-backed slice
2. `oxf-jbk.2.1`
   - define the deterministic field-source map from `V1`, `W049`, and witness
     enrichment surfaces
3. `oxf-jbk.3.1`
   - narrow the retained `W049` witness-bearing runtime attachment shape

Audit result:
1. no new bead creation is required from this first audit pass,
2. the current graph already covers the discovered gaps,
3. the first seed artifact remains valid as a seed/projection exercise, but not
   yet as a fully generator-backed witness slice.

## 5. Authoritative Upstream References
1. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1_README.md`
2. `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`
3. `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md`
4. `docs/function-lane/W49_RUNTIME_LIBRARY_CONTEXT_CONSUMER_WALKTHROUGH.md`
5. `docs/function-lane/OXFUNC_SURFACE_ADMISSION_AND_LABELING_POLICY.md`
6. `docs/worksets/W050_DEFERRED_CURRENT_VERSION_SURFACE.md`
7. `docs/worksets/W051_IN_SCOPE_CURRENT_VERSION_NOT_COMPLETE_SURFACE.md`
8. `docs/function-lane/OXFUNC_SEMANTIC_WITNESS_SNAPSHOT_V2_SEED_HVLOOKUP.json`
