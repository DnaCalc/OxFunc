# Initial decimal precision research

Status: `in_progress`; `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Open lanes: initial15 arithmetic, exceptional numeric publication, the older
conflicting TRUNC endpoint capture, contextual numeric text, receiving-repo
acknowledgment. This note records offline hypotheses, not a runtime repair or
a function completion claim.

The directed transfer packet contains 2,136 observations each for ROUNDDOWN,
ROUNDUP and TRUNC: 534 positive centers, both signs, and two adjacent counts.
The three functions agree throughout the capture, and both counts return the
same result for every center/sign pair. All sources are normal binary64 values.
The current 31-significant-decimal-digit intermediate followed by initial15
half-away normalization differs on 212 observations per function, corresponding
to 53 positive centers. These observations remain retained unmodified.

A uniform alternative scales the exact binary64 input by ascending low-nibble,
middle-nibble and high-nibble powers of ten. Powers and each multiplication use
nearest-even arithmetic with 64 significand bits. It rounds the resulting
15-digit-scale value to an integer, with exact halves away, then uses the
already-evidenced decimal assembly rule. This alternative differs on 18 of the
534 positive centers. The residuals include exact scaled halves that Excel
rounds down and values one 64-bit-significand ULP below a half that Excel rounds
up. Neither a uniform tie-mode change nor a simple epsilon follows from them.
This description is a reproducible arithmetic hypothesis, not an assertion
about Excel's implementation.

The same alternative preserves every currently matching result across 48,418
earlier ROUNDDOWN observations. Both models retain the same 95 raw-publication
residuals in those banks. That transfer is useful counterevidence against an
obvious broad regression; it does not resolve the 18 precision centers. Generic
ROUND also has a distinct policy: applying this directed graph with half-down
integer rounding leaves 72 positive-center differences in its precision bank.
No common initial-normalization primitive has been established.

The retained uniform model searches cover stage precision and rounding mode,
ascending/descending binary and nibble decompositions, fixed intermediate
decimal magnitudes, independently constructed powers and reciprocals, grouped
power products, and fixed extra-digit extraction. Across 1,242 tested model
configurations, none improves the 18-center result. No per-input or per-exponent
selection has been added to production.

Relevant reproducible tools are:

- `smart-fuzzer/tools/w111/analyze_initial15_directed_precision.py --directed`
- `smart-fuzzer/tools/w111/analyze_directed_initial_split_scaling.py`
- `smart-fuzzer/tools/w111/analyze_directed_initial_power_construction.py`
- `smart-fuzzer/tools/w111/analyze_directed_initial_digit_extraction.py`

The JSON artifacts beside this note preserve rankings, residual IDs and prior
bank comparisons. `directed-nibble-initial-residuals.json` preserves the exact
scaled fractions at the 18 centers. Grouped-product and prior-bank comparisons
are research artifacts; the ordinary runtime replay tests remain authoritative
for the current production candidate.

The subsequent discovery packet is frozen at
`smart-fuzzer/runs/w111-directed-initial-discriminators-20260929`. Its generator
uses seed 202609293036 and selected 96 fresh decimal-midpoint centers from
385,651 draws where the two leading models disagree. The 18 reused residual
centers are a separately labeled cohort. Both cohorts include two adjacent ULPs
on either side, both signs, and two adjacent counts: 2,280 probes for each of
ROUND, ROUNDDOWN, ROUNDUP and TRUNC. It is deliberately model-discriminating
discovery, not an independent validation claim. No new oracle output was used
to select it. Production arithmetic remains unchanged after the new observations.
Among 2,280 rows per directed function, the current helper differs on 388 while
the nibble hypothesis differs on 100. The latter consists of all 18 reused
residual centers plus seven new centers, each with both signs and both requested
counts. ROUND differs on 48 rows with the current helper and 20 with the nibble
half-down hypothesis; that improvement does not erase its distinct earlier
counterevidence. `initial-discriminator-model-judgement.json` preserves every
failed row for both models, and `initial-discriminators-*.json` preserve all
9,120 raw observations.

Combining the two directed packets gives 1,078 distinct positive binary64 inputs;
all repeated observations agree. An additional 216 uniform low/middle/high
table-policy configurations and 78 mixed power/stage-precision configurations
still leave 25 positive-input differences at best. No production replacement is
justified by these results. The current candidate and both failed model lineages
remain available for further research.
