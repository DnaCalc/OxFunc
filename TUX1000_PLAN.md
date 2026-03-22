# OxFunc TUX-1000 Plan

## 1. Purpose
`TUX1000_PLAN.md` is the aspirational execution adjunct to `CHARTER.md`.

It defines how OxFunc executes with high rigor while preserving throughput: small slices, full closure, explicit gates, and reusable method infrastructure.

## 2. North-Star Objective
Convert function compatibility from folklore into a repeatable assurance pipeline:
1. explicit contracts,
2. Lean formal obligations,
3. Rust runtime implementation,
4. differential and empirical evidence,
5. promotion gates that block unscoped claims.

## 3. Non-negotiable Operating Constraints
1. Sequence-only planning (no date-based commitments).
2. Dual-axis version scope on behavior claims.
3. Clean-room evidence only.
4. No promotion without replayable artifacts.
5. No hidden ambiguity; unresolved lanes remain explicit and bounded.
6. Completion reporting must be scope-qualified (scope/target/integration), never unqualified "done".

## 4. Slice Architecture (Coupled Lanes)
Every function/operator slice must traverse six synchronized artifacts:
1. Contract artifact.
2. Formal artifact.
3. Runtime artifact.
4. Verification artifact.
5. Evidence artifact.
6. Correlation artifact.

Promotion intent:
1. `draft` means incomplete or open-boundary contract.
2. `provisional` means complete shape with bounded unresolved lanes.
3. `validated` means scope-bounded closure with required artifacts and reproducible evidence.

Assurance maturity intent:
1. `exercised` for OxFunc-local closure.
2. `green-validated` for Foundation-level pack closure.

## 5. Kickoff Program (W1-W7)
This is one combined kickoff program, not seven unrelated documents.

### 5.1 W1 - `PI()` End-to-End Method Seed
Purpose:
1. establish the reusable slice template and correlation discipline.

Primary outcome:
1. reusable method pattern proven on a minimal deterministic function.

### 5.2 W2 - Floating-point Characterization
Purpose:
1. characterize IEEE-edge behavior as Excel-observable policy input.

Primary outcome:
1. normalized behavior map for `-0`, infinities, NaN, and subnormal lanes across formula/materialization/reference boundaries.

### 5.3 W3 - Value Universe and Extended Types
Purpose:
1. lock the value algebra used by formal and runtime lanes.

Primary outcome:
1. explicit typed value sets and boundary-specific admissibility model.

### 5.4 W4 - Coercion and Ref->Val Seam
Purpose:
1. formalize coercion primitives and the explicit out-of-model resolver seam.

Primary outcome:
1. one selected baseline seam model plus documented alternatives and tradeoffs.

### 5.5 W5 - `ABS` Full Formality
Purpose:
1. first nontrivial scalar function with full adapter/kernel/array/edge behavior closure.

Primary outcome:
1. complete contract/formal/runtime/evidence closure for `ABS` under declared scope.

### 5.6 W6 - `XMATCH` Deterministic Quirks Closure
Purpose:
1. close a behavior-rich deterministic candidate and settle classification confidence.

Primary outcome:
1. evidence-backed decision: downgrade interest tier or retain high-interest with explicit rationale.

### 5.7 W7 - String Characterization
Purpose:
1. characterize Excel string comparison/normalization/limit behavior via source extraction and empirical runs.

Primary outcome:
1. version-scoped string policy map for comparison semantics, control/unicode behavior, and boundary normalization.

## 6. Dependency Graph and Gate Discipline
Dependencies:
1. W1 has no upstream dependency.
2. W2 depends on W1 method template.
3. W3 depends on W2 characterization baseline.
4. W4 depends on W3 taxonomy closure.
5. W5 depends on W2 + W3 + W4.
6. W6 depends on W3 + W4 + W7 and consumes W2 numeric-edge findings.
7. W7 depends on W1 method template and feeds W3/W6.
8. W3 may start before W7 closure but must absorb W7 outputs before W3 validation closure.

Combined kickoff gates:
1. KG1 Method gate: W1 closure is reusable without ad-hoc process edits.
2. KG2 Numeric-policy gate: W2 yields replayable FP behavior map.
3. KG3 Value-core gate: W3 yields stable value universe and open-question ledger.
4. KG4 Coercion-seam gate: W4 yields selected seam contract plus alternatives.
5. KG5 Function-closure gate: W5 reaches at least `provisional` with complete artifact chain.
6. KG6 Deterministic-quirks gate: W6 records classification decision with evidence.
7. KG7 String semantics gate: W7 yields replayable string characterization and policy map.

## 7. Shared Artifact Contract for Kickoff
Mandatory outputs across W1-W7:
1. workset spec with explicit state and gate status.
2. conformance-row binding updates (`FDEF-*` lineage).
3. function-lane narrative spec updates for each scope area.
4. machine-readable correlation/evidence links where applicable.
5. explicit unknowns register (never implicit drift).

## 8. Later Successor Packets
1. `W046` now owns the `CALL` / `REGISTER.ID` worksheet UDF-registration seam as a distinct successor packet rather than leaving it buried in `W023`.

## 8. Foundation Handoff Expectations
For each completed workset, prepare a Foundation-consumable handoff bundle:
1. claimed scope and profile bounds,
2. requirement/evidence bindings,
3. replay artifacts and tool provenance,
4. unresolved-policy notes requiring Foundation decision.

Rule:
1. OxFunc kickoff closure is a precondition for robust Foundation pack integration, not a substitute for it.

## 9. Failure and Divergence Policy
Any divergence discovered during execution must be classified and persisted as one of:
1. spec gap,
2. policy ambiguity,
3. implementation defect,
4. environmental variability.

Each divergence becomes a replayable case and a tracked closure obligation.

## 10. Operating Posture
1. Keep slices small and complete.
2. Prefer one closed chain over wide unverified surface.
3. Treat each validated slice as reusable infrastructure.
4. Expand breadth only when the method itself remains stable.

## 11. Relationship to Doctrine
1. `CHARTER.md` is normative for mission/scope/done criteria.
2. `OPERATIONS.md` is normative for execution doctrine.
3. This plan is aspirational and directional, never overriding charter or Foundation doctrine.

## 12. Post-kickoff Extension Packets
1. `W8.1` String follow-up checklist:
   - `docs/worksets/W008_STRING_W8_1_CHECKLIST.md`
2. `W9` XLL add-in bridge packet:
   - `docs/worksets/W009_XLL_ADDIN_BRIDGE.md`
   - goal: build `OxFunc64.xll` as an adapter around OxFunc core functions, with seed exports (for example `ox_ABS`) and side-by-side native-vs-OxFunc workbook validation packs.
3. `W10` ten-function mixed-seam packet:
   - `docs/worksets/W010_TEN_FUNCTION_MIXED_SEAMS.md`
   - goal: execute one breadth packet across `SUM`, `IF`, `INDEX`, `MATCH`, `ISNUMBER`, `NOW`, `XLOOKUP`, `INDIRECT`, `SEQUENCE`, and `OP_ADD` to pressure-test classification, layering, and U/Q export policy.
4. `W11` XLL registration-flags evidence packet:
   - `docs/worksets/W011_XLL_REGISTRATION_FLAGS_EVIDENCE.md`
   - goal: produce empirical closure for volatile/thread-safe/macro-type registration flags before enabling profile-derived mapping.
5. `W12` moderate function expansion packet:
   - `docs/worksets/W012_MODERATE_FUNCTION_EXPANSION.md`
   - goal: implement a moderate fifteen-function batch (`AVERAGE`, `COUNT`, `COUNTA`, `IFERROR`, `ROUND`, `TEXTJOIN`, `TODAY`, `RAND`, `OFFSET`, `CELL`, `AND`, `CLEAN`, `DATE`, `EXACT`, `HSTACK`), with an explicit empirical-first probe on `CELL`, then feed stronger follow-back evidence into W11.
6. `W13` deceptively simple boundary functions:
   - `docs/worksets/W013_DECEPTIVELY_SIMPLE_BOUNDARY_FUNCTIONS.md`
   - goal: run one packet over apparently simple functions (`SIN`, `ASIN`, `N`, `T`, `TYPE`, `VALUE`, `ROW`, `COLUMN`, `TEXT`, `DOLLAR`, `FIXED`) to settle coercion, caller-context, type-classification, and locale/format-parser seams.
7. `W14` implicit intersection operator and scalarization seam:
   - `docs/worksets/W014_IMPLICIT_INTERSECTION_OPERATOR.md`
   - goal: characterize the `@` operator as an explicit semantic seam across OxFunc, OxFml, and FEC/F3E, including compatibility aliasing (`SINGLE`, `_xlfn.SINGLE`), caller-context scalarization, spill/provenance requirements, and executable/formal test planning.
8. `W15` `CELL` and `INFO` host-query functions:
   - `docs/worksets/W015_CELL_AND_INFO_HOST_QUERY_FUNCTIONS.md`
   - goal: close the deferred `CELL` and `INFO` information-function seam through an explicit typed host-query contract, reproducible empirical baselines, and aligned runtime/formal artifacts without pretending these functions are pure local kernels.
9. `W16` bulk non-interesting functions and operators:
   - `docs/worksets/W016_BULK_NON_INTERESTING_FUNCTIONS_AND_OPERATORS.md`
   - goal: freeze the remaining non-interesting inventory and execute family-by-family breadth implementation on top of the existing OxFunc method stack, starting with reusable pure unary numeric values-only families.
10. `W17` deferred low-interest hardening and host-seam packet:
   - `docs/worksets/W017_DEFERRED_LOW_INTEREST_FUNCTIONS_REQUIRING_HARDENING_AND_HOST_SEAMS.md`
   - goal: absorb the low-interest residuals extracted at `W16` closure, separating host/query seam work from bounded semantic hardening families that still need deeper replay before honest closure.
11. `W18` Replay appliance packet-adapter baseline:
   - `docs/worksets/W018_REPLAY_APPLIANCE_PACKET_ADAPTER_BASELINE.md`
   - goal: incorporate the Foundation replay handoff into OxFunc without weakening local semantic or evidence authority, publish the packet-first adapter contract and conservative capability manifest, and bind packet/evidence/invariant concepts into the local doctrine stack.
12. `W19` packet witness-distillation and retention baseline:
   - `docs/worksets/W019_PACKET_WITNESS_DISTILLATION_AND_RETENTION_BASELINE.md`
   - goal: define packet-first reduction units, lifecycle, supersession, quarantine, and retention expectations for future OxFunc reduced witnesses without overclaiming `cap.C4` or `cap.C5`.
13. `W20` emitted replay bundle layout and index baseline:
   - `docs/worksets/W020_OXFUNC_REPLAY_BUNDLE_LAYOUT_AND_INDEX_BASELINE.md`
   - goal: define the first explicit on-disk OxFunc replay bundle layout and index contract for packet-first bundles, including the W15 worked example.
14. `W21` first live W15 replay-adapter run baseline:
   - `docs/worksets/W021_W15_FIRST_LIVE_REPLAY_ADAPTER_RUN_BASELINE.md`
   - goal: emit the first real OxFunc replay bundle for W15 and judge it against the declared skeleton, conformance checklist, and diff/explain shape targets.
15. `W22` criteria-family shape hardening:
   - `docs/worksets/W022_CRITERIA_FAMILY_SHAPE_HARDENING.md`
   - goal: close the old generic criteria-family shape gap by pinning the current-baseline split between `AVERAGEIF` anchoring and exact-shape `*IFS` behavior.
16. `W23` deferred host, metadata, and database functions:
   - `docs/worksets/W023_DEFERRED_HOST_METADATA_AND_DATABASE_FUNCTIONS.md`
   - goal: isolate the low-interest residuals that are not honestly pure value functions on the current boundary, namely the host-sensitive cluster, the database family, and `ISFORMULA`.
17. `W24` ordinary functions mega-batch execution plan:
   - `docs/worksets/W024_ORDINARY_FUNCTIONS_MEGA_BATCH_EXECUTION_PLAN.md`
   - goal: freeze the remaining ordinary `W17` residuals into one uninterrupted execution checklist that still preserves family-level outputs, replay artifacts, tests, Lean alignment, and closure discipline.
18. `W25` deferred misc add-in and dynamic-array outliers:
   - `docs/worksets/W025_DEFERRED_MISC_ADDIN_AND_DYNAMIC_ARRAY_OUTLIERS.md`
   - goal: characterize the native `#NAME?` outliers (`EUROCONVERT`, `RANDARRAY`) that proved not to be ordinary current-host worksheet functions in `W24`.
19. `W26` deferred locale/profile and provider-sensitive ordinary outliers:
   - `docs/worksets/W026_DEFERRED_LOCALE_PROFILE_AND_PROVIDER_SENSITIVE_ORDINARY_OUTLIERS.md`
   - goal: characterize locale/profile-sensitive width-conversion and separator-default behavior plus provider-bound `TRANSLATE` behavior outside the ordinary mega-batch assumptions.
20. `W27` deferred advanced bond and odd-bond hardening:
   - `docs/worksets/W027_DEFERRED_ADVANCED_BOND_AND_ODD_BOND_HARDENING.md`
   - goal: reopen the extracted advanced bond and odd-bond families only after direct native parity packets and substrate corrections exist.
21. `W28` function name localization library discovery:
   - `docs/worksets/W028_FUNCTION_NAME_LOCALIZATION_LIBRARY_DISCOVERY.md`
   - goal: discover the official multilingual support surfaces for Excel function names, capture locale/article seeds and current-list extraction routes, and plan the future localization library that will feed catalog and library-context work.
22. `W29` finance functions F# benchmark cross-check:
   - `docs/worksets/W029_FINANCE_FUNCTIONS_FSHARP_BENCHMARK_CROSSCHECK.md`
   - goal: compare OxFunc finance families against the public ExcelFinancialFunctions F# implementation and compatibility/test surface, then classify any discrepancy against direct Excel evidence.
23. `W30` deferred locale/profile-sensitive text and number functions:
   - `docs/worksets/W030_DEFERRED_LOCALE_PROFILE_SENSITIVE_TEXT_AND_NUMBER_FUNCTIONS.md`
   - goal: take over the `W26` locale/profile-sensitive subset (`ASC`, `DBCS`, `JIS`, `NUMBERVALUE`) with an explicit host/profile matrix and honest boundary semantics.
24. `W31` deferred provider language functions:
   - `docs/worksets/W031_DEFERRED_PROVIDER_LANGUAGE_FUNCTIONS.md`
   - goal: take over provider-bound language functions such as `TRANSLATE` after `W26` proved they are not ordinary pure local kernels on the current host boundary.
25. `W32` reopened finance parity gaps from benchmark:
   - `docs/worksets/W032_REOPENED_FINANCE_PARITY_GAPS_FROM_BENCHMARK.md`
   - goal: repair the concrete OxFunc-vs-Excel finance discrepancies reopened by `W29`, currently `COUPDAYS`, `XNPV`, and `XIRR`.
26. `W33` information predicates and forecast compatibility closure:
   - `docs/worksets/W033_INFORMATION_PREDICATES_AND_FORECAST_COMPATIBILITY_CLOSURE.md`
   - goal: close the newly exposed ordinary-function catalog gaps from the corrected `W28` local canonical list, namely the missing `IS*` predicate family members and the `FORECAST` / `FORECAST.LINEAR` pair.
27. `W34` deferred width-conversion host/profile capability baseline:
   - `docs/worksets/W034_DEFERRED_WIDTH_CONVERSION_HOST_PROFILE_CAPABILITY_BASELINE.md`
   - goal: take over `ASC`, `DBCS`, and `JIS` after `W030` proved they are host/profile-sensitive rather than ordinary pure text functions on the current boundary.
28. `W35` deferred `NUMBERVALUE` locale-default profile baseline:
   - `docs/worksets/W035_DEFERRED_NUMBERVALUE_LOCALE_DEFAULT_PROFILE_BASELINE.md`
   - goal: isolate the omitted-default locale/profile behavior of `NUMBERVALUE` from the already-admitted explicit-separator lanes.
29. `W36` deferred provider-language capability baseline:
   - `docs/worksets/W036_DEFERRED_PROVIDER_LANGUAGE_CAPABILITY_BASELINE.md`
   - goal: carry `TRANSLATE` into a provider-language capability packet and decide how it aligns with `DETECTLANGUAGE`.
30. `W37` reopened `XIRR` large-root solver precision:
   - `docs/worksets/W037_REOPENED_XIRR_LARGE_ROOT_SOLVER_PRECISION.md`
   - goal: resolve the remaining direct-Excel precision drift on the large positive-root `XIRR` lane after the broader `W032` repair packet.
31. `W38` functional lambda and helper family:
   - `docs/worksets/W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md`
   - goal: execute the highest-value remaining in-core interesting seam packet for `LET`, `LAMBDA`, omission, and the lambda-helper family.
32. `W39` dynamic-array shaping and reshaping family:
   - `docs/worksets/W039_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY.md`
   - goal: close the remaining high-interest spill-shape and array-reshaping family on top of the existing array/publication substrate.
33. `W40` reference metadata and formula visibility functions:
   - `docs/worksets/W040_REFERENCE_METADATA_AND_FORMULA_VISIBILITY_FUNCTIONS.md`
   - goal: characterize the reference-identity, formula-visibility, and sheet-metadata functions that sit between pure semantics and host/grid truth.
34. `W41` external data provider and cube functions:
   - `docs/worksets/W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md`
   - goal: characterize the high-interest provider-bound and cube-context family, with explicit capability and external-failure taxonomy rather than fake pure-kernel claims.
35. `W42` deferred callable seam field lock and higher-order evidence:
   - `docs/worksets/W042_DEFERRED_CALLABLE_SEAM_FIELD_LOCK_AND_HIGHER_ORDER_EVIDENCE.md`
   - goal: hold the remaining callable/library-context seam locks until stronger empirical or implementation-facing evidence exists, instead of forcing premature closure from note-only reasoning.
36. `W43` RTD COM activation and topic lifecycle seam:
   - `docs/worksets/W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md`
   - goal: separate `RTD` from the generic provider packet and pin the minimal OxFml/OxFunc seam around ProgID/topic-string shape, host-managed topic lifetime, and external invalidation/value projection.
37. `W44` library-context snapshot export baseline:
   - `docs/worksets/W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md`
   - goal: produce the first honest OxFunc-local export or stable pointer for the external library-context snapshot that OxFml is now asking to consume directly.
38. `W45` non-`@` operator universe closure pass:
   - `docs/worksets/W045_NON_AT_OPERATOR_UNIVERSE_CLOSURE_PASS.md`
   - goal: run one dedicated operator packet for every evaluable non-`@` operator surface across contract, runtime, Lean/formal, empirical Excel validation, replay/test evidence, and library-context export refinement, while leaving `W014` as the dedicated implicit-intersection owner.
39. `W46` `CALL` / `REGISTER.ID` UDF registration seam:
   - `docs/worksets/W046_CALL_AND_REGISTER_ID_UDF_REGISTRATION_SEAM.md`
   - goal: separate worksheet registration/invocation semantics from generic host metadata work and align the future registration path with the library-context snapshot direction.
40. `W47` typed context and query bundle freeze:
   - `docs/worksets/W047_TYPED_CONTEXT_AND_QUERY_BUNDLE_FREEZE.md`
   - goal: lock the first shared typed context/query bundle for the already-covered seam-heavy functions so OxFml can wire them without side channels.
41. `W48` return surface and publication-hint freeze:
   - `docs/worksets/W048_RETURN_SURFACE_AND_PUBLICATION_HINT_FREEZE.md`
   - goal: lock the first shared return-surface split for ordinary values, presentation-aware values, and typed host/provider outcome projection.
42. `W49` runtime library-context provider consumer model:
   - `docs/worksets/W049_RUNTIME_LIBRARY_CONTEXT_PROVIDER_CONSUMER_MODEL.md`
   - goal: turn the agreed runtime `LibraryContextProvider` / immutable `LibraryContextSnapshot` direction into a concrete first-pass consumer/modeling artifact beyond the current CSV interchange pin.

