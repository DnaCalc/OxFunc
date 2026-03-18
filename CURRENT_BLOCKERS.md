# CURRENT_BLOCKERS.md — OxFunc

Status: active blockers recorded.

Last reviewed: 2026-03-18.

---

## Active Blockers

### BLK-FN-008: Odd-bond `ODDL*` parity is not yet closure-grade

- **Status**: active
- **Impact**: `W027` now owns the extracted odd-bond family rows and cannot honestly close them yet.
- **Current state**: native Excel replay on `2026-03-18` confirmed that the bounded odd-first sample still matches for `ODDFPRICE` / `ODDFYIELD`, but the local pinned `ODDL*` baseline is wrong. Native Excel returned `ODDLPRICE(DATE(2008,2,7),DATE(2008,6,15),DATE(2007,10,15),0.0375,0.0405,100,2,0) = 99.87828601472134`, while the local note and kernel test were still pinned to `99.8948395136953`. The corresponding inversion lane also differs: native `ODDLYIELD(...,99.8948395136953,...) = 0.04003268920721975`, not the pinned `0.0405`.
- **Exact unblock steps**: characterize the native odd-last coupon schedule and accrued-interest convention for the admitted `ODDL*` slice, patch the local `ODDLPRICE` / `ODDLYIELD` kernels and tests, then add a native worksheet packet before promoting the family.
- **Recommendation**: workaround
- **Opened**: 2026-03-18

### BLK-FN-007: Bond core basis-`1` parity is not yet closure-grade for `PRICEMAT` / `YIELDMAT`

- **Status**: active
- **Impact**: `W027` now owns the extracted advanced bond family rows and cannot honestly close them yet.
- **Current state**: direct native Excel parity probing on `2026-03-18` showed that the local bond core family is not yet closure-grade on at least the basis-`1` maturity-security lane. Native Excel returned `PRICEMAT(DATE(2024,6,15),DATE(2025,12,31),DATE(2024,1,1),0.0525,0.061,1) = 98.59811340546048`, while the local kernel produced `98.59584793189372`. The corresponding `YIELDMAT` parity lane also remains suspect until the same day-count / maturity-security convention gap is resolved. Existing internal round-trip tests were insufficient because they did not pin direct Excel values.
- **Exact unblock steps**: characterize the precise Excel basis-`1` convention for the maturity-security formulas, patch the shared day-count or direct algebra path as needed, add explicit Excel-valued parity tests and a native worksheet packet, then reopen bond core promotion under `W027`.
- **Recommendation**: workaround
- **Opened**: 2026-03-18

### BLK-FN-005: `ASC` / `DBCS` / `JIS` are host-profile-sensitive rather than ordinary pure text functions

- **Status**: active
- **Impact**: `W026` now owns the locale-width conversion family after extraction from `W024`; the family no longer fits the pure mega-batch assumptions after native Excel replay on this machine.
- **Current state**: the existing Rust kernel in `crates/oxfunc_core/src/functions/text_compat_locale_family.rs` assumes Japanese width-conversion semantics. Native Excel replay on `2026-03-18` showed `ASC("ＡＢＣ　１２３")` and `DBCS("ABC ｶﾞ")` as pass-through on the current host/profile, while `JIS(...)` returned `#NAME?`. That means function availability and conversion behavior are profile-sensitive at the host/locale layer, not fixed pure semantics.
- **Exact unblock steps**: reclassify the family out of `W024`; define a host/profile-aware seam or version/profile matrix for width-conversion availability and behavior; then reopen the family under a dedicated locale/profile workset.
- **Recommendation**: workaround
- **Opened**: 2026-03-18

### BLK-FN-006: `NUMBERVALUE` default separators and `TRANSLATE` provider behavior do not fit the ordinary pure mega-batch

- **Status**: active
- **Impact**: `W026` now owns `NUMBERVALUE` and `TRANSLATE` after extraction from `W024`; the current assumptions are not honest enough for closure.
- **Current state**: native Excel replay on `2026-03-18` showed `NUMBERVALUE("1,234.5%") -> #VALUE!` on this host/profile while explicit separator lanes still work, so omitted separator defaults are locale-profile-sensitive. The same replay showed `TRANSLATE("hello","en","es") -> #BUSY!` while same-language `TRANSLATE("hola","es","es") -> "hola"`, confirming an external-provider seam rather than a pure local function. The combined W16 Batch 75 note therefore overstates what belongs in the ordinary packet.
- **Exact unblock steps**: characterize `NUMBERVALUE` omitted-default semantics in a locale/profile-aware packet and characterize `TRANSLATE` in a provider-aware packet, both under `W026`.
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
