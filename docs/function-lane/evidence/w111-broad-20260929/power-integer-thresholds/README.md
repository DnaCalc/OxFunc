# POWER integer width and decimal exponent conversion

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes include uncharacterized POWER
inputs and typed/reference surfaces, broader consumer domains, and evaluator
integration. Exact retained observations do not establish universal parity.

The first four-family POWER holdout exposed 38 differences among 23,264 cases.
The initial source, source freeze, and unsuccessful models remain retained.
The subsequent black-box packets distinguish two general rules:

1. Repeated-product integer publication admits exact exponent magnitudes
   strictly below `u32::MAX` (4,294,967,295). The sentinel and larger integers
   take the existing noninteger body. This is an observed dispatch boundary;
   it does not require a claim about Excel's internal implementation.
2. Within that admitted branch, positive base ten receives the magnitude as
   a signed32 decimal scale. The original exponent's sign then controls the
   reciprocal. Thus `POWER(10,2^32-2)` returns 0.01, while the negative exponent
   returns 100. Negative base ten retains the general repeated-product path.

The unsigned32-exclusive model matches 13,622/13,622 discovery observations.
An inclusive unsigned32-sentinel variant loses 76 of those observations;
signed32-only bounds lose thousands. A separate 1,672-case packet exhausts
effective decimal scales -400 through -1, both exponent signs, and direct-scale
controls. It confirms the existing staged decimal-pair converter, followed by
the existing reciprocal/publication rule. Neither packet uses fitted output
corrections or a list of special input witnesses.

The candidate was frozen before generating an independent 2,978-case packet.
It matches all 2,978 outcomes, including 2,880 input pairs absent from the two
discovery packets. All six production source hashes in `candidate-freeze.json`
were unchanged before replay. The remaining 98 inputs are stable controls.
Each raw answer file retains Excel 16.0 build 20430, workbook compatibility
version 2, public bulk Value2 capture provenance and no-cache metadata.

| Packet | Current exact observations |
|---|---:|
| Earlier four-family POWER discovery | 20,206 / 20,206 |
| Earlier four-family POWER holdout | 23,264 / 23,264 |
| Width discovery | 13,622 / 13,622 |
| Signed decimal-scale controls | 1,672 / 1,672 |
| Frozen independent width holdout | 2,978 / 2,978 |
| Total observations | 61,742 / 61,742 |

The earlier 38 failed outcomes remain in the four-family evidence and the
initial model comparison here. These totals contain repeated controls and
must not be described as globally unique inputs. The two new focused Rust
tests check exact number bits and error identities; the existing eight
four-family tests also pass. `OxFunc.ElementaryPublication` builds with the
unsigned32 admission, effective signed32 decimal scale, and 32-step product
binding. Negative fractional-root admission remains outside that integer model.

The source change is confined to `power_fn.rs` and its integer formal binding.
The shared decimal converter and financial source files are unchanged. A
repeated [consumer audit](../power-consumer-audit/README.md) finds no changed
outputs across 76,611 retained FV/PV, DB/DDB/VDB, and XNPV observations; it
explicitly preserves the existing 317 FV/PV discrepancies and XNPV's array
runner limitation.

Reproduction tools are `gen_power_integer_thresholds.py`,
`gen_power_decimal_wrap.py`, `gen_power_integer_thresholds_heldout.py`, and
`power_integer_threshold_probe.rs`. The probe compares alternative width and
scale models; production replay uses the frozen public-dispatch `sf` binary
recorded in `candidate-freeze.json`. No Excel binary inspection or proprietary
source was used.

The research probe preserves the initial candidate's integer body after the
production refinement. Its `production` score label refers to that initial
candidate. Rebuilding it against current production reproduces every stored
row of `discovery-signed-scale-models.json` and `decimal-wrap-models.json`
exactly, including the rejected alternatives.
