# WORKSET - Locale/Format Seam Ownership Realignment (W082)

## 1. Purpose
Own the exact-shape OxFunc-side realignment for locale/format handling so
OxFunc keeps function semantics plus the typed seam contract, while OxFml/FEC
owns the actual parser/formatter implementation supplied into that seam. This
packet explicitly rejects bootstrap fallback or backward-compatible dual
ownership.

## 2. Why This Packet Exists
The current `LocaleFormatContext` seam shape is directionally correct, but the
runtime decomposition is not yet in the intended state:
1. locale-sensitive OxFunc functions already call through a typed
   `LocaleFormatContext`,
2. but the context constructors shipped in OxFunc currently point at local
   OxFunc parser/formatter implementations,
3. that leaves format-language ownership split awkwardly across repos and makes
   it too easy for OxFml to omit the seam support entirely and receive
   `#VALUE!` rather than an honest capability requirement,
4. the intended program shape is simpler: OxFunc owns function semantics and a
   typed capability seam, while OxFml/FEC owns format-language behavior and
   provides the concrete parser/formatter implementation.

## 3. Provenance
1. user direction on 2026-04-10
2. `crates/oxfunc_core/src/locale_format.rs`
3. `crates/oxfunc_core/src/functions/text_fn.rs`
4. `crates/oxfunc_core/src/functions/value_fn.rs`
5. `docs/function-lane/LOCALE_FORMAT_SEAM_EXECUTION_RECORD.md`
6. `docs/function-lane/LOCALE_AND_FORMAT_INTERFACE_OPTIONS.md`
7. `tools/xll-addin/oxfunc_xll/src/lib.rs`
8. `../OxFml/crates/oxfml_core/tests/evaluator_tests.rs`
9. `../OxFml/docs/handoffs/HO-FN-009_LOCALE_FORMAT_SEAM_OWNERSHIP_REALIGNMENT_ACK.md`
10. `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`

## 4. Scope
In scope:
1. move OxFunc to the exact intended decomposition with no backward-compatible
   bootstrap runtime path,
2. keep the typed locale/format seam in OxFunc, but remove OxFunc-owned
   production parser/formatter implementation from the ordinary runtime path,
3. require production callers to supply the parser/formatter capability bundle
   through `LocaleFormatContext`,
4. document the affected OxFunc-local function surfaces and runtime callers,
5. file a detailed OxFunc -> OxFml handoff that states exactly what OxFunc will
   require from OxFml/FEC and from OxFml-side callers/users of the OxFunc
   surface.

Out of scope:
1. broad locale/version sweep work beyond the seam-ownership correction,
2. expanding the admitted Excel format-code language beyond the currently
   replayed local slice,
3. keeping any transitional or fallback OxFunc-owned production formatter path,
4. silently reclassifying missing capability support as an OxFunc local
   function bug.

## 5. Initial Epic Lanes
1. current seam inventory and affected-surface enumeration
2. OxFunc runtime ownership removal and exact target-state implementation
3. local test and host caller alignment
4. downstream OxFml/FEC handoff and caller guidance
5. truth-surface reconciliation

## 6. Closure Condition
`W082` is complete for declared scope only when:
1. OxFunc no longer ships or relies on an OxFunc-owned production
   parser/formatter implementation for the locale-format seam,
2. the OxFunc runtime keeps only the typed seam and function-semantic
   orchestration on that boundary,
3. the affected locale-sensitive functions consume caller-supplied
   `LocaleFormatContext` only,
4. no backward-compatible runtime fallback remains,
5. the detailed downstream handoff is filed and the required OxFml/FEC support
   is stated unambiguously.

## 7. Current Reading
1. execution_state: `closed`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_complete`
4. integration_completeness: `integrated`
5. open_lanes:
   - none for the declared `W082` seam-ownership scope
6. closed lanes:
   - OxFunc has removed the old ordinary convenience constructors; the local
     parser/formatter implementation is now `#[cfg(test)]` explicit test-only
     support rather than an ordinary runtime fallback
   - the XLL add-in no longer imports an OxFunc-core convenience constructor;
     it now supplies a caller-owned locale-format capability bundle on the host
     side and delegates parse/render work to Excel through `xlfEvaluate`
   - the OxFunc-local OxFml seam integration test now uses the OxFml-owned
     `current_excel_host_context()` capability; any remaining OxFml evaluator
     or test helper migration is external to this repo
   - downstream OxFml/FEC acknowledgment is recorded in
     `../OxFml/docs/handoffs/HO-FN-009_LOCALE_FORMAT_SEAM_OWNERSHIP_REALIGNMENT_ACK.md`
     and `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md`
   - OxFml names its concrete implementation owner and call paths requiring
     `LocaleFormatContext`, assigns downstream caller migration ownership, and
     requests no OxFunc-side seam change
7. latest local evidence:
   - `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib locale_format -- --nocapture`
     passed on 2026-04-29
   - `cargo check --manifest-path tools/xll-addin/oxfunc_xll/Cargo.toml`
     passed on 2026-04-29 after XLL caller-side capability alignment
8. orthogonal lanes:
   - broader locale/version sweeps and full Excel format-code-language
     expansion remain separate validation work, not open `W082` lanes
