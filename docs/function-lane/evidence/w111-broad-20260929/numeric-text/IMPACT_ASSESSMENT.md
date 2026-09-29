# Implicit numeric-text coercion: W111 impact assessment

State: in_progress; scope_completeness=scope_partial,
target_completeness=target_partial, integration_completeness=partial.
Open lanes: locale-specific grouping/currency/digits/date/time, missing-year
clock context, decimal conversion precision/extremes, independent validation,
and OxFml acknowledgement of the preparation-context changes.

## Observed classes

The 2,544-row, 53-function exact-ingress Excel capture has 1,336 exact matches
and 1,208 differences against the frozen local capture. Classification in
`../numeric-text-analysis/classification.json` separates:

- 1,182 shared numeric-text parser differences across 52 functions;
- 24 NOT logical-argument policy differences, owned by the logical agent;
- two ACOT(20) one-ULP kernel differences, unrelated to text parsing.

Forty-seven numeric scalar functions coerce direct and referenced text with the
same observed grammar; ADDRESS's coordinate domain masks some admitted numeric
values as #VALUE!. SUM, AVERAGE, MIN, MAX and PRODUCT use that grammar for direct
scalar text and ignore referenced text. All 120 aggregate reference-text rows
already match and must retain that policy. NOT uses exact logical text, not the
numeric parser. The 720 ADDRESS syntax rows additionally compare 120 spellings
in all three numeric arguments, each as literal and referenced text; every pair
has identical Excel results. The current ADDRESS implementation differs on 288
of those rows. None of these differences may be treated as ingress failure.

## Narrow shared Rust repair authorized now

Change only the shared parser in `crates/oxfunc_core/src/coercion.rs` and its
Lean substrate. Preserve its caller signatures, origin policies, error mapping,
reference resolution, scalar/aggregate blank behavior and public metadata.

The proposed candidate trims ASCII space only; admits one percent prefix or
suffix with surrounding ASCII spaces; admits parentheses as a negative sign;
rejects explicit signed numbers inside parentheses, repeated percent marks,
non-ASCII/control whitespace and invalid residual decimal syntax. Parentheses
and a percent mark can compose (for example `(1%)` and `(1)%`). Decimal/exponent
conversion remains explicit in the candidate and needs fresh bitwise probing;
these observations alone do not establish whether percentage scaling before or
after binary rounding is universally correct.

This repairs a shared semantic function, not only ADDRESS. The two current
entry points `coerce_eval_to_number` and `coerce_calc_scalar_to_number` both use
this parser. `adapters::coerce_prepared_to_number` funnels scalar calls through
the former; `aggregate_common` calls the latter only after deciding direct
versus referenced/array origin. Therefore the grammar repair propagates to
numeric arguments without changing aggregate reference-text exclusion.

## Cross-repository impact

OxFml callers supplying raw text to numeric OxFunc arguments will observe the
new accepted percent/parenthesis forms and rejection of surrounding non-ASCII
whitespace. No FEC/F3E shape or Rust signature changes are proposed in this
narrow repair. The change still affects evaluator-facing coercion policy and
must be included in the root-owned OxFml handoff and register. This local
assessment is not receiving-repository acknowledgement.

The larger correction requires context already present at the dispatch seam:
`FunctionExecutionContextBundle` carries `locale_ctx` and a captured time
serial/provider; `eval_surface_value_call_with_dispatch_key` receives both.
But low-level coercion helpers do not receive either. Existing
`LocaleFormatContext` holds a FormatProfile, workbook date system, parser and
formatter; `LocaleValueParser::parse_value_text` has no captured current year.
The production parser is `OxFmlLocaleValueParser` in the sibling OxFml format
engine, not OxFunc's cfg(test) seed parser. That production parser also uses
Unicode trim and lacks several newly observed forms, so simply wiring it into
all numeric arguments would not establish parity.

Prefer an explicit preparation-context binding: extend the parsing contract
with the observed numeric-argument grammar and an evaluation-date snapshot,
then route that capability to the numeric coercion sites. Do not preconvert all
text arguments at global dispatch: ADDRESS's sheet/style positions, NOT's
logical input and aggregate origin policies demonstrate why that would alter
semantics. Do not hide context in thread-local/process globals or the reference
resolver. Preserve raw argument origins and left-to-right coercion/error order.
Exact sharing between implicit coercion and VALUE parsing needs differential
probes before making it a single unconditional parser mode.

Locale-specific grouping, currency and date interpretation are outside this
narrow patch. The recorded host uses decimal point, ASCII-space grouping, R
currency, YMD date order, 1900 date system and current year 2026. Euro acceptance,
Arabic digit acceptance and missing-year fallback are empirical observations;
they are not permission to hardcode those host settings in a generic parser.
Likewise a missing-year date must not read the process clock implicitly. The
function dependency/hoisting metadata needs a conditional-preparation context
assessment; nonvolatile worksheet recalculation does not mean independence
from captured locale/year context. Root owns that handoff and metadata decision.

## Validation plan

Retain the original 2,544 and 720 packets unchanged. Freeze the narrow candidate,
then probe fresh signs, parentheses, percent position, ASCII-space placement,
invalid decorations and non-ASCII whitespace through several numeric functions
and direct/reference origins. Include fractional and exponent strings that
separate decimal percentage scaling from binary64 division. Replay aggregate
reference-text and NOT controls. Compare exact output bits/types and report
remaining context-dependent failures separately. No whole-function completion
claim follows from the narrow grammar repair.

## Refinement after the first independent captures

The above assessment records the pre-edit proposal. Subsequent independent
captures required lexical retention of 15 significant digits without rounding,
decimal percent scaling before conversion, normalized decimal-point admission
within -308..308 (including zero mantissas), and subnormal publication as positive
zero. The first correctly rounded conversion still failed at rare normal
midpoints. The current portable staged-power candidate and its separately frozen
independent validation are described in `README.md`; these remain within the
same pure decimal-lexeme conversion seam and preserve all caller signatures.

The cross-repository handoff must include these precision/admission rules as
well as the original grammar changes. HO-FN-022 is filed and registered by the
root campaign; acknowledgement remains an open dependency. Context-dependent
forms are still excluded from this narrow implementation and remain observable
semantic gaps, including for ADDRESS and QUOTIENT numeric arguments.
