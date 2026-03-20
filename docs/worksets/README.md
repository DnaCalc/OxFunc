# OxFunc Worksets

Worksets are sequence-based execution packets for cross-cutting OxFunc slices.

Primary kickoff orchestration:
1. `W000_KICKOFF_PROGRAM_W001_W006.md`

Kickoff worksets:
1. `W001_PI_END_TO_END_SLICE.md` (W1)
2. `W002_FLOATING_POINT_CHARACTERIZATION.md` (W2)
3. `W003_VALUE_UNIVERSE_AND_EXTENDED_TYPES.md` (W3)
4. `W004_COERCION_AND_REFERENCE_RESOLUTION_PRIMITIVES.md` (W4)
5. `W005_ABS_FULL_FORMALITY.md` (W5)
6. `W006_XMATCH_DETERMINISTIC_QUIRKS.md` (W6)
7. `W007_STRING_CHARACTERIZATION.md` (W7)
8. `W008_STRING_W8_1_CHECKLIST.md` (W8.1)
9. `W009_XLL_ADDIN_BRIDGE.md` (W9)
10. `W010_TEN_FUNCTION_MIXED_SEAMS.md` (W10)
11. `W011_XLL_REGISTRATION_FLAGS_EVIDENCE.md` (W11)
12. `W012_MODERATE_FUNCTION_EXPANSION.md` (W12)
13. `W013_DECEPTIVELY_SIMPLE_BOUNDARY_FUNCTIONS.md` (W13)
14. `W014_IMPLICIT_INTERSECTION_OPERATOR.md` (W14)
15. `W015_CELL_AND_INFO_HOST_QUERY_FUNCTIONS.md` (W15)
16. `W016_BULK_NON_INTERESTING_FUNCTIONS_AND_OPERATORS.md` (W16)
17. `W017_DEFERRED_LOW_INTEREST_FUNCTIONS_REQUIRING_HARDENING_AND_HOST_SEAMS.md` (W17)
18. `W018_REPLAY_APPLIANCE_PACKET_ADAPTER_BASELINE.md` (W18)
19. `W019_PACKET_WITNESS_DISTILLATION_AND_RETENTION_BASELINE.md` (W19)
20. `W020_OXFUNC_REPLAY_BUNDLE_LAYOUT_AND_INDEX_BASELINE.md` (W20)
21. `W021_W15_FIRST_LIVE_REPLAY_ADAPTER_RUN_BASELINE.md` (W21)
22. `W022_CRITERIA_FAMILY_SHAPE_HARDENING.md` (W22)
23. `W023_DEFERRED_HOST_METADATA_AND_DATABASE_FUNCTIONS.md` (W23)
24. `W024_ORDINARY_FUNCTIONS_MEGA_BATCH_EXECUTION_PLAN.md` (W24)
25. `W025_DEFERRED_MISC_ADDIN_AND_DYNAMIC_ARRAY_OUTLIERS.md` (W25)
26. `W026_DEFERRED_LOCALE_PROFILE_AND_PROVIDER_SENSITIVE_ORDINARY_OUTLIERS.md` (W26)
27. `W027_DEFERRED_ADVANCED_BOND_AND_ODD_BOND_HARDENING.md` (W27)
28. `W028_FUNCTION_NAME_LOCALIZATION_LIBRARY_DISCOVERY.md` (W28)
29. `W029_FINANCE_FUNCTIONS_FSHARP_BENCHMARK_CROSSCHECK.md` (W29)
30. `W030_DEFERRED_LOCALE_PROFILE_SENSITIVE_TEXT_AND_NUMBER_FUNCTIONS.md` (W30)
31. `W031_DEFERRED_PROVIDER_LANGUAGE_FUNCTIONS.md` (W31)
32. `W032_REOPENED_FINANCE_PARITY_GAPS_FROM_BENCHMARK.md` (W32)
33. `W033_INFORMATION_PREDICATES_AND_FORECAST_COMPATIBILITY_CLOSURE.md` (W33)
34. `W034_DEFERRED_WIDTH_CONVERSION_HOST_PROFILE_CAPABILITY_BASELINE.md` (W34)
35. `W035_DEFERRED_NUMBERVALUE_LOCALE_DEFAULT_PROFILE_BASELINE.md` (W35)
36. `W036_DEFERRED_PROVIDER_LANGUAGE_CAPABILITY_BASELINE.md` (W36)
37. `W037_REOPENED_XIRR_LARGE_ROOT_SOLVER_PRECISION.md` (W37)
38. `W038_FUNCTIONAL_LAMBDA_AND_HELPER_FAMILY.md` (W38)
39. `W039_DYNAMIC_ARRAY_SHAPING_AND_RESHAPING_FAMILY.md` (W39)
40. `W040_REFERENCE_METADATA_AND_FORMULA_VISIBILITY_FUNCTIONS.md` (W40)
41. `W041_EXTERNAL_DATA_PROVIDER_AND_CUBE_FUNCTIONS.md` (W41)
42. `W042_DEFERRED_CALLABLE_SEAM_FIELD_LOCK_AND_HIGHER_ORDER_EVIDENCE.md` (W42)
43. `W043_RTD_COM_ACTIVATION_AND_TOPIC_LIFECYCLE_SEAM.md` (W43)
44. `W044_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_BASELINE.md` (W44)
45. `W045_NON_AT_OPERATOR_UNIVERSE_CLOSURE_PASS.md` (W45)

Common rules:
1. Worksets are sequence/gate driven, never date driven.
2. Each workset must declare dependencies, deliverables, and gate criteria.
3. Completion is recorded by gate closure and explicit status updates.
4. Claim confidence (`draft/provisional/validated`) and assurance maturity (`exercised/green-validated`) must be stated separately.

Mega-batch planning note:
1. If native replay shows that a row is host/profile-sensitive, provider-bound, or absent on the current host surface, extract it early to a successor packet instead of carrying it as an ordinary mega-batch member.
2. For advanced finance families, add at least one direct Excel-valued parity row early; internally consistent local tests are not enough by themselves.
3. Mega-batch packets should define the reconciliation rule up front: every row must end as either `done` or `extracted`, with no silent residual state.

Process references:
- Pre-closure checklist: `OPERATIONS.md` Section 12.
- Completion claim self-audit: `OPERATIONS.md` Section 14.
- Active blockers: `CURRENT_BLOCKERS.md`.
- In-progress feature register: `docs/IN_PROGRESS_FEATURE_WORKLIST.md`.

Replay rollout sequence after `W016`:
1. `W017` is now reconciled and remains as provenance only; active successor ownership moved to `W023`, `W026`, and `W027`, while `W025` has since closed as a classification packet.
2. `W018` establishes the OxFunc Replay appliance packet adapter baseline on top of the current packet/evidence discipline.
3. `W019` then defines packet-first witness distillation, lifecycle, retention, and quarantine policy without overclaiming pack-grade support.
4. `W020` defines and now has locally exercised the first emitted OxFunc replay bundle layout and index target on disk.
5. `W021` emits the first live `W15` replay-adapter bundle against that declared target and records the result in `W21_EXECUTION_RECORD.md`.
6. `W024` is now reconciled: ordinary rows are either closed in `W24` or extracted to successor handling; `W025`, `W026`, and `W027` are now closed for their declared scopes.
7. `W028` is complete for its declared discovery-and-library-seed scope.
8. `W029` is complete as a benchmark-and-classification packet; `W032` repaired the reopened finance packet and extracted the remaining large-root `XIRR` precision lane to `W037`.
9. `W030` and `W031` are now complete as seam-definition/reconciliation packets; successor ownership moved to `W034`, `W035`, and `W036`.
10. `W044` is now in progress with a first real downstream export artifact in `docs/function-lane/OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`; further refinement remains open.
11. `W045` is now complete for its declared scope: the full current non-`@` evaluable operator universe is covered across contract, runtime, Lean/formal, empirical Excel validation, and library-context export refinement, leaving `W014` as the dedicated `@` packet.


