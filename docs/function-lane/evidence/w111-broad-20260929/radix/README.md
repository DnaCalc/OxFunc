# BASE and engineering radix observations

State: `in_progress`; `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Owner: `oxf-mwue.28.2`, BUG-FUNC-054. Open lanes: shared text/context preparation,
combined validation and completion audits. The reference is Excel 16.0 build
20430, 64-bit, Compatibility Version 2, 1900 date system; channel is unverified.

BASE admits the original number only in `[0, 2^53)`, then truncates it. Radix
truncates to 2–36. The original minimum length must be nonnegative and its
truncated value at most 255. Omitted/explicitly missing length defaults to one;
an explicit zero length with number zero returns empty text.

Engineering conversions truncate numeric inputs and widths. A supplied width
must truncate to 1–10, even for a negative result whose valid width is then
ignored in favor of the ten-character signed representation. Width coercion,
worksheet errors and width bounds precede inspection of the first argument.
Explicitly omitted width uses the default. Logical arguments reject with
`#VALUE!`; a missing required first argument produces `#N/A`.

Binary, octal and hexadecimal source text accepts ASCII letter case and empty
text as zero, with no leading or trailing whitespace trimming. Ten characters
use signed 10-, 30- or 40-bit interpretation respectively. Referenced empty cells
prepare as zero. Target overflow and invalid source digits publish `#NUM!`.

The actual-dispatch replay tests cover 5,915 numerical boundary observations,
8,000 fresh numeric holdout observations, 3,249 admitted typed discovery cases
and 1,950 independent typed holdout cases: 19,114 exact comparisons. Twelve
empty-call formulas rejected during Excel formula entry remain retained and
withheld. Numeric arguments use Value2; text ingress is read back ordinally.
All 14 numeric and two typed replay tests pass on the current candidate.

Lean `BaseFn.lean` models finite numeric admission and omitted/zero widths.
`EngineeringRadixFamily.lean` now provides executable typed width-first error
preparation, source parsing, signed interpretation and digit encoding, with
captured boundary witnesses. The focused build passes eight jobs.

The repair retains current signatures, metadata and reference/array provenance.
Shared numeric-text conversion is separately evidenced; full regional/date-text
preparation awaits HO-FN-022 acknowledgment and integration. The XLL harness can
only recreate host context where its recorded provider supplies it. These
records do not claim full function parity beyond the exercised current reference.
