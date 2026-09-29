# HO-FN-022: implicit numeric-text coercion needs locale and evaluation-date context

Direction: OxFunc → OxFml. Status: `filed`, acknowledgment pending.
Owner: W111, `oxf-mwue.28.8`, BUG-FUNC-057.

The 2026-09-29 public-interface Excel campaign exposed a shared preparation gap.
Exact typed fixture readback rules out text-to-cell conversion as its cause.
The 2,544-row, 53-function packet contains 1,182 numeric-text parser differences,
24 separate NOT logical-text policy differences, and two ACOT kernel differences.
The [impact assessment](../function-lane/evidence/w111-broad-20260929/numeric-text/IMPACT_ASSESSMENT.md)
and [classification](../function-lane/evidence/w111-broad-20260929/numeric-text-analysis/classification.json)
retain the argument origins and exact outcomes. The reference is Excel16.0
build20430,64-bit,Workbook Compatibility Version2,1900date system; update channel
is unverified. The [captured locale profile](../function-lane/evidence/w111-broad-20260929/locale-context/live-profile.json)
has decimal point, ASCII-space grouping, R currency and YMD date order.

## Observations and proposed narrow repair

On this reference, numeric scalar arguments admit single-percent and negative
parenthesis forms. They trim ASCII space but reject surrounding tabs, newlines
and NBSP. Direct and referenced text have the same numeric grammar for scalar
functions. Aggregate SUM/AVERAGE/MIN/MAX/PRODUCT coerce direct text and continue
to ignore referenced text. NOT has a separate logical-text grammar.

OxFunc is testing a narrow shared parser repair for the locale-independent
decimal decorations and space rules. It preserves current Rust signatures,
argument-origin handling, blank/error policy and metadata. Consumers passing
raw text into numeric arguments will nevertheless observe different acceptance
and errors. Please acknowledge that behavior change against the retained cases.
The repair remains in progress; its independent heldout may require refinement.

## Context requirement still open

ADDRESS's numeric arguments also accept region-specific grouping/currency,
Arabic decimal digits and date/time text. Its 720-case syntax packet includes
`"1/2"` resolving to serial46024 (2026-01-02) and `"1/2/24"` resolving to
serial36946 (2001-02-24) on the captured YMD host. A fixed parser cannot reproduce
the missing-year form without an evaluation-date snapshot. These are observed
grammar cases, not a proposed universal interpretation across locales.

`FunctionExecutionContextBundle` already carries locale and time facilities, but
`coerce_prepared_to_number` and the scalar parser receive no context.
`LocaleValueParser::parse_value_text` currently receives profile/date-system data
without the captured evaluation year. The existing OxFml production parser also
differs on the newly observed grammar, so routing every argument to it is not
yet sufficient. Implicit coercion and VALUE need differential tests before they
share an unconditional parse mode.

Please assess an explicit preparation-context path that supplies a locale
profile, date system and evaluation-date snapshot only where numeric coercion
is requested. Preserve raw text and argument origins, left-to-right error order,
reference/array aggregate exclusion and distinct logical parsing. Do not
preconvert all text at dispatch or place locale/time state in thread-local or
process globals. ADDRESS's sheet/style arguments illustrate why that would
change unrelated semantics.

The function dependency and expression reuse rules also need assessment:
nonvolatile recalculation does not imply that text preparation is independent
of locale and captured year. No new FEC/F3E signature or metadata promotion is
made in this packet. Any final context design must be exercised on both repos.

## Receiving-side response requested

1. Acknowledge the narrow grammar behavior and preserve origin/error invariants.
2. Identify the evaluator's authoritative captured date and locale objects and
   how their identity participates in preparation, caching and replay.
3. Agree the parser mode/context contract, then replay percent/parenthesis,
   whitespace, regional and missing-year cases through the actual evaluator.

`scope_completeness: scope_partial`; `target_completeness: target_partial`;
`integration_completeness: partial`. Open lanes: heldout refinement, locale/date
grammar, context contract, receiving-repository acknowledgment and integration.
Filing this packet opens the dependency; it does not establish cross-repo parity.
