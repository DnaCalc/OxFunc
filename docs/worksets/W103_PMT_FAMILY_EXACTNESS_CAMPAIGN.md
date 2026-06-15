# W103 PMT Family Exactness Campaign

Status: `planned`

## Purpose

Queue the PMT-family exactness campaign after the W102A checkpoint is reviewable
and the W102B evidence lanes needed by the finance witnesses are visible.

This workset does not claim PMT exactness repair. It records the next lane and
the evidence needed before stronger status language is allowed.

## Canonical Surfaces

1. `.beads/` task `oxf-acdw.4`
2. `docs/bugs/streams/BUG-FUNC-034_ipmt_ppmt_type_one_interest_omits_beginning_payment.md`
3. `crates/oxfunc_core/src/functions/financial_time_value_family.rs`
4. `crates/oxfunc_core/src/functions/cumulative_finance_family.rs`
5. `docs/KNOWN_EXACTNESS_DEVIATIONS.md`

## Entry Conditions

1. W102A working-tree checkpoint is reviewed and either landed or intentionally
   split.
2. W102B has linked or refreshed the live Excel evidence needed for
   `BUG-FUNC-034`.
3. Existing wrong-bit PMT/IPMT/PPMT pins and known exactness deviations are
   identified before changing publication targets.

## Initial Evidence Needs

1. Candidate formulation fingerprinting in the production Rust/toolchain path.
2. Live Excel or durable artifact evidence for the finance witnesses being
   promoted.
3. Targeted tests for PMT, IPMT, PPMT, CUMIPMT, CUMPRINC, FV, PV, EFFECT, and
   adjacent solver/payment control lanes.
4. Updated bug stream/report/workset records for any promoted mismatch class.

## Doctrine Axes

scope_completeness: `scope_partial`
target_completeness: `target_partial`
integration_completeness: `partial`
open_lanes: `[W102A_checkpoint_dependency, BUG-FUNC-034_probe_dependency, PMT_candidate_fingerprint, PMT_family_validation_matrix]`
