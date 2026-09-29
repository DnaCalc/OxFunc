# Recorded locale provider binding

The local typed evaluator can optionally use OxFml's existing production
`oxfml_locale_context` parser/formatter through Cargo feature `oxfml-locale`.
The default remains no provider. An explicit named locale and captured profile
record are required; the `CurrentExcelHost` placeholder is not accepted.

The captured Excel 20430/CV2 profile is en-ZA with decimal point, ASCII-space
grouping, currency R, and year/month/day date order. It differs from OxFunc's
canonical en-ZA table (decimal comma and NBSP grouping). Excel's application
separator property and Windows culture also report NBSP grouping, while
`Application.International` and the DOLLAR/FIXED output sentinels use ASCII space.
Both observations are retained unchanged in `live-profile.json`.

Canonical binding rejects the decimal mismatch. The separately requested
`--use-recorded-locale-settings` mode applies the recorded regional fields to the
named en-ZA profile. Before evaluation, it verifies the recorded provider-source
hashes and exact text results for the DOLLAR, FIXED and TEXT formatting sentinels.
`provider-binding.json` records canonical/applied fields, provider identity,
snapshot hash, version axes, and the unresolved provenance qualifications on
the Windows two-digit-year pivot and negative-currency rendering.

```powershell
cargo run -q --manifest-path smart-fuzzer/tools/pmt_ppmt_local_eval/Cargo.toml --features oxfml-locale --bin array_tranche_local_eval -- --cases smart-fuzzer/cache/w111-locale-profile-cases-20260929.jsonl --out smart-fuzzer/cache/w111-locale-profile-local-20260929.jsonl --locale-profile en-ZA --locale-profile-record smart-fuzzer/cache/w111-locale-profile-20260929.json --use-recorded-locale-settings
```

The local outcomes identify their provider/profile/snapshot hash; the adjacent
`.provenance.json` sidecar records the full binding. Locale-backed outcomes must
be compared with fresh Excel captures using the same case set. A default generic
runner's no-provider local outcomes are not locale conformance evidence.

Cross-repository impact: the optional dependency exists only in this local
verification executable. It calls existing public OxFml APIs through existing
OxFunc context traits. No production core dependency, evaluator-facing contract,
canonical locale table, or OxFml source file was changed. The XLL locale provider
is not used because it delegates to Excel, which would make the comparison
circular. The test-only OxFunc parser/formatter is not used. Any discrepancy
inside the production OxFml provider requires separate triage and handoff before
a cross-repository semantic fix or completion claim.

The fresh 358-case capture has 209 raw exact outcomes. Six functions have their
required context supplied: DATEVALUE 17/32, TIMEVALUE 19/28, VALUE 28/31,
DOLLAR 44/67, FIXED 48/75 and TEXT 49/68, giving 205/301 admitted exact comparisons.
All execution statuses are `ok`. The other 57 ASC/DBCS/JIS rows are explicitly
excluded from a semantic verdict because their separate
`HostInfoProvider::query_width_conversion_mode` is absent. JIS also produces
`#NAME?` on this Excel host, which is a name-availability observation.

The missing provider does not explain DATEVALUE/TIMEVALUE discrepancies: those
functions currently call fixed parsers in `date_value_family.rs` rather than the
locale provider. Examples include valid `2026/09/29`, `24:00`, `12:60` and `1 PM`.
VALUE rejects a blank reference and time text, while accepting a trailing currency
symbol rejected by this host. DOLLAR/FIXED reject array arguments and do not round
negative decimal counts as Excel does. Further decimal rounding and TEXT format
differences remain. `comparison.json` preserves every exact typed outcome with
its admission qualification; retained cases, source hashes and provider binding
make these distinctions replayable. These are discovery results, not a function
completion claim or evidence from a substituted test provider.

Status: in progress. `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Open lanes: the 96 admitted mismatches; width-conversion host context and JIS
availability; explicit version/channel and locale expansion; the recorded
profile qualifications described above; independent heldout validation after
semantic fixes.
