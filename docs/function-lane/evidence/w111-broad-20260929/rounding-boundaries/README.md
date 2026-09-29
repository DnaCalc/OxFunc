# Rounding counts, decimal widths and decimal-pair assembly

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: raw overflow numeric publication and historical TRUNC MAX conflict, exceptional initial15 precision
from numeric-to-text/precision.json, independent validation after refinement,
shared contextual text preparation, and evaluator declaration review.

The frozen candidate in original-frozen-candidate was evaluated before any
refinement. On 12,000 independently generated normal-number/count pairs per
function, TRUNC and ROUNDDOWN each matched 9,925, ROUNDUP matched 9,994 and ROUND
matched 8,559. All failures remain in frozen-independent-judgement.json. These
observations are discovery evidence for the subsequently refined candidate.

The further 2,130 count/width discriminators per function establish distinct
integer preparation rules. ROUND truncates the absolute count, saturates it at
2^31-1, and reapplies the sign. It combines that signed count with the initial
decimal-point position in a wrapping signed32 carrier before decimal reduction.
TRUNC, ROUNDDOWN and ROUNDUP instead saturate the absolute count at 2^32-1,
retain its low16 bits and reapply the original sign. In their decimal truncation
stage, a low16 magnitude above 32767 substitutes 100. ROUNDUP still computes
its added decimal unit from the original low16 magnitude. Thus count 65537
behaves as one, count 32768 uses the truncation fallback, and an added unit can
overflow or underflow separately from the truncated value.

Every decimal pair is assembled with the already evidenced RN64 base16 scaling
graph from the numeric-text parser: ascending exponent chunks (units, sixteens,
256s), each power and multiply rounded to 64 binary significand bits, followed
by binary64 publication. Subnormal output from this ordinary assembly becomes
positive zero. Directed modes have a separate initial-normalization exception:
when the initial retained pair's normalized binary exponent is -1023, they
publish its raw fraction bits before using the requested count. This returns
0x000ffffffffffffa from the minimum normal input; its negative counterpart
preserves the sign. The shared arithmetic components were extracted without
changing the parser or PERMUTATIONA publication policy. ROUNDUP adds the two
ordinary assembled values in binary64. The narrow
crate-private coercion::scale_decimal_pair_to_binary wrapper exposes this
existing arithmetic to bounded numeric callers without imposing text grammar.
The parser's algorithm and lexical admission rules are unchanged. The shared
generic text formatter's initial15 conversion is also unchanged.

The refined candidate matches all 102,472 numeric rows retained in the four
numeric-*.json files, including the formerly failed 48,000-row packet, all 8,520
new count controls and 45,952 prior threshold/refinement controls. Raw captures
use hexadecimal input and output bits. The production-dispatch replay test
w111_rounding_boundary_live_replay exercises each function. No independently
captured post-refinement success was inferred from those discovery rows.

The subsequent 27,120-row independent packet is retained as refined-heldout-*.json.
Its final frozen sources were unchanged at production replay. ROUND agrees on
6,770 finite/error rows with 10 additional unresolved numeric payloads;
ROUNDDOWN and TRUNC each agree on 6,765 finite/error rows with 15 payloads each.
ROUNDUP agrees on 6,779 of 6,780, exposing a new tiny-difference counterexample:
ROUNDUP(1.0000000000000502e-298,305) returns 1e-298 while the candidate adds a
1e-305 unit. The initial decimal value minus its truncated value is subnormal;
a dedicated discriminator packet rejected simple subnormal subtraction flushing.
The subsequent signed residual ladders support the local upper16 predicate in
SIGNED_RESIDUAL_RULE.md; the former independent failure is now refinement evidence.
The frozen failed row and all raw payloads remain in
refined-independent-production-judgement.json. This is partial validation,
not a full function claim.

The older broad TRUNC packet contributes explicit counterevidence. Its admitted
minimum-normal inputs return a slightly subnormal 15-digit value even when
ordinary truncation at the requested count would be zero. Its admitted maximum
normal inputs return a lower finite 15-digit decimal where this candidate gives
NUM. There are 22 such rows, retained in refined-candidate-judgement.json; the
normal source values are not ingress exclusions. Fresh endpoint-neighbor probes
and 224 worksheet controls plus 960 repeated controls in two independent Excel
processes instead publish raw nonfinite-range numeric encodings at the upper
endpoint. All input bits are exact in the repeated controls. General/scientific
formats, two calculation paths, two formula APIs and dependent recalculation
all agree; the older conflicting capture remains unresolved counterevidence.
See ENDPOINT_IMPACT_ASSESSMENT.md and endpoint-refinement-analysis.json. The
finite directed subnormal repair addresses 672 endpoint rows; the 572 raw
overflow payload rows remain explicitly outside the passing production slice.
All 14,224 endpoint observations remain retained without reclassifying payloads.
Separately, the exceptional initial15 midpoint bank still exposes
the shared decimal normalization gap. These lanes prohibit a full function claim.

The cross-function initial15 transfer retained as directed-initial-precision-*.json
adds 2,136 normal-source observations per directed function. The unchanged
candidate differs on 212 rows in each function (53 positive centers with two
signs and two requested counts); all three directed functions agree with each
other in Excel. The original generic ROUND precision bank still has 34 modeled
differences after the decimal assembly repair. These precision observations are
separate from the signed-residual rule, whose fresh 10,012-row independent
packet passes production replay. No initial normalization lookup or per-case
fallback has been added.

DecimalRounding.lean gives an executable Float/bit-carrier model of the current
finite numeric candidate, including the newly observed initial subnormal. It binds existing exact binary64 decoding,
initial15 decimal normalization, count conversion, signed wrapping, staged
decimal assembly and binary64 addition. ROUND, ROUNDUP, ROUNDDOWN and TRUNC
import this substrate. The focused 13-job build and 22 round-filtered Rust
library tests pass. The old exact-rational ROUND helper is explicitly identified
as an ordinary-count reference, with roundExecutable as the runtime binding.

The separate prepared-value repair is documented in PREPARED_IMPACT_ASSESSMENT.md.
All four functions now map explicit Missing to zero and preserve positional NA
padding through left-to-right coercion. A rounding-local helper provides this
policy without changing generic adapters. ROUND's stale UnaryNumericScalarOnly
declaration changed to Custom; the other three were already Custom. The prepared
candidate agrees with 690 admitted discovery rows and 928 independently captured
rows, with immutable candidate hashes verified before subsequent arithmetic
refinement. One rejected TRUNC() formula remains an explicit harness exclusion.
The parent owns canonical status, issue and handoff records, including the
HO-FN-028 extension and unresolved HO-FN-029 overflow publication dependency.
