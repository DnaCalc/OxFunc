# Typed JSON numeric ingress correction

The default serde_json number decoder changed 2,918 of 10,000 independently
generated positive normal binary64 inputs by one or two ULPs before the ABS
kernel ran. Enabling `float_roundtrip` gives 10,000/10,000 original-bit retention.
This corrects the local verification input; the Excel Value2 input/readback and
the hex-encoded numeric WitnessSet path are separate and unaffected.

`cases-and-results.json` retains each exact decimal token, the original source
hex bits, and the ABS result bits before and after the correction, together with
source-file hashes. The expected source bits were generated independently of
the JSON parser under test. Values are normal positive numbers, so this is not
the separate Excel negative-zero/subnormal storage issue.

The local evaluator's normal dependency and `oxfunc_core`'s test-only dependency
now enable `serde_json/float_roundtrip`. Reduced permanent tests exercise the
actual typed materialization/public-dispatch path and the regression-fixture
decoder. No production function arithmetic was changed for this correction.

`smart-fuzzer/tools/w111/replay_typed_campaign_20260929.py` replays finished typed
captures into separate `local-roundtrip-v1.jsonl` files without overwriting the
original outcomes. Its summary is `smart-fuzzer/cache/w111-typed-roundtrip-replay.json`.
Current function fixes can also change those replay outcomes; the summary does
not attribute every changed output solely to the parser.

Status: in progress. `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Open lanes: final campaign replay/retention after all function candidates settle.
