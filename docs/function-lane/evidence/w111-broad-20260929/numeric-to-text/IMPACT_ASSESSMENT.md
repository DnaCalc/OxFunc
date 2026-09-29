# Generic numeric-to-text characterization: scope and potential impact

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: midpoint arithmetic precision, contextual policy, unexercised consumers,
and upstream acknowledgement. This assessment precedes the proposed source change.

`coerce_prepared_to_text` in `functions/adapters.rs` currently formats a Number
using Rust's default `format!("{n}")`. The helper is referenced by 28 source
modules, including its defining module. Consumers include text slicing, EXACT,
concatenation, TEXTJOIN, regex text admission, search/replace, text delimiters,
Unicode conversion, date-value admission, engineering conversion, hyperlink,
reference metadata and several host-service argument adapters. A change is
therefore evaluator-visible and is not confined to one text function.

The first packet has 7,048 cases covering 1,465 distinct zero/normal binary64
values. LEFT returns the full formatted text, LEN measures it, and EXACT compares
candidate spellings. RIGHT, MID, TRIM, CONCAT, ENCODEURL, LOWER, UPPER, CLEAN,
REPT, SUBSTITUTE and TEXTJOIN supply independent coercing-function controls.
Fresh normal bit patterns, exponent transitions, decimal ties, minimum/maximum
normal values and previously distinguishing formatter inputs are all retained.
Negative zero and subnormals are not admitted through Value2; their transformation
must not become an invented core-formatting result.

The discovery capture executed all 7,048 cases with exact numeric ingress. All
1,465 LEFT strings match the already retained ADDRESS formatting model exactly:
initial 15 significant decimal digits with exact ties toward zero, fixed notation
when the unsigned fixed string has at most 20 characters, otherwise scientific
notation with a signed exponent of at least two digits, and a second half-away
reduction to 14 digits when the exponent magnitude is at least 99. Every generic
cross-function observation agrees with that LEFT text. This supports factoring
the observed mechanism while keeping the two semantic call sites explicit; it
does not establish universality beyond the retained observations. The fresh
heldout includes decimal midpoint neighbors, exponent/width transitions,
fresh random normal bits, reference/array origin and additional consumers.

COMPLEX coefficients remain a distinct observed formatting policy. The packet
includes 87 controls for each specialized formatter. COMPLEX's current numeric
rounding substrate still has retained smart-probe counterexamples; the shared
decimal parser's separately evidenced 15-digit truncation and staged conversion
must not be transplanted into binary-to-decimal output formatting.

If the observations support a shared generic numeric-text policy, its pure
finite-number formatting mechanism can be isolated as a crate-private helper,
called from the existing Number branch and the unchanged ADDRESS numeric sheet
branch. Text, logical, blank, error and reference preparation stay unchanged.
COMPLEX stays on its separately observed formatter. The finite normal/zero domain
is the evidenced domain; nonfinite or subnormal direct carrier values are not
promoted by this capture. Context-dependent locale formatting or a changed public
declaration requires the parent campaign's explicit boundary assessment and
handoff. Any inferred common arithmetic must first survive distinguishing and
fresh independent observations. No whole-function parity claim follows from
this proposed shared-helper scope.

The 28 affected modules (including the helper definition) are adapters,
arabic_fn, array_text_split_family, call_register_id_family, cell, clean_fn,
concat_family, date_value_family, decimal_fn, engineering_radix_family, exact_fn,
hyperlink_fn, info_fn, misc_conversion_family, number_regex_translate_family,
operator_compare_concat_family, reference_metadata_family, rtd_fn, textjoin,
text_b_compat_family, text_compat_locale_family, text_delim_family,
text_scalar_misc, text_fn, text_slice_family, text_search_replace_family,
text_unicode_fn and web_text_xml_family. Some receive text selectors, parse bases,
or forward strings to host services rather than returning text directly; matching
the shared conversion alone does not characterize those downstream semantics.

The declaration signature and preparation order remain unchanged, but the Number
to Text policy is observable at the FEC/F3E seam and warrants a parent-owned
cross-repo handoff before integration promotion. Existing locale and workbook
context providers are not replaced with a hardcoded new context. The current
capture is Excel 16.0 build 20430, workbook CV2, recorded oracle locale only.

Follow-up explicit serializers: all 560 VALUETOTEXT/ARRAYTOTEXT observations
(both modes, scalar numbers and mixed arrays) use exactly the generic number
text captured independently through LEFT. Their six Number branches will call
the existing prepared text helper, adding valuetotext_fn as the 29th dependent
module. Non-number serialization, quoting, separators, format selectors and
array publication stay unchanged. TEXT with General format is observably
different and is excluded from this repair. This extension needs its own fresh
serializer heldout before a bounded evidence claim.
