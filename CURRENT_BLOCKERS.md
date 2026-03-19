# CURRENT_BLOCKERS.md — OxFunc

Status: active blockers recorded.

Last reviewed: 2026-03-18.

---

## Active Blockers

### BLK-FN-010: `XNPV` / `XIRR` negative-rate and root-finding parity is reopened by `W29`

- **Status**: active
- **Impact**: `W032` now owns reopened cashflow-rate parity repair work; `FDEF-053` and the older `W24` cashflow-rate packet can no longer be treated as closure-grade on those lanes.
- **Current state**: the `W29` three-way benchmark showed `XNPV` negative-rate cases where OxFunc and the public ExcelFinancialFunctions F# library both returned finite values while direct Excel returned `#NUM!`, plus `XIRR` lanes where OxFunc either rejected a negative root that both F# and Excel accept or converged to a large positive root where Excel returned a different result or `#NUM!`.
- **Exact unblock steps**: characterize the negative-rate admissibility and root-selection policy directly against Excel in `W032`, repair the `cashflow_rate_family` solver/admission behavior, and rerun the benchmark ledger until the reopened lanes are reconciled honestly.
- **Recommendation**: workaround
- **Opened**: 2026-03-18

### BLK-FN-009: `COUPDAYS` leap-year actual/actual parity is reopened by `W29`

- **Status**: active
- **Impact**: `W032` now owns reopened coupon-family parity repair work; `FDEF-054` and the older `W24` coupon packet can no longer be treated as closure-grade for `COUPDAYS` on the reopened lane.
- **Current state**: the `W29` three-way benchmark showed a leap-year actual/actual case where OxFunc and the public ExcelFinancialFunctions F# library both returned `182` while direct Excel returned `184`, with the related `COUPDAYBS` and `COUPDAYSNC` lanes still matching Excel on the same case.
- **Exact unblock steps**: characterize Excel's leap-year actual/actual coupon-period sizing rule in `W032`, repair `COUPDAYS` without regressing the aligned coupon-family lanes, and rerun both the direct Excel packet and the benchmark ledger.
- **Recommendation**: workaround
- **Opened**: 2026-03-18

### BLK-FN-005: `ASC` / `DBCS` / `JIS` are host-profile-sensitive rather than ordinary pure text functions

- **Status**: active
- **Impact**: `W030` now owns the locale-width conversion subset after `W026` completed as a characterization-and-extraction packet.
- **Current state**: the existing Rust kernel in `crates/oxfunc_core/src/functions/text_compat_locale_family.rs` assumes Japanese width-conversion semantics. Native Excel replay on `2026-03-18` and the dedicated `W26` packet showed `ASC("ＡＢＣ　１２３")` and `DBCS("ABC ｶﾞ")` as pass-through on the current host/profile, while `JIS(...)` returned `#NAME?`. That means function availability and conversion behavior are profile-sensitive at the host/locale layer, not fixed pure semantics.
- **Exact unblock steps**: define a host/profile-aware seam or version/profile matrix for width-conversion availability and behavior under `W030`, then revisit runtime admission and semantics honestly.
- **Recommendation**: workaround
- **Opened**: 2026-03-18

### BLK-FN-006: `NUMBERVALUE` default separators and `TRANSLATE` provider behavior do not fit the ordinary pure mega-batch

- **Status**: active
- **Impact**: `W030` now owns `NUMBERVALUE` and `W031` now owns `TRANSLATE` after `W026` completed as a characterization-and-extraction packet.
- **Current state**: native Excel replay on `2026-03-18` and the dedicated `W26` packet showed `NUMBERVALUE("1,234.5%") -> #VALUE!` on this host/profile while explicit separator lanes still work, so omitted separator defaults are locale-profile-sensitive. The same replay showed `TRANSLATE("hello","en","es") -> #BUSY!` while same-language `TRANSLATE("hola","es","es") -> "hola"`, confirming an external-provider seam rather than a pure local function.
- **Exact unblock steps**: characterize `NUMBERVALUE` omitted-default semantics in `W030` and characterize `TRANSLATE` in `W031`, then revisit runtime admission and semantics honestly.
- **Recommendation**: workaround
- **Opened**: 2026-03-18

### BLK-FN-003: W023 host-integrated, metadata-query, and visibility-sensitive residuals need new FEC/host seams

- **Status**: active
- **Impact**: `W023` cannot close the host-sensitive and metadata-query subset until the required host/query seams exist. Current known members: `CALL`, `COPILOT`, `DETECTLANGUAGE`, `GETPIVOTDATA`, `HYPERLINK`, `IMAGE`, `PHONETIC`, `REGISTER.ID`, `ISFORMULA`, plus row-visibility-sensitive `SUBTOTAL` and `AGGREGATE`.
- **Current state**: `W16` is closed, `W022` reconciled the criteria family, and these rows are now extracted to `W023`. Core OxFunc can close pure and low-risk spreadsheet semantics, but these lanes still require workbook/application/pivot/link/ui/visibility or cell-formula-metadata facts that are not presently available through the admitted typed seams. No honest in-core pure implementation exists for them on the current boundary.
- **Exact unblock steps**: define typed evaluator/host capabilities for link/pivot/selection/row-visibility/query state in OxFml/FEC, or explicitly reclassify the affected functions out of `W017`.
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
