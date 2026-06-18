# W102 Structural Fix Batch And Probe Queue

Status: `in_progress`

## Purpose

Track the June 2026 review-derived structural cleanup batch and split it into:

1. `W102A`: no-probe or already-evidenced structural cleanup currently present
   in the working tree.
2. `W102B`: probe-first evidence lanes that must not be promoted on local
   inference alone.

`.beads/` owns live readiness and blockers.

## Canonical Surfaces

1. `.beads/` task `oxf-acdw.2` for `W102A`
2. `.beads/` task `oxf-acdw.3` for `W102B`
3. `docs/bugs/BUG_STREAM_REGISTER.csv`
4. `docs/bugs/BUG_REPORT_REGISTER.csv`
5. `docs/bugs/streams/BUG-FUNC-034_ipmt_ppmt_type_one_interest_omits_beginning_payment.md`
6. `docs/bugs/streams/BUG-FUNC-036_reference_provider_ref_wrapper_drops_capabilities.md`
7. `docs/bugs/streams/BUG-FUNC-037_xnpv_wrongly_inherits_sign_change_gate.md`
8. `docs/bugs/streams/BUG-FUNC-038_npv_fv_pv_negative_base_rate_lanes.md`
9. `docs/bugs/streams/BUG-FUNC-039_statistical_and_boundary_edge_batch.md`
10. `docs/bugs/streams/BUG-FUNC-040_approximate_match_blank_skipping_gap.md`
11. `docs/bugs/streams/BUG-FUNC-041_regex_silent_escape_fallthrough_and_dead_translate_phrasebook.md`

## Current Checkpoint

2026-06-18 (land + close pass):

1. The W102A code checkpoint (`7a0003f`) plus W100 and W104 were fast-forwarded onto
   `main` (`ee86681`).
2. Excel automation confirmed available locally (Excel 16.0 build 20026); the W102B
   evidence gate is now openable.
3. Live Excel verification of the W102A streams (OxFunc local value surface vs Excel):
   - `BUG-FUNC-034` (IPMT/PPMT/CUMIPMT/CUMPRINC type=1) — 4/4 match to 10dp → **closed**.
   - `BUG-FUNC-038` (NPV/FV/PV negative base; PMT-rejects contrast) — match → **closed**.
   - `BUG-FUNC-036` (resolver `&T` capabilities) — static fix, lib suite green → **closed**.
   - `BUG-FUNC-039` — MROUND/GAMMA.INV(p=1)/CONFIDENCE/CHISQ.DIST match, **but
     `GAMMA.INV(0,..)` returns `#NUM!` where Excel returns `0` — regression**, bead
     `oxf-99zz`; stream stays **open**.
   - `BUG-FUNC-037` (XNPV no-sign-change) — bit-exact vs Excel (`0x40808e043b3d5af9`) → **closed**.
   - `BUG-FUNC-040` (lookup blank-skip) — MATCH over range with a blank matches Excel → **closed**.
   - `BUG-FUNC-041` (regex escapes) — a 40-escape Excel battery found the fix **over-rejects**
     18 escapes Excel admits (anchors, whitespace, escaped metacharacters); bead `oxf-fyhi`,
     stream stays **open**.

This pass surfaced two over-correction regressions in the W102A batch — `GAMMA.INV(0)`
(`oxf-99zz`) and the regex admitted slice (`oxf-fyhi`) — both "the fix rejects what Excel
accepts." Verification against live Excel before closure is what caught them.

2026-06-15:

1. `BUG-FUNC-034`, `036`, `037`, `038`, `039`, `040`, and `041` are registered
   in the bug stream register.
2. `BUGREP-FUNC-020` through `BUGREP-FUNC-026` are registered in the report
   register.
3. `BUG-FUNC-039` now has a stream record rather than only source comments.
4. W102A stream wording has been normalized to `fix_in_progress` and
   `not yet fixed` for unlanded working-tree patch state.
5. W102B evidence ownership is explicit for live bit pins, durable probe
   artifacts, lookup error/cross-type lanes, and the regex admitted-escape
   contract update.
6. `BUG-FUNC-035` remains an unassigned numbering gap with no references found.

## Validation Evidence

1. `cargo fmt --check` passes.
2. `git diff --check` reported no whitespace errors before the latest workset
   packet additions.
3. `cargo test -p oxfunc_core --lib` passed earlier in the checkpoint with
   `1398 passed; 0 failed; 1 ignored`.
4. Full `cargo test -p oxfunc_core` after the local W100 `INDEX` fix now has
   core lib passing `1399 passed; 0 failed; 1 ignored`; remaining package
   failure is the W100 explicit-`@` parser blocker.

## Open W102A Lanes

1. Review/land or intentionally split the current dirty working tree.
2. Keep W102A status at `fix_in_progress` until the checkpoint lands or is
   intentionally split.

## Open W102B Lanes

1. `BUG-FUNC-034`: live Excel bit pins for type=1 finance witnesses.
2. `BUG-FUNC-038`: link or refresh the durable negative-base live probe matrix.
3. `BUG-FUNC-040`: probe error-cell and cross-type approximate lookup lanes.
4. `BUG-FUNC-041`: update the OxFunc-local admitted regex escape contract and
   file a cross-repo handoff only if W102B evidence requires one.
5. `GAMMA.INV`: tail/cap behavior remains split from W102A and needs live
   evidence and/or a numeric exactness repair lane before promotion.

## Doctrine Axes

scope_completeness: `scope_partial`
target_completeness: `target_partial`
integration_completeness: `partial`
open_lanes: `[W102A_land_or_split, BUG-FUNC-034_bit_pins, BUG-FUNC-038_probe_artifact, BUG-FUNC-040_probe_matrix, BUG-FUNC-041_contract_update, GAMMA_INV_tail_cap_evidence]`
