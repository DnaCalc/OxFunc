# LOG, POWER, FISHER and MEDIAN arithmetic observations

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Owner: W111, BUG-FUNC-069, oxf-mwue.28.19.

Excel 16.0 build 20430, 64-bit, Compatibility Version 2, 1900 date system;
channel unverified. Inputs and results use exact Value2 binary64 bits. All
inputs are normal finite numbers or positive zero. Subnormal outputs and typed
errors remain admitted. Capture uses public worksheet interfaces only.

| Function | Discovery | Fresh frozen candidate |
|---|---:|---:|
| LOG | 8,035/8,035 | 12,671/12,671 |
| FISHER | 8,350/8,350 | 24,600/24,600 |
| MEDIAN | 10,590/10,590 | 31,611/31,611 |
| POWER | 20,206/20,206 | 23,226/23,264 |

LOG uses a staged quotient of the stored logarithms and publishes positive
zero. FISHER stages the two separate 1+x and 1-x operations and their quotient;
reusing the first factor to form the second is refuted. MEDIAN uses the sorted
lower endpoint plus half the difference, with a tiny half-difference becoming
zero; overflow publishes NUM. POWER uses staged products for integer repeated
squaring, a positive-base-ten decimal-scale branch, and tiny-positive-power
checking before a negative exponent's reciprocal.

The initial POWER heldout exposes 38 residuals at very large exponents. Its
candidate source and failed judgement remain retained. The subsequent
[width refinement](../power-integer-thresholds/README.md) explains all 38 and
matches 13,622 boundary rows, 1,672 signed-scale controls and a frozen 2,978-row
independent bank. All eight four-family replay tests now pass. The table above
records the first frozen candidate rather than replacing that failed result.
Passing numeric banks do not characterize the whole prepared function surface.
No whole-function promotion follows from this packet.

ElementaryPublication.lean supplies executable bindings for the observed LOG,
FISHER, median midpoint and integer POWER graphs. Its numerical primitive
bindings are empirical; the older rational POWER model does not describe all
fractional negative-base branches. The admitted integer graph and surrounding
model obligations must not be conflated.

Open lanes: wider POWER domains, typed/coercion behavior, numerical
primitive and platform alignment, wider version/locale phases, shared POWER
consumer coverage, and combined campaign validation. The
[consumer audit](../power-consumer-audit/README.md) finds no changed output in
76,611 observations, preserving 317 pre-existing FV/PV discrepancies. XNPV
uses an explicitly qualified dependency-composition replay. The frozen discovery/heldout source copies
and research graph outputs preserve candidate lineage.

## Subsequent prepared-value characterization

The separate prepared packet changes LOG/POWER Missing and positional padding
handling, selects the existing LOG10 kernel for an omitted LOG base, and
preserves MEDIAN's Empty/Missing distinction. POWER's generated route now uses
its tested custom surface. See `PREPARED_IMPACT_ASSESSMENT.md` and
`prepared-validation.json` for the exercised boundary and HO-FN-030 dependency.

The first frozen 832-case prepared bank exposed eight MEDIAN failures; that
failed freeze remains under `prepared-first-heldout`. A 675-case discovery
confirmed direct-scalar coercion-error priority for MEDIAN, HARMEAN and DEVSQ,
while retaining original numerical value order. Their refined eleven-file
freeze is independently validated by a serialized 2,190-case packet, all exact
and with unchanged hashes. Its earlier overlapping-run capture is retained
as provisional; it agrees with the serialized outcomes but adds no validation
count. Direct surfaces and generated dispatch both replay the fresh bank.

Current prepared exact observations are LOG 453, POWER 375, FISHER 142 and
MEDIAN 1,250. The shared aggregate controls add HARMEAN and DEVSQ 1,362 each,
for 4,944 observations across all six families. The optional LOG comparison
adds 4,827 numeric observations. Counts are observations, not unique input
pairs. Earlier failed candidates retain their original judgement. Wider
contextual text, evaluator provenance/resolver failure ordering, numerical
platform bindings and receiving-repository acknowledgement remain open.
