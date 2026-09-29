# Text slicing, defaults and array observations

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: exceptional midpoint arithmetic in shared numeric textification,
context/version declarations, DBCS behavior, empty-call syntax admission, and
evaluator-facing metadata and handoff acknowledgement.

The follow-up in `../numeric-to-text/` repairs the generic number formatting
policy for 15,169 passing generic/ADDRESS observations, but retains six LEFT
counterexamples from deliberately close decimal midpoints. Thus numeric source
text remains an explicit semantic gap even though the slice/count/raw-unit
observations below continue to pass.

The candidate matches 34,267 admitted observations through actual production
dispatch: 1,945 typed discovery rows, 1,888 numeric boundary rows, 2,508 initial
raw UTF16 rows, 12,672 raw tail controls, 2,100 independent typed/numeric rows
6,342 independent raw-unit observations, and 6,812 independent fallback controls. Seven empty-call formulas failed
Formula2 assignment and have no asserted function result. Another 264 initial
raw rows contain NUL inputs that Value2 truncated; they are retained but withheld
by their explicit `ingress_exact=false` flag. The tail capture has exact ingress
for all 12,672 rows. No source or result surrogate is converted through a lossy
text encoding in the raw-unit tests.

The current reference is Excel 16.0 build 20430, 64-bit, Compatibility Version 2,
with the existing non-DBCS locale profile. Channel is unverified. These are
public COM black-box observations; no binary internals were inspected.

LEFT/RIGHT/LEFTB/RIGHTB use one when the count argument is absent and zero when
an explicit argument slot is omitted. Every slicer rejects a negative raw count,
including fractions between minus one and zero. LEFT/RIGHT and their B aliases
use the same observed positive tolerant floor as ADDRESS/DATE: for `u=ceil(x)`,
admit `u` when `u-x <= 2049/8589934592`; otherwise take the floor. Large positive
counts clip safely to available text. MID/MIDB instead truncate their numeric
arguments, require raw start at least one and raw count at least zero, and reject
either raw value at or above 2^31. Omitted MID numeric slots become zero before
these domain checks. Scalar coercion occurs from left to right before validation.

Compatibility Version 2 treats valid high/low surrogate pairs as one position in
LEFT, RIGHT, LEFTB, RIGHTB and MID. Isolated surrogates remain raw units. LEN and
MID's forward path omit a final unmatched high surrogate. RIGHT preserves that
tail. LEFT falls back to the original count measured in raw units when the forward
scan stops at a final high surrogate before reaching that character count. This
can split an earlier surrogate pair. A count beyond the raw length returns the
whole text. The 96-string tail
packet separates these rules with exhaustive A/high/low sequences of lengths
one through three and fresh longer combinations. MIDB indexes raw UTF16 units
and LENB reports raw UTF16 length. No grapheme, normalization or replacement
character conversion is introduced.

The six slicers, LEN, LENB, EXACT and ENCODEURL now apply their scalar policy to
each position of the broadcast result. Singleton dimensions broadcast; other
absent coordinates become #N/A arguments. An earlier explicit argument error
can outrank a later padded #N/A. The existing generic broadcast helper returned
#N/A immediately when any coordinate was missing, so the final candidate uses
the local `run_text_lifted` helper. The generic helper and other families are
unchanged. Reference inputs are prepared through the existing resolver seam.

`w111_text_slice_live_replay.rs` asserts retained outputs rather than generating
expected values from these rules. Its seven retained replay tests pass, as do 23
existing unit tests across the four Rust source families. The Lean build passes
12 jobs, including the executable UTF16 paths, finite numeric conversion models,
raw EXACT identity and a shared text broadcast model. Existing metadata is kept
unchanged for the parent campaign's cross-repo declaration assessment.

`candidate-freeze.json` records source hashes before fresh 2,100-row typed/numeric
and 6,342-row raw-unit heldouts. The raw heldout includes arbitrary fresh
surrogate values and maximum-length tail controls. All 2,100 typed/numeric outcomes matched the first frozen candidate. The raw
heldout falsified the first tail model in five rows; those failures are retained
in `first-raw-heldout-failures.json`. The generalized raw-count fallback above
now matches all 6,342 raw heldout observations. `fallback-candidate-freeze.json`
records the refined source before a new 6,812-row independent count sweep.
All 6,812 inputs were ordinal-exact and every outcome matches the frozen refined
candidate. These observations do not establish general
number-to-text formatting, DBCS modes or evaluator syntax/scheduling semantics,
and no whole-function parity claim is made.
