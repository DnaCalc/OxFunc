# CURRENT_BLOCKERS.md — OxFunc

Status: active blockers recorded.

Last reviewed: 2026-03-21.

---

## Active Blockers

### BLK-FN-003: W023 remaining publication/provider residuals still need richer host seams

- **Status**: active
- **Impact**: `W023` remains partial until the remaining publication/provider seams are pinned honestly for `HYPERLINK` and `IMAGE`.
- **Current state**: `ISFORMULA`, `SUBTOTAL`, and `AGGREGATE` now have typed host-query seams and admitted current-baseline OxFunc closure. The blocker is narrowed to `HYPERLINK` publication metadata/clickability and the richer provider/value/publication model pressure from `IMAGE`.
- **Exact unblock steps**: define the host-facing publication seam for hyperlink decoration/click behavior and either (a) pin `IMAGE` as a host-managed rich value/publication object or (b) explicitly classify it as an external/provider surface above the current OxFunc value boundary.
- **Recommendation**: workaround
- **Opened**: 2026-03-15

## Resolved Blockers

### BLK-FN-008: Odd-bond `ODDL*` parity is not yet closure-grade

- **Status**: resolved
- **Impact**: had blocked `W027` odd-last promotion.
- **Current state**: `W027` replaced the old odd-last discounted-boundary model with the Excel-style normalized quasi-coupon accumulation and the US 30/360 modify-both-dates hack. Native worksheet replay in `.tmp/w27-bond-odd-bond-results.csv` now matches `ODDLPRICE(...)=99.87828601472134` and `ODDLYIELD(...,99.87828601472134,...)=0.04050000000000125`.
- **Exact unblock steps**: none
- **Recommendation**: continue
- **Opened**: 2026-03-18
- **Resolved**: 2026-03-18

### BLK-FN-010: `XNPV` / `XIRR` negative-rate and root-finding parity is reopened by `W29`

- **Status**: resolved
- **Impact**: had blocked `W032` cashflow-rate repair.
- **Current state**: `W32` repaired `XNPV` negative-rate worksheet admission to match direct Excel `#NUM!`, repaired the reopened `XIRR` negative-root lane, and repaired the negative-guess rejection lane for the positive-root-only benchmark case. The previously extracted large-root precision lane is now also repaired and closed by `W037`.
- **Exact unblock steps**: none
- **Recommendation**: continue
- **Opened**: 2026-03-18
- **Resolved**: 2026-03-19

### BLK-FN-011: `XIRR` large positive-root publication mismatch after `W32`

- **Status**: resolved
- **Impact**: had blocked full finance closure after `W32`.
- **Current state**: `W037` characterized Excel's published-result policy on the large positive-root two-cashflow lane directly, replaced the old exact closed-form shortcut with an Excel-like bracket-and-bisection publication solver for the admitted slice, reran the `W29` benchmark, and matched the installed Excel guess matrix exactly.
- **Exact unblock steps**: none
- **Recommendation**: continue
- **Opened**: 2026-03-18
- **Resolved**: 2026-03-21

### BLK-FN-009: `COUPDAYS` leap-year actual/actual parity is reopened by `W29`

- **Status**: resolved
- **Impact**: had blocked `W032` coupon-family repair.
- **Current state**: `W32` repaired `COUPDAYS` on the reopened leap-year actual/actual lane by using the maturity-day nominal previous coupon date for period-size calculation while preserving the aligned `COUPDAYBS` / `COUPDAYSNC` lanes.
- **Exact unblock steps**: none
- **Recommendation**: continue
- **Opened**: 2026-03-18
- **Resolved**: 2026-03-19

### BLK-FN-006: `NUMBERVALUE` default separators and `TRANSLATE` provider behavior do not fit the ordinary pure mega-batch

- **Status**: resolved
- **Impact**: had blocked honest closure claims inside the ordinary mega-batch and then `W030` / `W031`.
- **Current state**: `W30` and `W31` completed as seam-definition/reconciliation packets. `NUMBERVALUE` now moves to `W035`, and `TRANSLATE` now moves to `W036`.
- **Exact unblock steps**: none inside `W030` / `W031`; successor worksets now own the residual function work.
- **Recommendation**: continue
- **Opened**: 2026-03-18
- **Resolved**: 2026-03-19

### BLK-FN-005: `ASC` / `DBCS` / `JIS` are host-profile-sensitive rather than ordinary pure text functions

- **Status**: resolved
- **Impact**: had blocked honest closure claims inside the ordinary mega-batch and then `W030`.
- **Current state**: `W30` completed as a seam-definition/reconciliation packet. `ASC`, `DBCS`, and `JIS` now move to `W034`.
- **Exact unblock steps**: none inside `W030`; successor workset `W034` now owns the residual function work.
- **Recommendation**: continue
- **Opened**: 2026-03-18
- **Resolved**: 2026-03-19

### BLK-FN-007: Bond core basis-`1` parity is not yet closure-grade for `PRICEMAT` / `YIELDMAT`

- **Status**: resolved
- **Impact**: had blocked `W027` bond-core promotion.
- **Current state**: `W027` corrected `PRICEMAT` / `YIELDMAT` to use the Excel-style `DaysInYear(issue,settlement)` denominator on the admitted maturity-security slice. Native worksheet replay in `.tmp/w27-bond-odd-bond-results.csv` now matches `PRICEMAT(...)=98.59811340546048` and `YIELDMAT(...)=0.06100000000000056`.
- **Exact unblock steps**: none
- **Recommendation**: continue
- **Opened**: 2026-03-18
- **Resolved**: 2026-03-18

### BLK-FN-004: W021 first live OxFunc replay-adapter run is blocked by missing adapter implementation and runner surfaces

- **Status**: resolved
- **Impact**: had blocked `W021` from producing any exercised proving artifact for `cap.C0.ingest_valid` through `cap.C3.explain_valid`.
- **Current state**: `tools/replay-adapter/run-w15-replay-adapter-baseline.ps1` now emits the first live local W15 replay bundle under `.tmp/replay-bundles/oxfunc-w15-v1/`, validates the required layout/fields, replays the row views deterministically, and emits diff/explain artifacts recorded in `docs/function-lane/W21_EXECUTION_RECORD.md`.
- **Exact unblock steps**: none
- **Recommendation**: continue from the emitted W15 bundle toward reduced-witness and external replay-host evidence
- **Opened**: 2026-03-16
- **Resolved**: 2026-03-17

### BLK-FN-002: Existing `text_scalar_misc` full-suite failures block clean packet-wide cargo test runs

- **Status**: resolved
- **Impact**: had qualified packet-wide verification hygiene for `W016`; until resolved, only targeted family verification could be claimed.
- **Current state**: W16 Batch 31 promoted the existing `text_scalar_misc` family (`CHAR`,`CODE`,`LOWER`,`UPPER`,`TRIM`,`REPT`) into the runtime/export/formal surface and aligned the stale unit-test expectations with the adapter's explicit domain-error contract. Full `cargo test --manifest-path crates/oxfunc_core/Cargo.toml` now passes cleanly.
- **Exact unblock steps**: none
- **Recommendation**: remove the packet-wide verification qualification and continue W16 family expansion
- **Opened**: 2026-03-15
- **Resolved**: 2026-03-15

### BLK-FN-001: W15 upstream typed host-query seam acknowledgment

- **Status**: resolved
- **Impact**: had blocked a full W015 completion claim and kept `IP-08` open even though the local current-baseline `CELL` / `INFO` semantics were replay-clean.
- **Current state**: OxFunc has local typed seam/runtime/formal/evidence closure for the admitted `CELL` / `INFO` slice, including dual-run (`default` + `compat_template`) native probes and dual-run generated XLL bridge parity. OxFml has now acknowledged `HO-FN-002` in both `docs/upstream/NOTES_FOR_OXFUNC.md` and `docs/handoffs/HANDOFF_REGISTER.csv`.
- **Exact unblock steps**: none
- **Recommendation**: close W015 locally and remove `IP-08` from the in-progress register.
- **Opened**: 2026-03-15
- **Resolved**: 2026-03-15

---

## Entry Template

```
### BLK-FN-NNN: <title>

- **Status**: active | resolved | closed
- **Impact**: <which worksets/features are blocked>
- **Current state**: <what has been attempted, what failed>
- **Exact unblock steps**: <specific actions needed>
- **Recommendation**: wait | escalate | workaround
- **Opened**: YYYY-MM-DD
- **Resolved**: YYYY-MM-DD (if applicable)
```
