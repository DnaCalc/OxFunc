# DB, DDB and VDB: W111 characterization

Tracking: BUG-FUNC-058 / BUGREP-036, bead `oxf-mwue.28.9`. These are public
COM/Value2 observations from Excel 16.0 build 20430, 64-bit, workbook Compatibility
Version 2, 1900 date system, precision-as-displayed disabled. Channel is unavailable.
Independent cohorts admit normal binary64 operands and positive zero; source
negative zero and subnormals are excluded because this host changes those during
Value2 ingress. Comparisons preserve exact result bits, error identity and ordinal
text; no result normalization is used.

The primary capture originally matched DB 593/1206, DDB 691/1206 and VDB 40/1205.
DB/DDB each received separate discriminators, candidate v1, a first 5250-row
independent cohort, refinements, candidate v2, a second 5300-row independent cohort,
further boundary probes, candidate v3 and a third independent cohort. Every
formerly failed heldout is discovery evidence after adapting to it. The first
judgements and exact source snapshots are retained, including recovered v1 source
whose bytes match its original frozen SHA-256.

DB candidate v8 matches 93,616 captured discovery rows. The third cohort was 7891/7904 against v3: twelve errors
located the strict upper integer POWER overflow branch at exponent 2^32-1; one
two-ULP mismatch distinguished extended book subtraction. The fourth cohort was
7473/7475 against v4: two extreme-life cases distinguished internal NaN from
ordinary POWER underflow/overflow. A 452-case discriminator established zero-rate
publication on that NaN route. The fifth cohort was 9850/9851 against v5: its last
two-ULP case identified first-period multiplication. A subsequent offline graph
search selected 100 distinguishing inputs each for first cost-rate multiplication,
first fraction multiplication and final rate multiplication; all 300 live results
confirmed x87 stores. The sixth cohort matched 21796/21797 against v6; its final
witness showed that subnormal annual depreciation is published as zero before
updating the book. A 720-case discriminator supported this rule for first and
regular periods. Another 744-case large-exponent rate-half discriminator exposed
the unsigned 32-bit power dispatch boundary even when the power stays finite.
All six earlier DB heldouts are now discovery. Candidate v7 was frozen before a
fresh 29,807-row seventh cohort. It matched 29,802 rows; all five failures involve
minimum-normal costs and final partial-year publication. A separate 300-case
discriminator distinguished subnormal final quotients from subnormal rate
products. All 300 observations rejected the old graph and confirmed publication
at both stages. Candidate v8 matches all prior rows and is frozen before a fresh
35,783-row eighth independent cohort, which matches 35,783/35,783 exactly. The seventh
cohort remains discovery with its first failure judgement unchanged.

DDB candidate v4 matched 20,934 discovery rows and 7997/7997 fresh independent rows.
Its previous independent third cohort matched 7984/7984, but a later VDB witness
exposed one further x87 product store; separate DDB annual-value probes confirmed
it. That earlier successful cohort was consequently replayed as discovery against
the new candidate, followed by the fresh fourth cohort. Both source snapshots and
first judgements remain available. A subsequent 304-case large-period sweep
matched only 171 rows against v4. Ninety gross differences identified the same
unsigned 32-bit dispatch boundary as DB; the remaining 43 differences of one to
three ULP identified x87 stores in integer-power accumulator products. The
shared private depreciation helper now matches all 29,235 DDB discovery rows.
Candidate v5 was frozen before a fresh 13,093-row fifth cohort. It matched
13,092 rows; the remaining large fractional-period witness distinguishes an x87
base subtraction from binary64 subtraction. A 300-case independent operation
selection confirmed x87 subtraction: 82/300 matched the old graph and 300/300
match the refined graph. Candidate v6 matches all 42,628 discovery rows and is
frozen before a fresh 19,002-row sixth independent cohort, which matches
19,002/19,002 exactly. The successful fourth cohort
remains retained as discovery for v5; the fifth cohort's first failure judgement
is also retained.

DB keeps fractional life, truncates month, and converts whole period/life through
unsigned-32 saturation followed by signed interpretation. Positive periods below
one request initial depreciation. The rate uses existing numeric power and Excel decimal
ROUND substrates; integer-path overflow below 2^32-1 publishes zero, whereas
fractional/oversized exponent overflow is numeric error. Internal ratio overflow
or subnormal publication produces zero. The first-period products, book recurrence products/differences, and final rate
product use x87 stores. First/regular amounts and the final quotient/rate-product
stages publish subnormals as positive zero before subsequent arithmetic. First depreciation is `(cost * rate) * (month / 12)`;
the final partial year is `((book / 12) * rate) * (12 - month)`.

DB and DDB use a private bounded integer-power helper: exponents below
`2^32-1` use square-and-multiply with x87 product stores. Larger integers and
fractional exponents use the existing positive power chain. This differs from
the worksheet POWER wrapper's wider integer range and binary64 products. DDB's
internal division publishes a subnormal rate as zero before multiplication by
cost. Its base subtraction, book product, and declining product use x87 rounding. Declining-product overflow is observed
before salvage clipping, including cost below salvage. A generic interval
integrator or an early cost<=salvage shortcut fails captured cases.

Typed discovery (107 cases) and optional-argument discrimination (44 cases)
replay 151/151 against the latest candidate and exact JSON ingress. Required
explicit missing positions become zero; optional missing/blank positions also
become zero, while absent optional positions select defaults. Arrays lift
componentwise. VDB admits factor zero, rejects end beyond life, accepts TRUE/FALSE
logical text for no_switch, and rejects numeric text in that flag. The shared
numeric-text grammar remains a separate coercion lane, outside these 151 cases.
A new 369-case independently generated typed cohort matches 369/369 against the
frozen source: DB 116, DDB 116, and no-switch VDB 137, with every row admitted
on both sides. It includes separately declared area fixtures and row/column
broadcasts and limits numeric text to simple ASCII decimals. The independent
Python judgement uses strict typed digests and checks execution status before
counting any match.

VDB no-switch candidate v1 matched 1230 discovery rows and 6351/6469 fresh rows.
Adjacent interval endpoints exposed its final-fraction calculation:
`1 - (ceil(end) - end)` differs from `end - floor(end)`. Separate partial products
can even publish a tiny negative result for increasing adjacent endpoints; those
observed bits are retained. A final one-ULP case isolated the same annual declining
product subsequently confirmed independently in DDB. Refined candidate v2 matches
8299/8299 discovery rows and 6473/6473 fresh independent rows. The shared integer
power refinement later replayed all 6473 rows exactly; that replay is discovery
for the new arithmetic candidate. Frozen v3 then matched 8472/8473 independent
rows. One adjacent-endpoint cancellation revealed extended-precision stores of
the two partial products: 800 distinguishing probes match the extended stores,
while ordinary binary64 products match 785. Refined v4 replays 24,045 observations
exactly across six retained cohorts (the raw count can contain repeated discovery
tuples). A fresh 11,972-row independent cohort matched 11,969: the three residuals
distinguished publication of subnormal partial products from final subtraction.
The 660-row discriminator confirms that each product flushes subnormals to zero,
while subtraction of normal products may return a nonzero subnormal Value2.
Candidate v5 matches 36,677 discovery observations and then 16,975/16,975 fresh
independent observations. Neither the oracle nor local output is normalized in
these comparisons. The failed v3 and v4 cohorts remain retained as discovery.
A separate [large-index progress defect](VDB_LARGE_INDEX_PROGRESS.md)
remains: adding one to a binary64 annual index at or above `2^53` can leave the
index unchanged, so dangerous cases are restricted to oracle-only capture until
a supported progress repair exists. The [switched VDB graph and failed-candidate
history](VDB_SWITCHED_GRAPH_RESEARCH.md) now have an explicit production binding:
113,329 numeric observations match, including a 20,000-row frozen publication
cohort and fresh 8,000-row dispatch cohort; 522 fresh typed cases also match.
These period endpoints remain below 1000. No full VDB parity claim is made.

SYD has a separate [evidence story](SYD.md) under BUG-FUNC-067 / oxf-mwue.28.17.
Its sign, arithmetic-stage, missing-argument and array-lifting observations are
separate from the DB/DDB/VDB results above.

Fresh SYD v3 independent validation matches 21,526/21,526. Financial positional
padding discovery matches 75/75 and fresh independent controls match 584/584;
see [the boundary assessment](POSITIONAL_PADDING.md). These observations preserve
known shared numeric-text and receiving-dependency limits.

Validation currently includes four switched VDB tests with 44 numerical witnesses, eighteen reduced Rust depreciation parity tests, five SYD tests,
one financial padding test, eight existing family tests, three SLN tests and a passing Lean family model. The model names DB
host-count conversion, rational book scheduling, internal stage publication,
DDB rate/period rules, VDB no-switch partial-product stores, and switched comparison/publication/step routes. POWER and IEEE
rounding remain dependencies on existing substrates; this is not a duplicate
proof of every IEEE operation. Non-x86_64 publication and alternate Excel/version
axes remain unvalidated.

`retention-manifest.json` maps compact lossless oracle/case artifacts to original
and retained SHA-256 hashes. `candidate-source-snapshots.json` preserves exact
UTF-8 source including original line endings. No session transcripts are stored.

Status: in progress. `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Open lanes: VDB large-index progress and periods beyond the bounded switched numerical evidence; full shared coercion;
receiving-dependency acknowledgement/integration for positional padding;
alternate baseline axes; non-x86_64 numeric publication.
