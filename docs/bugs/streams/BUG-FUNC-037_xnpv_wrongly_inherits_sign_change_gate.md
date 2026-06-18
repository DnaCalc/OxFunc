# BUG-FUNC-037: XNPV wrongly inherits IRR/XIRR sign-change gate

## Summary
- **Bug id**: `BUG-FUNC-037`
- **Opened**: `2026-06-11`
- **Status**: `closed` (fix landed in `7a0003f` on main; live Excel 16.0 b20026 bit-exact verified 2026-06-18)
- **Owner workset**: `W102A`

## Live Excel Verification (2026-06-18)
`XNPV(0.1,{100,200,300},{43831,44196,44561})` = `529.7520661157024`, f64 bits
`0x40808e043b3d5af9` on **both** OxFunc and Excel 16.0 build 20026 — bit-exact, and
it computes (no `#NUM!`), confirming the sign-change precondition was correctly
removed for pure discounting. PMT/IPMT/XIRR keep their sign-change requirement.

## Source Refs
- **Reported against ref**: `caffd0f`
- **Reproduced on ref**: `caffd0f`
- **Introduced in ref**: `unknown` (initial XNPV implementation)
- **Fixed in ref**: `not yet fixed` (working-tree patch present; checkpoint not landed)
- **Ref notes**: Live Excel 16.0 build 20026 probe confirmed during 2026-06-10 review verification pass.

## Ownership And Root Cause
- **Ownership class**: `OxFunc-owned bug`
- **Root cause class**: `initial_impl_gap`
- **Root cause summary**: `xnpv_kernel_raw` delegated its input validation to
  `validate_xcashflow_inputs`, which calls `validate_cashflows` and requires
  (a) at least two cashflows and (b) at least one positive and one negative value.
  Those constraints are correct preconditions for IRR/XIRR root-finding (a sign
  change is necessary for a real root to exist) but are wrong for XNPV, which is
  pure discounting and has no such requirement.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: `no` — Microsoft's XNPV docs
  contain the boilerplate sentence "values must contain at least one positive and
  one negative value" in the argument prose, but that sentence does not appear
  among the documented error conditions, and live Excel does not enforce it.
- **Spec vague or missing?**: `yes` — the documentation prose is misleading.
- **Code once correct and later regressed?**: `no`
- **Likely introduced in ref**: `unknown` (initial XNPV implementation shared the
  XIRR validation path without checking whether the sign-change constraint applies)
- **Explanation**: The shared `validate_xcashflow_inputs` function was written to
  serve XIRR's root-finding semantics, and `xnpv_kernel_raw` reused it without
  auditing which constraints are intrinsic to XNPV vs inherited from the solver.

## Reproduction

| Formula | OxFunc (before fix) | Excel 16.0 |
|---|---|---|
| `=XNPV(0.1,{100,200},{45000,45100})` | `#NUM!` | `294.8451203808644` |
| `=XNPV(0.1,{500},{45000})` | `#NUM!` | `500` |
| `=XNPV(0.1,{-100,-110},{45000,45365})` | `#NUM!` | `−200` |

All three cases returned `#NUM!` before the fix because the sign-change gate fired.
The length-mismatch and pre-anchor-date lanes were already correct.

## Fix Plan
Added `validate_xnpv_inputs` alongside `validate_xcashflow_inputs` in
`crates/oxfunc_core/src/functions/cashflow_rate_family.rs`. The new function
checks: equal-length arrays, non-empty, all values finite, no date before anchor.
It does **not** check for sign change or minimum length of two.

`xnpv_kernel_raw` and `xnpv_derivative` now call `validate_xnpv_inputs`.
`validate_xcashflow_inputs` is unchanged and still used by `xirr_kernel`.

Kept intact (per spec and live-Excel confirmation):
- Surface-level negative-rate rejection in `xnpv_kernel` (lines ~471–474).
- All IRR/XIRR validation paths.
- Length-mismatch `#NUM!`.
- Pre-anchor-date `#NUM!`.

## Validation

Witness tests added to the `tests` module in `cashflow_rate_family.rs`:

- `xnpv_live_excel_probe_all_positive_two_cashflows`: asserts bit-exact
  `294.8451203808644_f64` for the live-probed case. The value was independently
  recomputed from the discounting formula (sum of `v / (1+r)^((d-d₀)/365)`) and
  matches the probed digits to all 16 significant figures.
- `xnpv_single_element_equals_anchor_cashflow`: single-element array at the
  anchor date equals the cashflow value (rate has no effect at t=0).
- `xnpv_all_negative_series_accepted`: all-negative series accepted; result
  verified against the analytic formula.

`cargo test -p oxfunc_core cashflow`: **22 passed, 0 failed**.
`xirr_requires_sign_change` still passes — XIRR validation is unchanged.

## Similar-Risk Scan
- `xnpv_derivative` also called `validate_xcashflow_inputs`; updated in the same
  commit. `xnpv_derivative` is only reachable from `xirr_kernel` (XIRR solver
  uses the XNPV derivative), so this was a latent issue that would have surfaced
  if XIRR ever attempted to evaluate the derivative for a one-sided series.
- `periodic_npv_with_t0` (used by IRR) and `irr_kernel` call `validate_cashflows`
  directly and correctly require mixed signs. No change needed.

## Closure Checklist
- [ ] fix landed or non-OxFunc ownership recorded (working-tree patch present; checkpoint not landed)
- [x] validation recorded
- [x] root cause recorded
- [x] similar-risk scan recorded
- [ ] spec/matrix/contract updated if required
- [ ] handoff filed if required
- [x] linked reports updated
