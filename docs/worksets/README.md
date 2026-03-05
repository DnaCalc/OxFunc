# OxFunc Worksets

Worksets are sequence-based execution packets for cross-cutting OxFunc slices.

Primary kickoff orchestration:
1. `WORKSET_TUX1000_KICKOFF_PROGRAM_W1_W6.md`

Kickoff worksets:
1. `WORKSET_TUX1000_PI_END_TO_END_SLICE.md` (W1)
2. `WORKSET_TUX1000_FLOATING_POINT_CHARACTERIZATION.md` (W2)
3. `WORKSET_TUX1000_VALUE_UNIVERSE_AND_EXTENDED_TYPES.md` (W3)
4. `WORKSET_TUX1000_COERCION_AND_REFERENCE_RESOLUTION_PRIMITIVES.md` (W4)
5. `WORKSET_TUX1000_ABS_FULL_FORMALITY.md` (W5)
6. `WORKSET_TUX1000_XMATCH_DETERMINISTIC_QUIRKS.md` (W6)
7. `WORKSET_TUX1000_STRING_CHARACTERIZATION.md` (W7)

Common rules:
1. Worksets are sequence/gate driven, never date driven.
2. Each workset must declare dependencies, deliverables, and gate criteria.
3. Completion is recorded by gate closure and explicit status updates.
4. Claim confidence (`draft/provisional/validated`) and assurance maturity (`exercised/green-validated`) must be stated separately.
