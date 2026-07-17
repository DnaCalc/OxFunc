# The exp that wasn't a library (a field diary)

*W109 G3-01, 2026-07-17 session 5. Live oracle: Excel 16.0 build 20131,
bulk recalc engine. The two-session hunt for "which exp does the gamma
kernel call" — and the answer that made every candidate wrong for the
same reason.*

## The setup

After the GRATIO identification landed, one residual on the gamma series
path refused to close: at a=2 (normalizer exactly 1, series staging
proven), every remaining miss was exactly a ∓1-ULP deviation of
`exp(t1)`. Excel's exp was *one-sided low* — never above the correctly
rounded value, below it on 20 of 45 decoded rows. That smelled like a
particular library: fdlibm's rounds-low bias matched 28/45, better than
CR's 25/45. So the hypothesis solidified: "a 2010-era statically-linked
CRT exp with a rounds-low bias." Sessions of racing followed: fdlibm
variants, Cephes, UCRT, the x87 fFEXP chain — nothing beat 28.

## Act I — kill the CRT with real binaries

An agent lane finally did what proxies couldn't: called `exp()` in the
*actual DLLs*. A 32-bit harness probed msvcr100 (the 2010-03-18 binary),
110, 120, msvcrt, the x87 fallback via `_set_SSE2_enable(0)` — and, via
a VC90 SxS activation-context manifest, msvcr90 9.0.30729: the exact CRT
generation Office 2010 statically linked. Bonus archaeology: ReactOS's
`libm_sse2/exp.asm` turned out to be AMD's win-libm verbatim (matching
Open64's libacml_mv instruction for instruction), so the x64 lineage got
bit-faithful Python transcriptions too.

Verdict: every Microsoft CRT exp of the era is **one-sided HIGH** (0..+1
above CR) or CR-identical. The mirror image of Excel. The entire "which
CRT" question was dead — no library rounds low like that.

## Act II — 44% is not a bias, it's a mode

The number that broke the case open was sitting in the agent's report:
Excel's exp equals CR−1 on 20/45 rows, CR on the rest. 20/45 ≈ 44% ≈
half. What lands exactly one ULP below correctly-rounded about half the
time? **A truncated result.** floor(true exp) is CR−1 precisely when CR
rounded up — which happens ~50% of the time. Not a worse approximation;
a different *publication rounding*. Every earlier refutation — including
"the x87 chain is CR at double granularity, refuted" — had only ever
raced round-to-NEAREST publications.

One 20-line race later: floor(true exp) scored **38/45**. Ten points
clear of the best library candidate after two sessions of library
hunting.

## Act III — the chop has an address

Swapping a chopped exp into the whole emulator made things *worse*
(416→380 on the 692-row corpora). The win only appears at one call site:
the gser series `r = exp(t1)/Γ`. The a==1 wrapper's `exp(−x)` is
nearest; the continued-fraction path is nearest; the a<1 erf-side paths
are nearest. One compiled function, different rounding treatment per
call site — the same pattern as the a≥1-double vs a<1-extended log
staging found earlier. (Excel probably computes that series r via a
different internal routine entirely, and *that* routine truncates.)

Production landing: a double-double Tang-style `exp_rd` with directed
truncation (validated 0 mismatches against mpmath floor-exp on 25k
points), fed only to the series arm; plus the a==1 exponential-CDF
dispatch the identification had proven months of probes ago but had
never landed inside gratio. CHIDIST 148→152/195, GAMMA.DIST 151→159/268.
Held-out gate (fresh a slices, b20): +3/111 — real, modest, capped by
the fractional-a normalizer that belongs to the G3-02 lane.

## Dead ends worth remembering

- Two sessions of "which library" when the answer was "which rounding
  mode." **Race directed roundings (RZ/RD/RU publications) alongside
  nearest for every unidentified transcendental.** It's one extra
  candidate and it falsifies a whole class.
- The loadable-DLL probes of session 4 tested the wrong bitness — the
  64-bit CRT forwards to different code than the 32-bit static CRT
  Excel actually shipped with. Real-binary probes need the right era
  *and* the right bitness (SxS manifests get you old side-by-side CRTs).
- Hand-converting a 64-entry constant table to bit patterns produced
  exactly one wrong entry. Generate constants programmatically, always.
- The same session also killed the erf "fine comb": three matched
  relative-resolution scans (242k oracle rows) showed every measured
  "period" rescaling with the scan grid — the grid echoing its own step
  through a ~26-43% miss density. A period is only real if it survives a
  10× finer grid. The alias-free residual fingerprint is the per-binade
  phase-gradient (miss probability vs position-in-ULP), and by that
  fingerprint the erf residual is per-row transcendental last-bit noise
  — the same internal exp/log identity problem, not a spatial generator.

## Where this leaves the lane

The gamma series is now: identified structure (GRATIO), identified
staging (Cephes-igam form), identified normalizer (CR-Γ), identified
publication (chopped exp), identified dispatch (a==1). The residual is
the ~2⁻⁵⁶-error approximation *behind* the chop on 7/45 rows, the
fractional-a internal lgamma (G3-02's wall, measurable through this
window), and the erf/beta sub-kernels (beta tail's best hypothesis:
Excel swapped NSWC's grat1 for its own full GRATIO — its own code, not
a foreign library, which after this session should surprise nobody).
