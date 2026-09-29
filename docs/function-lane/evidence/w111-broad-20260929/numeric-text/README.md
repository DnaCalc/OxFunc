# Shared numeric-text coercion: W111 observations and candidate

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: locale grouping/currency/native digits/date/time,
captured current-year context, and receiving-repository
acknowledgement/integration of HO-FN-022. No full function identity is claimed.

`IMPACT_ASSESSMENT.md` was written before the shared parser edits. It identifies
the scalar coercion callers, aggregate origin policy, existing provider seam and
cross-repository impact. This candidate changes only the locale-independent
ASCII grammar and decimal conversion. It does not preconvert global arguments,
change aggregate reference-text exclusion, or introduce implicit locale/clock
state. NOT remains a distinct logical-argument policy.

The observed grammar trims ASCII space only; accepts one sign (leading sign or
outer parentheses) and one percent mark (prefix or suffix), with ASCII spaces
between decorations; and rejects repeated marks, double signs, non-ASCII/control
whitespace and malformed decimal/scientific syntax. Sign and percent decoration
can be consumed in either order. The parser keeps the first 15 significant
lexical digits without rounding. Percent contributes -2 to the decimal exponent
before conversion. It is not a binary64 division after parsing.

For admission, the normalized decimal point position is lexical exponent minus
fractional-digit count plus significant-digit count, including the percent
adjustment. It must be within -308..308. All-zero mantissas have zero significant
digits, so `0e309` fails while `0.0e309` succeeds. Admitted binary subnormals
publish positive zero. The 364 paired ABS/IMREAL and 360 percent-boundary probes
distinguish the position rule, zero behavior and scale/admission order.

The original direct correctly rounded decimal-to-binary candidate was falsified
by `9.35037643088324e-128`: Excel returns `0x25903414b3117a7e`, while exact RN53
returns the next larger encoding. The 1,960-row targeted midpoint packet expands
that failure to 58 rows. RN63, RN64 followed by RN53, single rounded-power
multiplication/division, and two-stage scale models are also falsified.

The current arithmetic candidate starts with the retained integer significand.
Split the absolute decimal scale into its base-16 digits. Multiply, in ascending
digit order, by correctly rounded powers of ten for the low digit, the next
digit times 16, and the high digit times 256, using reciprocal powers for a
negative scale. Each multiplication rounds to 64 binary significand bits with
ties to even and an unbounded exponent. Finally round once to binary64. This is
an observable arithmetic model, not a claim about Excel internals.

The 64-entry power table is generated from exact rational mathematics by
`gen_decimal_text_powers.py`. Portable u128 multiplication and exact quotient
rounding implement the stages in `coercion_decimal.rs`; no x87 or platform DLL
dependency is introduced. The executable Lean `DecimalTextConversion` computes
the same powers and stages directly and checks five retained distinguishing
encodings. `CoercionPrimitives` models lexical truncation and position admission.
Both Lean modules build.

Production replay currently passes 1,974 core discovery rows, 2,980 prior grammar
rows, all 3,226 precision rows, 360 percent-boundary rows and 1,960 midpoint rows.
The historical `precision-heldout-open.json` contains the former two-row
counterexample; its regression is now active and passes. Historical captures
and model-comparison files retain the rejected candidates' results unchanged.

`nibble-candidate-freeze.json` records the arithmetic model before the independent
3,760-row oracle capture; `nibble-rust-candidate-freeze.json` records the portable
Rust candidate before those outcomes were consulted. The independent packet
contains 160 newly selected midpoint centers, five decimal neighbors each,
paired ABS/IMREAL, 1,200 fresh short-to-15-digit decimal controls, and 160
equivalent-spelling triples paired across both functions. All 3,760 independently
captured rows match exactly, with no ingress exclusions or normal residuals.
The complete retained packet is `nibble-heldout.json`; production regression
exercises every row through the real function dispatch.

All identification used reproducible public-interface Excel observations and
exact arithmetic models. Comparative calls to documented `strtod` and
`VarR8FromStr` APIs did not match the midpoint corpus and were not used as a
semantic authority or runtime implementation. No binary internals were read.
