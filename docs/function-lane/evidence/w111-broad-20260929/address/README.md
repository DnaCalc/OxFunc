# ADDRESS observations, 2026-09-29

State: `in_progress`; `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Open lanes: exceptional midpoint arithmetic in numeric sheet formatting,
implicit numeric-text parsing and its locale/date/current-year
context, cross-repository assessment of that preparation seam, and final
function-level claim audit. This is not a function completion claim.

All captures use the public worksheet interface of Excel 16.0 build 20430,
Workbook Compatibility Version 2. No Excel binaries or internals were inspected.
Arithmetic explanations are behavioral models, not claims about execution
instructions. Original manifests/profiles are retained in each evidence packet.

Later precision counterevidence is retained in `../numeric-to-text/precision.json`:
six ADDRESS numeric-sheet results differ on three positive midpoint-adjacent
centers and their negatives. For example bits `0x3bbf045596054946` render a sheet
number `6.56809060701061E-21` in Excel versus `6.5680906070106E-21` locally.
These are admitted exact-ingress semantic differences, not a change to any of
the earlier passing banks. The specialized formatter was factored into the
shared `numeric_text` module after 15,169 generic/ADDRESS observations agreed;
its initial rounding rule remains partial after the more targeted capture.

## Coordinates and selectors

The initial 1,284-row capture exposed 23 structural differences. Relative R1C1
axes accept signed offsets with strict bounds `-limit < offset < limit`; zero
renders `R` or `C`, without `[0]`. Absolute R1C1 and A1 coordinates instead use
`1 <= coordinate <= limit`, with limits 1,048,576 rows and 16,384 columns.

| Capture | Rows | Purpose |
|---|---:|---|
| discovery.json | 1,284 | Initial broad baseline |
| boundaries.json | 928 | Signed limits, fractions and adjacent floats |
| rounding.json | 6,240 | Integer-neighbor neighborhoods |
| floor-threshold.json | 8,362 | Broad fractional distances |
| floor-bits.json | 2,064 | Exact-bit threshold neighborhoods |
| floor-zoom.json | 1,851 | Independent dyadic threshold refinement |

All 20,729 rows match the production-dispatch replay. The provisional truncation
candidate in `candidate-freeze.json` was refuted by the 928-row capture; it is
retained as history, not the current candidate.

The observed conversion for coordinates and `abs_num` is a tolerant floor. Let
`u=ceil(x)`. Return `u` if `u-x <= 2049/2^33` for positive `u`, or
`u-x <= 2049/2^34` otherwise; outside the interval use `floor(x)`. Domain checks
follow conversion. On the admitted ADDRESS range this equals
`floor(RN53(RN64(x+2^31))-2^31)`, where RN64 denotes 64 binary significand bits.
The two half-spacings above 2^31 are 2^-22 and 2^-33; below it, 2^-23 and 2^-34.
RN53 alone matches only 1,230/2,064 narrow probes; the two-stage model matches all
2,064 and all subsequent 1,851 zoom rows. Fifteen-significant-decimal conversion
and plain truncation were separately refuted. Rust uses the reduced comparison.

Coercion errors propagate in argument order; coordinate/mode domain checks wait
until supplied arguments have been coerced. Thus a later worksheet error wins
over a numeric invalid mode, but not over earlier unparseable text. Style accepts
numbers, logical values, and exact ASCII-case-insensitive TRUE/FALSE text; it
rejects numeric text and whitespace around logical text. Omitted first coordinate
acts as zero; omitted second coordinate errors. Omitted mode/style default to
1/true. Referenced and array blank mode/style instead coerce to zero/false.

## Sheet text and UTF-16 rendering

Sheet grammar is position-sensitive. `address_name_classes.rs` and
`name-classes.json` hold 437 NameStart and 416 NameContinue BMP intervals derived
from exhaustive position probes, not a table of whole names. The retained
`unicode-answers.json.gz` contains all 130,562 discovery observations. Exact
ordinal Value2 ingress qualified every text in that discovery. ASCII/Latin-1
rules remain explicit in the Rust source. High surrogates qualify initially;
low surrogates qualify only after the first unit. Every admitted isolated UTF-16
unit is preserved unchanged, rather than converted through a lossy String.
`utf16.json` records 4,102 raw-unit probes: 4,100 exact ingress and matching output,
with two NUL-ingress exclusions. Both well-formed and isolated surrogate cases
are retained as hexadecimal code units.

The grammar composes simple names and `[book]sheet` names. Nonempty book text uses
the continuation class; the suffix uses the simple-name grammar. Whole-token
A1 references and TRUE/FALSE/R/C/RC require quoting. A leading R row or C column
numeric token, including RC column form, also requires quoting when its integer
lies within that axis limit, unless the next character is a dot. The remaining
suffix need not form a reference: R1foo and C1R0 are quoted. C16385foo and
R1048577foo are bare; A1 reference recognition still quotes exact C16385.
External-book prefixes bypass this whole-name ambiguity check.

This general composition rule is supported by the public name grammars in
[MS-OE376](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oe376/a469e5f5-a102-49bd-9642-8a8e8aaf1623)
and MS-XLSB. Those documents do not determine the observed Unicode classes or
all ambiguity details; the captures are the empirical authority here.

Sheet text must contain at most 255 original UTF-16 units and at most 256 after
apostrophe doubling. Quoted names add outer apostrophes. Prefix construction
precedes bounded body append, and the prefix can itself reach 259 units. Body
literals clip at capacity 258; each integer magnitude appends only if all digits
fit. Its minus sign is a separate literal fragment. Long names therefore can
omit digits while retaining brackets, signs or labels. `tokens.json` (236) and
`sheet.json` (634), plus the independently composed heldout, exercise these rules.

## Numeric sheet text

`numeric-sheet.json` (1,770), `numeric-format-discriminators.json` (2,100), and
`numeric-ties.json` (288) all match. The candidate first rounds the exact value
magnitude to 15 decimal significant digits, with an exact midpoint toward zero.
The 288 parity-controlled tie rows distinguish this from half-even (14 misses)
and half-away (28 misses). The initial decimal pair therefore reuses ROUND's
`fifteen_significant_digits(number, false)` helper; only its visibility changed.
No other ROUND behavior changed, and this primitive is distinct from COMPLEX's
still separately studied numeric formatting.

Unsigned fixed text is used when its length is at most 20. Otherwise scientific
text uses an explicit exponent sign and at least two exponent digits. Exponent
magnitude 99 or greater narrows the initial decimal pair to 14 digits in a
second, half-away decimal step. Direct initial 14-digit rounding is refuted.
The sign does not count toward the fixed-width threshold. MAX binary64 becomes
1.7976931348623E+308; the minimum normal remains nonzero.

## Independent validation and remaining failures

`grammar-validation.json` contains 4,096 composed names; two rows with native
numeral strings changed type under General Value2 ingress and are excluded by
exact input string. All 4,094 admitted rows match, including the reference-prefix
cohort after the 716-row `reference-prefix.json` refinement.

`refined-candidate-freeze.json` records source hashes and independent seed
202609291429 before the 5,400-row heldout capture. `refined-heldout.json` retains
all answers and ingress qualification. All 5,330 admitted rows match production
dispatch: 1,800 coordinate cases, 1,800 numeric-sheet cases (including 1,200 fresh
normal input bit patterns and 600 fresh midpoint neighborhoods), and 1,730 sheet
grammar cases. Seventy rows share 65 strings altered by General Value2 ingress,
mostly leading-apostrophe names. These are excluded as transport observations,
not accepted function results. The earlier initial-candidate 4,096-row heldout
has not been used for refinement or promotion.

Typed regression evidence comprises 301 initial cases, 84 error controls, 634
sheet cases, 597 refinement cases, 236 token cases, and 40 blank/omission/lift
cases. Four NUL fixtures failed exact ingress, leaving 1,888 admitted rows.
PowerShell's flattened one-row result wrapper is restored using explicit shape
metadata; every cell's exact type/text/error is preserved. The production test
runs 41,015 admitted equivalence comparisons across numeric, typed, grammar,
raw UTF-16 and heldout packets.

**Known numeric-text failures are retained, not accepted as equivalence.**
`blank-numeric-text.json` records 100 rows including 60 numeric-text cases; 19 of
those rows differed before the subsequent shared ASCII grammar candidate. `numeric-text-syntax.json` records 720 further exact-ingress
literal/reference controls over 120 spellings and three numeric arguments. Direct
and referenced forms agree in every pair. The current host accepts ASCII-space
grouping, R/euro currency, percent prefixes/suffixes, parenthesized negatives,
Arabic digits, and profile-specific date/time forms; it rejects comma grouping,
dollar currency, fullwidth digits, and control/NBSP/other Unicode whitespace.
A missing-year date uses the current host year. The original generic trimmed Rust float parser lacked this grammar and overaccepted
whitespace. The separately scoped shared ASCII grammar candidate reduces the 720-row
packet from 288 differences to 64; those remaining cases still require broader
locale/date/time preparation. The replay file has
two explicitly ignored reproductions for this open semantic lane; invoke them
with `--ignored` during repair. No current-year constant is embedded in ADDRESS.

## Formal binding and checks

The Lean model now covers signed bounds, tolerant-floor thresholds, A..XFD column
labels, raw UTF-16 prefix validation/escaping, bounded body fragments, integer
atomic append, scientific precision selection and second decimal rounding.
It takes the versioned sheet-name classification as an explicit prepared adapter
input, backed by the class evidence and composed-name replays. The old
single-name quotation placeholder has been removed. Its executable theorems
compile with `lake env lean OxFunc/Functions/ReferenceMetadataFamily.lean`.

This aligns the repaired rendering substrate. It does not resolve the open
numeric-text/date preparation seam or establish full-function identity.
