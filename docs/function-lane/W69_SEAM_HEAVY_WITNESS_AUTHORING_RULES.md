# W69 Seam-Heavy Witness Authoring Rules

This note defines the curated authoring conventions for the special-interface
remainder set in `W069`.

## 1. Scope
This ruleset applies to the rows frozen in:
1. [W69_SEAM_HEAVY_SUPPORTED_SURFACE_LEDGER.md](W69_SEAM_HEAVY_SUPPORTED_SURFACE_LEDGER.md)
2. [W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv](W69_SEAM_HEAVY_SUPPORTED_SURFACE_INVENTORY.csv)

The targeted rows are the remaining supported non-deferred surfaces with:
1. `special_interface_kind != ordinary`
2. explicit dependency gates on retained live authorities such as `W041`,
   `W043`, or `W049`

## 2. Purpose
These rows are not suitable for the ordinary-extracted template.
They require curated witness authoring because they carry seam-sensitive or
host-sensitive behavior that must remain visible in the witness payload.

The authoring rules should:
1. keep the row's `V1` identity unchanged,
2. keep the dependency gate explicit,
3. make the seam or host surface obvious in the witness payload,
4. avoid collapsing seam-heavy rows into generic ordinary-language prose,
5. remain deterministic and reviewable.

## 3. Row Families
### 3.1 Callable Helper Rows
Rows:
1. `LAMBDA`
2. `BYCOL`
3. `BYROW`
4. `ISOMITTED`
5. `MAKEARRAY`
6. `MAP`
7. `REDUCE`
8. `SCAN`

Gate:
1. `W049`

Authoring rule:
1. use the helper/callable vocabulary already established by the helper and
   runtime carrier work,
2. keep higher-order behavior visible in `semantic_modes`,
3. keep witness examples tied to callable formation or callable runtime
   behavior rather than to ordinary scalar formulas,
4. keep `help_detail` explicit about the helper contract and admitted slice.

### 3.2 Host / Presentation / Locale Rows
Rows:
1. `RTD`
2. `NUMBERVALUE`
3. `NOW`
4. `TODAY`
5. `ASC`
6. `DBCS`
7. `JIS`

Gates:
1. `W043` for `RTD`
2. `W049` for the presentation, locale, and width-conversion rows

Authoring rule:
1. keep the host or presentation dependency visible in the witness payload,
2. make the current authoritative context or profile note explicit,
3. do not hide the retained authority behind generic ordinary-row prose,
4. keep witness examples anchored to the retained baseline behavior.

### 3.3 Registered-External Rows
Rows:
1. `CALL`
2. `REGISTER.ID`

Gate:
1. `W041`

Authoring rule:
1. keep registration and invocation semantics explicit,
2. treat these rows as seam-authored witnesses rather than ordinary extracted
   rows,
3. retain the host/add-in registration vocabulary in help and notes,
4. keep the gating authority visible in the witness payload.

## 4. Field Rules
1. `surface_stable_id` and `canonical_surface_name` must be copied unchanged
   from `V1`.
2. `metadata_status` must remain the `V1` metadata status.
3. `dependency_gate` must be present and non-empty for these rows.
4. `help_summary` should name the seam surface in one short sentence.
5. `help_detail` should describe the retained authority and the admitted slice.
6. `semantic_modes` should include the seam-specific behavior class.
7. `witness_examples` should prefer current-baseline replayed examples or
   clearly labeled seam examples.
8. `current_support_basis` should explain that the row is supported in the
   parked baseline but remains dependency-gated for witness rollout.

## 5. Determinism Rules
1. Rows must be emitted in stable `surface_stable_id` order.
2. The same input snapshot and dependency map must produce the same output.
3. The generator must not convert seam-heavy rows into ordinary rows.
4. If a row's witness detail remains incomplete, the gap must stay visible.

## 6. Closure Note
This ruleset is the authoring companion to the seam-heavy inventory ledger.
It is intended to make the later `W069` seam-heavy rollout explicit without
forcing the ordinary extracted template onto special-interface surfaces.

The retained seam-heavy inventory is fully represented by the seeded `SH1`
artifact already emitted in `W069`. Any later seam-heavy generator run should
therefore emit a zero-entry confirmation artifact unless new seam-heavy rows
are intentionally added to the parked baseline.
