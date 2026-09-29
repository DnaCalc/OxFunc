# Broad live Excel campaign, 2026-09-29

This evidence belongs to W111 and bead `oxf-mwue.28`. The user's declared scope
is a broad sweep, prioritizing functions without Excel comparisons, followed by
repairs and more informative probing of unresolved differences. Execution state
is in `.beads/`; this directory records observations and their limitations.

`scope_completeness: scope_partial`; `target_completeness: target_partial`;
`integration_completeness: partial`.

Open lanes: unresolved numeric and typed discrepancies; reference/host coverage;
fresh held-out replay after each changed candidate; formal alignment; canonical
ledger/catalog reconciliation; repository-wide validation. No whole-function
completion claim follows from a passing subset.

## Oracle and reproducibility

- Live Excel 16.0 build 20430, 64-bit, workbook Compatibility Version 2, 1900
  date system. The update channel is not verified. Each retained capture records
  its actual profile and UTC time; alternate builds/locales are not covered.
- Source baseline: `11b23504fe8180d08397be5435bd215e1bbb9722`. Captures and
  candidate checks made after repairs use the dirty working tree, with candidate
  hashes in the family records. Excel is owned by the root runner through COM.
  One accidental overlap between two typed captures is retained as provisional;
  both were repeated serially with identical observable outputs. Only the serial
  repetitions count toward the independent evidence.
- Only public Excel interfaces and typed workbook inputs are used. No binary
  inspection, proprietary sources, decimal-literal substitution, or tolerance
  is used to establish equality.
- `numeric-input-manifest.json` identifies the deterministic input generator,
  seed, row counts and hashes. `numeric-outcomes.json` retains every numeric
  discovery answer and its source/profile data. Run
  `python smart-fuzzer/tools/w111/retain_numeric_campaign.py --replay docs/function-lane/evidence/w111-broad-20260929 OUT`
  to reconstruct the full WitnessSets. A rejudge of all reconstructed sets was
  byte-identical to the original baseline judgement (SHA-256
  `b22f9977d4ded1445c18a844d2331a2df71f7d527a37d8cd81d996e717293633`).
- The baseline executable SHA-256 was
  `94753fdbe0cd688756fe7c02b30a370930e77b6f90a1fff1e375a61f74dbc4ae`.
  Its raw comparisons are in `baseline-judgement.json`.

## Breadth and exclusions

The first numeric sweep captured 140,114 rows across 113 functions. The typed
sweep captured 998 cases across 156 functions, including 39 previously marked
unverified. A second numeric wave added 83,245 observations over 72 surfaces;
its first 68 files and four subsequent nullary captures are retained separately
under `extended-numeric/` and `extended-nullary/`. The union of these discovery
waves covers 288 distinct functions. The locale packet adds six further
provider-qualified surfaces. These are discovery corpora, not whole-domain claims.

`numeric-ingress.json` shows that this Excel host stores negative zero and
subnormal `Value2` inputs as positive zero. Such rows remain in the evidence,
but are withheld from function verdicts because the two evaluators did not
receive identical input bits. Normal inputs with differing output zero signs
remain genuine mismatches. Bulk `ISREF` rows are also withheld: the worksheet
formula sees a cell reference whereas the local numeric batch receives a value.
`baseline-qualified.json` identifies every withheld row and every admitted miss:
138,523 admitted rows, 126,637 agreements, and 61 functions with admitted misses.

The initial typed run exposed a local resolver bug: ranges were wrapped as a
single cell and enumeration coordinates were zero based. The repaired resolver
preserves matrix shape, individual typed cells and one-based coordinates, and
supports contained A1 subranges. The repeated unchanged-input run
`w111-broad-typed-20260929-002` originally reported 854 exact agreements. A second
harness defect was then found: the JSON parser rounded 2,918 of 10,000 normal
binary64 source patterns differently. Enabling `serde_json/float_roundtrip` makes
all 10,000 ingress controls exact. Replaying the unchanged 998 typed cases with
that parser gives 884 exact agreements. `snapshot-1337/report.json` explicitly
withholds missing host/metadata providers, unqualified lazy-evaluator routes and
1×1 array publication differences before updating function verdicts.

`typed-ingress.json` contains the 19-case transport check. The generic runner now
verifies numeric bits, ordinal text, logicals, blank cells and worksheet error
codes immediately after writing fixtures, and records source/profile hashes.
Quoted text is excluded from the unsafe long-numeric-literal guard. This removed
273 false harness rejections from the radix typed holdout; its fresh 1,950-row
capture then matched entirely. Digest and text comparisons now use ordinal
equality, including array digests. Different output zero bits remain mismatches.
The shared helper suite has 48 passing checks, and seven checks exercise the
actual typed runner's ordinal comparison definitions.

The 13:37 snapshot updates 273 ledger rows, preserving earlier unresolved
divergences. Its numeric replay agrees on 131,704/138,523 qualified first-wave
rows, compared with baseline 126,637/138,523, and 80,018/83,241 second-wave rows.
Those counts describe a changing working-tree candidate, not the final campaign.
The ledger now records 260 divergent, 219 consistent and 48 unverified functions.
The separately frozen `snapshot-1735/` records later candidates and 354 source
hashes. It updates 277 ledger rows: 132,624/138,345 admitted first-wave numeric
rows, 81,455/83,245 second-wave rows, and strict typed qualification from 3,542
discovery observations. Its admission denominator differs from the first
snapshot because unsupported WEEKNUM selectors are now explicitly withheld
after contradictory repeated oracle observations; four nullary captures are
also admitted in the extended wave. The ledger remains at 260 divergent, 219
consistent and 48 unverified functions because prior residuals are preserved.
These figures describe the frozen candidate, not subsequent endpoint repairs.

The later elementary packet records exact fresh LOG 12,671, FISHER 24,600
and MEDIAN 31,611 observations. POWER's first 23,264-row holdout failed 38;
the width/signed-scale refinement subsequently matches that bank and a new
2,978-row independent packet. HARMEAN and DEVSQ repairs independently match
9,000 and 10,935 fresh rows. Switched VDB research was translated after its
20,000-row independent publication bank passed; the production translation
then matched 8,000 new numeric and 522 new typed observations. These family
results postdate snapshot 1735 and preserve their original failed candidates.

## Repair evidence

Family subdirectories retain observations, candidate hashes, exact dispatch
replays and known residuals. Discovery and held-out corpora remain distinct;
a failed held-out corpus becomes discovery evidence for a subsequent candidate.
Passing old rows after a repair does not replace a fresh independent holdout.

- `elementary-publication/` and `power-integer-thresholds/`: LOG/FISHER stored
  arithmetic, MEDIAN midpoint publication and POWER width/conversion branches;
  `power-consumer-audit/` exercises unchanged financial consumers.
- `aggregate-publication/` and `aggregate-prepared/`: HARMEAN reciprocal-average
  arithmetic, DEVSQ per-square publication and direct scalar error priority.
  Their numeric bank has 39,890 exact observations; a separate shared MEDIAN,
  HARMEAN and DEVSQ bank adds 2,190 fresh prepared observations, all exact.
- `mround/`: staged quotient/product and an empirically bounded halfway cutoff
  match 47,964 numerical observations. Prepared repairs retain their failed
  independent reference bank; multi-cell reference admission is being refined
  separately from array-value lifting.
- `mod-publication/`: endpoint normalization and tiny-remainder publication
  match 151,438 numeric observations, including 10,784 fresh production cases.
  The first tiny-result research candidate failed 62 independent observations;
  its failed freeze remains retained. Prepared Missing behavior is a separate
  refinement and receiving-side obligation.
- `logical/`: 540 AND/OR/XOR typed rows agree; separate spelling and precedence
  observations and exercised Lean bindings document the existing runtime rule.
- `fact/`: negative fractions reject before truncation; 1,209 discovery and
  1,509 independently generated held-out rows agree through surface dispatch.
- `bitwise/`: integer admission, shift bounds and explicit missing arguments.
  The promotion audit retains 34,246 qualified numeric and 862 typed original
  observations, but later shared-parser evidence has 50 mismatches among 240
  admitted text cases. Those current-baseline failures prevent promotion.
- `address/`: coordinates, conversion boundaries, style/error rules and sheet
  text are under active investigation.
- `complex/`: decimal presentation and shared formatting; independent probing
  has exposed an additional rounding boundary, retained as a failure.
- `radix/`: BASE bounds and engineering places validation; typed parsing and
  ordinary numeric and typed heldouts pass, while the newly discovered shared
  numeric-text/context rules remain open.
- `sln/` and `depreciation/`: SLN arithmetic and DB/DDB/VDB admission, period and
  operation-graph observations. Failed holdouts remain distinct from subsequent
  independent tests.
- `dates/`: serial bounds, half-second extraction, integer conversion and 1900
  rollover. Repeated unsupported WEEKNUM selectors produce contradictory oracle
  outputs, retained as an unresolved behavioral lane rather than fitted answers.
- `numeric-text/` and `numeric-text-analysis/`: 2,544 exact-ingress cases across
  53 functions separate 1,182 shared parser differences, 24 NOT logical-policy
  differences and two ACOT kernel differences. The narrow grammar candidate is
  undergoing independent refinement; regional/date context remains open.
- `locale-context/`: actual OxFml production parsing/formatting under the captured
  host profile, with 205/301 qualified matches and 57 explicit host-context or
  name-availability exclusions.
- `atan/`, `acot/` and `asin/`: reduced-angle and staged inverse-trigonometric
  candidates retain 181,495 numeric plus 841 typed ATAN matches, 17,146 ACOT
  matches, 22,192 ASIN matches and 14,232 ACOS matches. Each record distinguishes
  discovery, failed candidates and fresh independent observations.
- `integer-publication/`: INT matches 123,229 retained observations including
  31,210 fresh inputs after its last refinement; ISEVEN/ISODD fresh banks match
  36,200/36,150. PERMUTATIONA matches the second independent 25,321 numeric bank
  after a decimal-power refinement, and 85 discovery plus 252 fresh prepared
  observations after omitted-argument and positional-padding repairs.
- `gcd-lcm/` and `quotient/`: GCD/LCM have 13,742 numeric plus 2,078 typed
  matches; QUOTIENT has 3,141 numeric plus 504 typed matches. Aggregate order,
  blank origins and declarations remain receiving-side review obligations.
- `numeric-to-text/`, `text-slice/` and `count-family/`: generic rendering and
  explicit serializers retain passing independent banks, while 48 exceptional
  initial-precision discrepancies remain. Text slicing has 34,267 admitted
  matches; the count family has 2,238. NUL transformations and formula-entry
  failures remain distinct transport limitations, not worksheet errors.
- `distribution-surface/`: focused replays retain 8,484 typed observations
  (7,815 exact, 647 numeric discrepancies, 22 input limitations) and 38,510
  numeric observations (36,672 exact, 1,838 discrepancies). Density graphs match
  19,094 observations; normal CDF and discrete interior kernels remain partial.
- `depreciation/`: DB, DDB and no-switch VDB have separately frozen independent
  numeric evidence. SYD's third independent bank matches 21,526 observations;
  the financial positional-padding bank matches 584. Switched VDB production
  matches 113,329 numeric and 522 fresh typed observations. Its DB/DDB/no-switch
  consumer controls add 71,760 unchanged exact numeric observations. Huge-start
  progress failures remain open; safe small-endpoint controls do not discharge
  them.
- `hyperbolic/` and `tanh-coth/`: COSH matches 32,090 initial and independent
  numeric observations. The TANH denominator refinement sharply reduces
  discrepancies, but a fresh bank exposes 52 TANH and 50 COTH failures. Direct
  SINH/COSH controls locate all 52 TANH failures in the small-input SINH path.
- `rounding-boundaries/`: count conversion and decimal scaling explain the
  ordinary retained banks. Fresh endpoints expose finite subnormal output and
  numeric-kind payloads in the IEEE nonfinite encoding range. Separate worksheet
  controls and two-session repetitions retain their observable behavior; the
  earlier conflicting TRUNC MAX capture remains unresolved.

HO-FN-022 and blocker `oxf-mwue.28.8.1` record the unresolved OxFml preparation
context/acknowledgment dependency. Filing a handoff is not integration. The narrow
shared grammar changes preserve signatures and argument origins, but still need
receiving-side acknowledgment before cross-repo promotion. HO-FN-023 through
HO-FN-030 separately record conditional preparation, text/compatibility,
generic number rendering, distributions, aggregate ordering, positional
arguments, endpoint payload publication and elementary/aggregate preparation.
Each has an open receiving bead;
filing a packet is not receiving integration.

Validation history remains in the family records. The combined Rust run recorded
at 19:22 has 1,992 passing tests, three failing tests and eight ignored tests
across 60 targets. Failures are the retained NEGBINOM one-ULP difference and two
normal-CDF replay tests. The stale conditional preparation fixtures, metadata
newline and rounding zero-sign checks from earlier runs now pass. Full Lean
passes 526 jobs after correcting a Windows generated-cache module-name casing
collision. Subsequent source refinements require renewed validation; the final
verification record identifies its exact source snapshot. No campaign or
whole-function completion claim is made.
