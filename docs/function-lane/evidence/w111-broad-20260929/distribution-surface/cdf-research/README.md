# Normal CDF wrapper and ERFC dependency

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`.

Open lanes: ERFC numerical backend, broader distribution normalization and
contexts, GAUSS tiny branch, platform coverage. This is not whole-function parity.

The retained 9,606-row NORM.S.DIST packet contains 4,803 CDF observations and
4,803 density observations. Original CDF production agrees on 4,468 and differs
on 335. Existing public NSWC, Cody and CDFLIB rational packets, plus a public
continued fraction and explicit arithmetic lifetimes, were compared without
fitting coefficients. The best initial composition agrees on 4,549/4,803;
every candidate remains refuted. No ERFC backend replacement was made.

Direct public ERFC and ERFC.PRECISE captures agree on all 544 inputs. Repeated
NORM.S.DIST outputs agree with the prior capture on all 463 selected inputs.
Composing the published ERFC at native-binary64 `abs(x)*RN(1/sqrt(2))` agrees
on 462/463. The remaining input is `0xc040049695c7ac92`: native multiplication
gives z bits `0x4036a71b83b8015f`, while RN64 multiplication followed by a
binary64 store gives `0x4036a71b83b8015e`. The published ERFC at the latter value
reproduces the NORM result exactly. That row also has a local ERFC backend error;
fixing its wrapper does not by itself eliminate its production mismatch.

`z-wrapper-candidate-freeze.json` records the resulting wrapper hypothesis
before fresh discriminators were captured. The generator selected 128 inputs
solely because native and RN64 multiplication disagree, from 460,113 seeded
arithmetic draws, then added 128 fresh controls and both signs. Independent
public-ERFC composition gives:

| Wrapper | Native z | RN64 then binary64 z |
|---|---:|---:|
| NORM.S.DIST | 379/512 | 512/512 |
| GAUSS ordinary branch | 470/512 | 512/512 |

The production CDF wrapper now uses that staged multiply. Its ERFC call,
half-probability/sign composition and tiny-result publication remain unchanged.
The distinct GAUSS tiny helper remains unchanged. The executable Lean model
binds the wrapper with supplied multiply and ERFC primitives, including the
original public intermediate counterexample; focused family builds pass.

The local ERFC backend remains openly inaccurate. Full NORM.S.DIST still has
335 mismatches among 9,606 observations. The new targeted packet has 236 local
NORM.S.DIST mismatches and 60 local GAUSS mismatches among 512 each, despite the
public-dependency composition agreeing everywhere. These are pinned explicitly
in production replay, never counted as matches. `initial-*` retains the old
source and rejected public-packet research; `z-heldout-*` retains all fresh
inputs, direct oracle answers, intermediate comparisons and current outcomes.

The ERFC implementation still contains an older fitted correction. This campaign
does not add coefficients, tune that fit, or infer Excel internals. Further work
must distinguish published algorithms and arithmetic graphs through new public
observations. The direct 544-row packet refutes every raced public packet;
the best agrees on 163, versus the existing backend's 54 on this deliberately
failure-focused set. Those counts alone do not justify replacing the backend.
