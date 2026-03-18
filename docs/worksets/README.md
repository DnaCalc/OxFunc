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
1. `W017` is now reconciled and remains as provenance only; active successor ownership moved to `W023`, `W025`, `W026`, and `W027`.
2. `W018` establishes the OxFunc Replay appliance packet adapter baseline on top of the current packet/evidence discipline.
3. `W019` then defines packet-first witness distillation, lifecycle, retention, and quarantine policy without overclaiming pack-grade support.
4. `W020` defines and now has locally exercised the first emitted OxFunc replay bundle layout and index target on disk.
5. `W021` emits the first live `W15` replay-adapter bundle against that declared target and records the result in `W21_EXECUTION_RECORD.md`.
6. `W024` is now reconciled: ordinary rows are either closed in `W24` or extracted to `W025` / `W026` / `W027`.


