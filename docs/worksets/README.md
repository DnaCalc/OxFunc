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

Common rules:
1. Worksets are sequence/gate driven, never date driven.
2. Each workset must declare dependencies, deliverables, and gate criteria.
3. Completion is recorded by gate closure and explicit status updates.
4. Claim confidence (`draft/provisional/validated`) and assurance maturity (`exercised/green-validated`) must be stated separately.

