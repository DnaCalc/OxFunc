# SLN evidence, 2026-09-29

Status: in progress. `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Open lanes: current-phase review/promotion; non-x86 numeric fallback parity;
alternate Excel versions/channels and locale phases.

The candidate agrees exactly with 1,205 numeric discovery, 399 discriminator,
7,250 independent numeric heldout, 39 typed discovery and 265 independent typed
heldout cases. Source inputs in the numeric cohorts are positive zero or normal
finite binary64 values; subnormal and negative-zero Value2 ingress substitutions
are excluded. Output zero/subnormal cases are deliberately included.

The old negative-input guards and epsilon test on life rejected valid inputs.
Excel accepts either sign for cost, salvage and nonzero life. Exact zero life
returns `#DIV/0!`. The observed graph is binary64 subtraction followed by an
extended-precision quotient rounded to binary64. Nonfinite results publish
`#NUM!`; final signed zeros and subnormal results publish positive zero.
Subnormal intermediate differences survive until division and can yield normal
results. An overflowing stored difference is not rescued by a later large life.

For example, `SLN(100,0,-1)` is -100, a life of `1e-13` is accepted, and
`SLN(7,3,MAX_DOUBLE)` publishes the minimum normal value. The last witness
distinguishes the observed quotient from ordinary binary64 division by one ULP.
This black-box distinction selects the existing `excel_x87_div` substrate; it
does not assert knowledge of Excel internals. That substrate has a documented
single-rounded fallback outside x86-64, which this evidence does not validate.

SLN explicitly missing argument positions coerce to zero and typed arrays lift
through the existing broadcast helper. The changes are confined to SLN paths
inside `depreciation_family.rs`; DB/DDB/SYD/VDB arithmetic was untouched by this
candidate. The heldout excludes previously observed exact input tuples and adds
full-exponent random normals plus adjacent cancellation cases.

Reference baseline: Excel 16.0 build 20430, Windows 64-bit, workbook Compatibility
Version 2. Channel is unavailable. Numeric capture is uncached bulk Value2;
typed capture verifies fixture readback and records the 1900 date system and
`PrecisionAsDisplayed=false`. Each retained file records its original source
and retained SHA-256 in `artifact-manifest.json`; `freeze-v1.json` predates
independent heldout capture. All final numeric replays are in
`final-numeric-judgement.json`.

The typed discovery's original 25/39 count is historical. Its final local
outcomes and the heldout were replayed after the independent JSON decoder repair
using `serde_json/float_roundtrip`; `outcomes/local-roundtrip-v1.jsonl` records
the current exact result. Historical outcomes and comparisons were preserved.

Validation passes: focused SLN dispatch/kernel tests (3), the existing
depreciation-family tests (8), and `lake build OxFunc.Functions.DepreciationFamily`.
The Lean layer models exact rational-domain admission, explicit missing input,
and the ordered publication branches. Binary64/extended rounding remains bound
to the existing numeric substrate and the retained exact witnesses.

```powershell
cargo test -p oxfunc_core --test sln_excel_parity_20260929
cargo test -p oxfunc_core --lib functions::depreciation_family::tests
cargo run -q --manifest-path smart-fuzzer/engine/Cargo.toml -- judge-witnesses docs/function-lane/evidence/w111-broad-20260929/sln/discovery-answers.json docs/function-lane/evidence/w111-broad-20260929/sln/discriminator-answers.json docs/function-lane/evidence/w111-broad-20260929/sln/heldout-answers.json --json
```
