# Theory handoff: the error-function publication group

Date: 2026-08-21
Lane: W109 inverse-problem / calc-graph search
Audience: a swarm of independent agents who will propose evaluation trees,
discriminators, and clean-room probes. This file is the shared prior. It is
not a closure claim.

**Correction pointer (same day):** a theory swarm plus Excel captures attacked
this document. Several sections here are retired. Do not quote §4.4 (tooth
law), the inferred Ext80 `gam1(½)` mantissa `0x906eba8214db6c6f`, T1c, or
`NORMSDIST = 0.5+GAUSS` as current fact. The surviving scoreboard, capture
verdicts, and new hard constraints are in
[`W109_ERF_SWARM_RESULTS_20260821.md`](W109_ERF_SWARM_RESULTS_20260821.md).
Production kernels are still unchanged.

Reference oracle: Excel 16.0 build 20228, x64, Workbook Compatibility Version 2,
numeric inputs through `Range.Value2` / `cell_value2_bulk`, NoCache unless a
cited older corpus says otherwise.

## Status axes

- `execution_state`: `in_progress`
- `scope_completeness`: `scope_partial`
- `target_completeness`: `target_partial`
- `integration_completeness`: `partial`
- `open_lanes`: ERF/ERFC.PRECISE body (all numeric branches); GAUSS tiny-direct
  body; GAUSS ordinary wrapper residuals inherited from Q; NORMSDIST /
  NORM.S.DIST CDF inherited from the same body; CHIDIST(df=1) and
  GAMMA.DIST(shape=0.5) inherited after an identified argument transform;
  frozen GAUSS heldouts sealed until one coherent discovery survivor exists

This document is a theory pack, not an implementation. Production currently
still uses `libm::erf` / a fitted `libm::erfc` correction. Those are
scaffolding. Do not treat them as identified graphs.

---

## 0. Why this group, and why not another leftover

Several open W109 leftovers are 1–4 ULP near-identities (BINOM CDF vs
`BETA.DIST(1-p,n-k,k+1)`, F implied-z, odd-df `CHIDIST` vs `ERFC+EXP`). Those
are real, but they are *local*. The error-function group is the opposite: one
unidentified scalar body fans out into many published worksheet functions.

If the body is identified bit-exactly, the following surfaces move together:

| Surface | How it consumes the body |
|---|---|
| `ERF` / `ERF.PRECISE` | the P-side publication |
| `ERFC` / `ERFC.PRECISE` | the Q-side publication |
| `GAUSS` | sign-split wrapper around stored `z = \|x\|/√2` and Q, plus a tiny-x odd route |
| `NORMSDIST` / `NORM.S.DIST` CDF | empirically `0.5 + GAUSS` except where that subtract cancels |
| `NORM.DIST` CDF | `NORMSDIST((x-μ)/σ)` on 45/45 |
| `LOGNORM.DIST` CDF | `NORM.S.DIST((LN(x)-μ)/σ)` on 45/45; LN site already `excel_log` |
| `CHIDIST(x,1)` / `CHISQ.DIST.RT(x,1)` | `ERFC.PRECISE(SQRT(x/2))` on 154/154 |
| `CHISQ.DIST(x,1,TRUE)` / `GAMMA.DIST(x,0.5,2,TRUE)` | `ERF.PRECISE(SQRT(x/2))` on 154/154 |
| `GAMMA.DIST(x,0.5,β,TRUE)` | `ERF.PRECISE(SQRT(x/β))` for β in {1,2,4} |
| `CHISQ.TEST` df=1 | inherits `CHIDIST` after an identified statistic |
| odd-df `CHIDIST` | still GRATIO; not a worksheet ERFC+EXP recurrence, but df=1 is the a=½ special |

PHI is **not** in this body. PHI is a closed independent graph
(`RN53(RN64(x*x)) → excel_exp(−sq/2) → RN53(RN64(e * RN(1/√(2π))))` with
subnormal flush). `PHI(0)` bits `0x3fd9884533d43651` are the reciprocal
constant, not a divide. Do not re-open PHI. Do use `x * PHI(0)` as a *tiny
GAUSS* candidate — that is a different claim.

BINOM/F/T/PRICE/Poisson-PMF are out of this pack's implementation scope. They
may be cited as method analogies only.

---

## 1. Doctrine for every agent in this swarm

Clean-room is non-negotiable. Allowed:

1. public specifications and published algorithms (TOMS 654/708, NSWC/CDFLIB,
   Cephes, fdlibm, Cody SPECFUN, Boost, Hart, Ooura, SLATEC, ReactOS CRT
   sources, documented C API of era-appropriate runtime DLLs);
2. reproducible black-box Excel behavior through public automation
   (`Range.Value2`, not formula literals with long decimals);
3. the already-captured WitnessSets and GAUSS banks named below.

Forbidden:

1. disassembly, decompilation, dumping, or any inspection of Excel or
   Microsoft-shipped binaries;
2. proposing binary archaeology as the next step;
3. empirical correction polynomials as a *claimed graph* (the existing
   `excel_erfc = libm.erfc * (1+corr(s))` fit is scaffolding and has been
   explicitly retired as an identification candidate);
4. treating pass-rate gradient descent as identification;
5. opening a sealed heldout until one coherent discovery survivor exists;
6. reporting scaffolding as implemented, closed, done, or complete.

Comparator rule: bit-exact claims require Value2 plumbing. `Cells.Item.Value2
= 1+ε` has been observed to round back to 1.0; write ladders through a 2-D
`object[,]` into `Range.Value2`. Short exact literals `0, 0.5, 1, 2` in
formulas are safe.

Match Excel including Excel's imprecision. A row where a candidate is
correctly rounded and Excel is 1 ULP off is still a miss, tagged
`excel_imprecision_witness` only to record repair direction.

When two public functions agree bit-exactly, the *composition* is identified
even if the body is not. Do not throw away wrapper identifications because
the body still drifts.

---

## 2. Surface map and current OxFunc kernels

### 2.1 Worksheet names

Legacy and modern aliases that have been bit-identical on every bank cited
here, unless a row explicitly says otherwise:

- `ERF(x)` one-arg ≡ `ERF.PRECISE(x)`
- `ERFC(x)` ≡ `ERFC.PRECISE(x)`
- `NORMSDIST(x)` ≡ `NORM.S.DIST(x, TRUE)`
- `CHIDIST(x,1)` ≡ `CHISQ.DIST.RT(x,1)`
- `CHISQ.DIST(x,1,TRUE)` ≡ `GAMMA.DIST(x,0.5,2,TRUE)`

`ERF` also admits an interval form `ERF(lower, upper)` which OxFunc currently
implements as `erf(upper) - erf(lower)` via the same `libm` body. Interval
staging (two-arg minus vs a dedicated difference) is an open sub-lane and
should not be mixed into the one-arg body hunt until the one-arg graph is
known.

Logical operands: the ERF/ERFC family rejects `TRUE`/`FALSE` with `#VALUE!`
while still accepting numeric text. GAMMA/GAMMALN accept logicals. This is
admission, not the body.

Non-finite: kernels return `#NUM!` for non-finite x. Excel never publishes
`±Inf` for these (XMD-008); overflow/underflow becomes `#NUM!` or a flushed
`+0`.

### 2.2 What production actually computes today

`crates/oxfunc_core/src/functions/special_dist_family.rs`:

- `erf_precise_kernel` / one-arg `erf_kernel` → `libm::erf`
- `erfc_precise_kernel` → `excel_erfc`: `libm::erfc` for `x < 1.25` and all
  negatives; for `x ≥ 1.25` multiply by a piecewise Horner correction in
  `s = 1/x²`, split at `x = 2.857`, fitted to ~48 positive witnesses
  (~20/48 exact, ~28 blocked, worst 6 ULP)
- `erf_of_sqrt_half_x` / `erfc_of_sqrt_half_x` → the identified
  `SQRT(x/2)` wrappers used by chi/gamma df=1 and shape 0.5

`crates/oxfunc_core/src/functions/normal_dist_common.rs`:

- `gauss_kernel` → `0.5 * libm::erf(x / SQRT_2)` with a `x == 0` zero
- `norm_cdf` (in `normal_log_family.rs`) → `0.5 * (1 + erf_approx(x / SQRT_2))`
- `phi_kernel` → the closed PHI graph above

These production paths are **known not to be Excel's graph**. They exist so
the rest of the catalog can call a function. Improving the libm-correction
fit is not the assignment. Identifying the evaluation tree is.

### 2.3 Catalog rows

- G4-04 mixed combinatorial row: COMBIN/COMBINA signed off; **remains open
  only for ERF/ERFC.PRECISE**
- G3-07: PHI signed off 2026-07-11; GAUSS still open, currently documented
  as “0.5*erf(x/sqrt2) via libm erf” which the August 2026 GAUSS composition
  work has already refuted as the ordinary-magnitude wrapper
- G3-01: GRATIO family identified for incomplete-gamma; a=0.5 / df=1
  *dispatch* is the published ERF/ERFC, so G3-01's remaining df=1 residual
  **is this body**
- XMD-011: Excel's ERFC tail is *less* accurate than CR libm; chaotic
  ±(0.5–3)×2⁻⁵², worst ~6 ULP. Deviation direction established; reproduction
  partial

---

## 3. What is already proven (do not re-prove, do use)

Every identity below is bit-exact on the cited bank, live Excel, Value2,
build 20228 unless marked older.

### 3.1 Argument transforms (wrappers, not the body)

| Identity | Bank | Note |
|---|---:|---|
| `CHIDIST(x,1) = ERFC.PRECISE(SQRT(x/2))` | 154/154 | divide-by-two first |
| `CHIDIST(x,1) = ERFC.PRECISE(SQRT(x)/SQRT(2))` | 123/154 | **refuted** |
| `CHIDIST(x,1) = ERFC.PRECISE(SQRT(x)*(1/SQRT(2)))` | 117/154 | **refuted** |
| `GAMMA.DIST(x,0.5,2,TRUE) = ERF.PRECISE(SQRT(x/2))` | 154/154 | |
| `GAMMA.DIST(x,0.5,β,TRUE) = ERF.PRECISE(SQRT(x/β))` | 68/68 for β in {1,2,4} | |
| `CHISQ.DIST(x,1,TRUE) = GAMMA.DIST(x,0.5,2,TRUE)` | 154/154 | |
| `CHIDIST(x,1) = 1 - GAMMA.DIST(x,0.5,2,TRUE)` | 92/154 | cancellation, not the graph |
| `ERFC.PRECISE(z) = CHIDIST(2*z*z, 1)` | 27/27 and 160/160 on the k/32 ladder | implied Q *is* published ERFC |
| `GAMMA.DIST(k²/1024, ½, 1, TRUE) = ERF.PRECISE(k/32)` | 160/160 | older xv-gd capture |
| `CHIDIST(k²/512, 1) = ERFC.PRECISE(k/32)` | 160/160 | older xv-chi capture |
| `LOGNORM.DIST = NORM.S.DIST((LN(x)-μ)/σ)` | 45/45 | LN already `excel_log` |
| `NORM.DIST = NORM.S.DIST((x-μ)/σ)` | 45/45 | |
| `NORMSDIST = NORM.S.DIST(_, TRUE)` | 14/14 and 10/10 | alias |
| `NORMSDIST = 0.5 + GAUSS` | 19/22; 11/14 sparse | misses are cancellation tinies |
| `PHI = NORM.S.DIST pdf` | 14/14 | PHI body independent and closed |

Consequence: you may treat published `ERF.PRECISE` / `ERFC.PRECISE` as the
canonical P/Q oracles. Chi df=1 and gamma shape 0.5 are *the same oracle
with an identified `SQRT(x/β)`*. Decoding Q from `CHIDIST(2z²,1)` is a
valid implied-argument channel that does not require a second body.

### 3.2 GAUSS wrapper (August 2026 discovery, 8,192-row exact bank)

Source of truth: `ERF_GAUSS_EXACT_GRAPH_DISCOVERY_CHECKPOINT_20260809.md`
and the route-store increment. This supersedes the sparse 2026-08-20
`0.5*ERF(x*SQRT(0.5))` probe (13/22) as a *wrapper* claim.

1. GAUSS forms `z = abs(x) * FRAC_1_SQRT_2` and stores the binary64
   multiply. Native multiply and x87-multiply-then-store were identical on
   those inputs. Divide by `√2` and recomputed square-root forms are
   refuted. Witness: `GAUSS(1) = 0x3fd5d897a241a6fc`; the divide-staged
   alternative is `0x3fd5d897a241a6fa`.
2. For `abs(x) > 1e-15`, GAUSS publishes through a **sign-split complement**:
   negative `0.5*Q(z) - 0.5`, positive `(1 - 0.5*Q(z)) - 0.5`.
3. For `abs(x) <= 1e-15`, GAUSS uses a **direct odd small-result route**.
   The route-discovery ULP window pins the predicate as *inclusive*:
   binary64 `1e-15` = `0x3cd203af9ee75616` is direct; successor
   `0x3cd203af9ee75617` is sign-split. Inclusive wins 1,024/1,024;
   strict wins 1,022/1,024.
4. Existing ERF and ERFC maps reproduce GAUSS on every overlap for stored
   multiply plus sign-split: ERF 24/24, ERFC 24/24. Direct half-ERF is only
   21/24.
5. Direct-tiny route is globally odd: 1,355/1,355 nonzero signed pairs
   bit-exact; 54/54 subnormal-flush pairs publish canonical `+0`.

Diagnostic composite on the 8,192-row exact discovery (not a publication
graph): tiny direct 2,374/2,646 (max 2 ULP); branch-190 body 1,919/1,971
(max 8); public-libm erfc tail 3,181/3,557 (max 2); saturation 18/18.

### 3.3 ERF ↔ ERFC complement *direction* (700 paired discovery rows)

`ERF_GAUSS_ROUTE_STORE_OFFLINE_CHECKPOINT_20260809.md`, joining 1,720 ERF
and 870 ERFC nonnegative observations by exact input bits:

| Region | Primary relation | Exact | Reverse |
|---|---|---:|---:|
| `0 ≤ x < 0.5` | `Q = RN53(1 − stored P)` | 513/513 | reverse 196/513 |
| `x = 0.5` | both collapse | 1/1 | 1/1 |
| `0.5 < x < 1.375` | `P = RN53(1 − stored Q)` | 55/55 | reverse 16/55 |
| `1.375 ≤ x < 6` | `P = RN53(1 − stored Q)` | 130/130 | reverse 0/130 |
| `x ≥ 6` | `P = RN53(1 − stored Q)` | 1/1 | reverse 0/1 |

Direction-selected ordinary binary64 complement is exact **700/700**.
Compensated `0.5+(0.5−primary)` is only 649/700 (51 discriminators). The
`z < 0.5` residual therefore lives in the **P-side body before complement**,
not in an ERFC-below-half wrapper.

Independent older tooth-zone check: ERFC ≡ `RN53(1 − P-internal)` 48/48.

### 3.4 Branch boundary at 0.5, no Boost 5.8 cutoff, no tiny-z shortcut

- Small-branch / complement boundary at `0.5` (messy crossing scan).
- No Boost 5.8f `erf → 1` cutoff: Excel stays CR, not 1.0, just above 5.8f.
- No Boost `z < 1e-10` tiny shortcut: tiny-z rows are consistently CR+1.
- Small-branch value at 0⁺ is pinned:
  **R(0⁺) = `0x3ff20dd750429b6e` = CR(2/√π) + 1 ULP**.
- Apparent erfc hard-zero in (26.543, 26.544] is **not** Cody XBIG. The last
  finite sits in the smallest normal binade (`0x001…`): Excel-wide
  **subnormal publication flush**, same class as PHI.

### 3.5 Unsplit `exp(−RN53(z·z))` on the erfc side

Regression of messy-grid (full-mantissa z) residual against the exactly
computable `RN53(z·z)` rounding error: slope +0.95, residual stdev 7.9e-16
→ 2.0e-16 (±1 ULP floor). This **kills split-argument families**
(fdlibm/Cody) structurally. Method caution: dyadic-clean ladders (k/32)
have exact `z²` everywhere and make split-vs-unsplit invisible. Always
include a full-mantissa grid.

### 3.6 a = ½ GRATIO dispatch is the published ERF, not NSWC erfc1

Closed-halfint (erfc1-based) and `erf(a=1/2)` NSWC routes fail ±1..±24.
Excel uses **its own** erf/erfc at the a=½ special, which is the same
published `ERF.PRECISE` / `ERFC.PRECISE`. Identifying the body once closes
the dispatch.

For `a = ½` and `x = z² < 0.25`, TOMS 654 `gratio` always takes **branch
190** (direct), never complementary 200. Path 200's `1−q` staging is
catastrophic at tiny z in any precision. That is a structural reason the
small-z body looks like a series in `x = z²`, not like a complementary
rational.

### 3.7 PHI and the linear tiny GAUSS candidate

PHI is closed and odd-independent of ERF. On a dyadic GAUSS ladder
(2026-08-21):

- `GAUSS(2^{-k}) = 2^{-k} * PHI(0)` bit-exactly for k = 50..60
  (mantissa of PHI(0) transported, exponent subtracted)
- at `2^{-49}` GAUSS publishes quantized `0x3cc8000000000000`, **not**
  `2^{-49}*PHI(0) = 0x3cc9884533d43651`
- from `2^{-48}` through `2^4` on that ladder, `GAUSS = NORMSDIST − 0.5`
- `NORMSDIST − 0.5` underflows to `+0` in the deep tiny band while GAUSS
  stays nonzero — that is why GAUSS exists as a worksheet function

Treat `x * PHI(0)` as the leading *deep-tiny* candidate, and
`NORMSDIST(x)−0.5` / the August sign-split Q wrapper as the *ordinary*
candidate. The seam at `1e-15` (August route bank) and the NORMSDIST
cancellation seam near `2^{-49}` are **different thresholds** and must not
be collapsed.

---

## 4. Working theory of the evaluation tree

This is the theory agents should attack, not a claim.

### 4.1 One compiled special, several publications

Excel's 1990s statistical DLL lineage (NSWC/CDFLIB, TOMS 654 GRATIO, TOMS
708 BRATIO) is already the identified family for incomplete gamma and
incomplete beta. The a=½ / erf special is **the same era and the same
source tree**, but the compiled erf publication is not a faithful public
NSWC `erfc1` and not a faithful public `gratio` complementary return.

Theory T1 (leading):

```
z = |argument|                    # ERF/ERFC: the input; GAUSS: |x| * 1/√2 stored
if z == 0: publish 0 for erf/gauss, 1 for erfc
if z is huge: saturate erf→±1, erfc→0, gauss→±0.5; then subnormal-flush
if z < 0.5:
    x = z*z                       # possibly stored
    P = branch_190(a=1/2, x)      # series in x, times g=1+gam1(1/2), times exp(0.5*ln x)
    Q = RN53(1 - P)
else:
    Q = erfc_tail(z)              # unsplit exp(-RN53(z*z)) * rational(1/z²) or equivalent
    P = RN53(1 - Q)
publish according to the surface (ERF=P, ERFC=Q, GAUSS=sign-split of Q)
```

For GAUSS specifically, an extra predicate `abs(x) <= 1e-15` skips the
complement wrapper and runs an odd direct body that is *related to* the
z→0 limit of branch 190 but is not exactly `x * PHI(0)` except on a subset
of dyadic tinies.

### 4.2 Branch 190 as written in the public TOMS 654 / CDFLIB source

`check_erf190.rs` encodes the published recurrence the races actually run.
Agents should start from this, not from a re-derived series:

```
a  = 1/2
x  = z*z
an = 3; c = x; sum = x/(a+3)
loop { an += 1; c = -c*(x/an); t = c/(a+an); sum += t }   # until t underflows
j  = a*x*((sum/6 - 0.5/(a+2))*x + 1/(a+1))
zl = a * ln(x)                     # 0.5 * ln(z²)
h  = gam1(a)                       # NSWC rational for 1/Γ(1+a) − 1
g  = 1 + h
w  = exp(zl)
ans = w * g * (0.5 + (0.5 - j))    # one of several associations
```

Public `gam1` coefficients (NSWC, also in `check_erf190.rs`):

```
GP = [0.577215664901533, -0.409078193005776, -0.230975380857675,
      0.0597275330452234, 0.00766968181649490, -0.00514889771323592,
      0.000589597428611429]
GQ = [1.0, 0.427569613095214, 0.158451672430138,
      0.0261132021441447, 0.00423244297896961]
```

Empirically, `h` at a=½ wants something near `0x3fc06eba8214db6c`
(fdlibm's `efx` is `…db69`, 3 ULP away). A full 8,192 mixed-spill
enumeration of published `gam1(1/2)` produced 17 distinct Ext80 values;
**none** hits the inferred mantissa `0x906eba8214db6c6f`. Nearest public
graph is 57 Ext80 mantissa units away. Substituting every public value
does not lift the 850/1,508 plateau.

### 4.3 Competing theories for the last mile

Agents should keep these as named rivals, not blend them.

**T1a. Faithful branch 190 + unknown spill/association.** Plateau 850/1,508
on `0<z<0.5` (max 3 ULP) across 18,432 source-backed graphs; 1024 spill
configs of the 190-path with true x87 chains plateau 850/1,508. A 13-site
recurrence spill-mask is flat. This is the “we have the right recurrence
and the wrong store schedule” theory. It has already been densely raced.

**T1b. Branch 190 with a fused `(1−j)` or rearranged j-cluster.** The
tooth law (below) looks like a z²-magnitude term beating a ~2⁻⁶⁴ quantum.
The public `j` formula has several algebraically equal associations.
b10 designed max-residue battery: `z̃ = z_direct × g` wins 37/50 over
sqrt / explog-of-`RN53(z²)` / reflection. ~92% of those rows within ±1.
The remaining sawtooth is outside the 13 enumerated axes.

**T1c. Double-precision LN residual, tight EXP.** b18 (242,474 ERF.PRECISE
rows) killed the “fine comb / period table” as scan-grid aliasing. The
real fingerprint is a **phase gradient**: miss probability rises toward
the rounding boundary (~10% at phase 0 to ~48% at |phase|=0.5 in e=−25/
−30/−40; flatter ~36%→52% at e=−15/−20). Misses at phase 0 exist, so the
driver reaches ≥0.5–1 ULP. Spatially incoherent, equidistributed: the
signature of a last-bit transcendental, not a coherent staging. Chopped
exp is **refuted** on this path (both +1 and −1 flips). Tension: POISSON
k=0 exp is ~99.4% CR, inconsistent with a ±0.45 ULP exp residual here.
Unification hypothesis: the residual lives in **internal LN** (~±0.5–0.7
ULP), and EXP is one tight near-CR routine (chop-published only at the
gamma series r-site). Battery b25 (163,840 rows) was designed to split
exp-relative vs ln-absolute at the `m ≈ 1.7724` binade crossing of `ans`
(`S_exp` halves, `S_log` unchanged). That result is the next thing to
read from `answers-b25-erfx.json` if the files are still in
`smart-fuzzer/work/w109/G3-01-dist/`.

**T1d. Custom Microsoft rational, no public tables.** Dissolved as a
*coefficient* hypothesis: Boost int_<64>, Cephes ndtr, Ooura, Hart 5666,
SLATEC, renormalized Cody/Cephes, Padé [m/n] m,n≤8, Taylor truncations,
two-product sums, constant neighborhoods — all ruled out. “There are no
coefficients” means *no unidentified rational table that fits as a
drop-in*. It does not mean the graph is coefficient-free; `gam1` is a
rational, the tail is a rational, the series has terms. It means hunting
a new Hart-style polynomial by least squares is the failed 2024–2025
method (`ERFC_EXCEL_EMULATION.md`).

**T1e. GAUSS tiny-direct is `x * c` with c near `1/√(2π)`.** Rounded
binary64 `1/√(2π)` times `x` scores 2,397/3,158 (max 2 ULP) on the
combined direct-route corpus. Best zero-limit branch-190 association:
2,632/3,158. After carrying stored `z = RN53(|x|*FRAC_1_SQRT_2)` into the
full public 190 body including `J`: **2,822/3,158, max 1 ULP, sum 336,
480 tied graphs**. Histogram of expected−model: {−1: 170, 0: 2822, +1:
166}. A ±4,096 Ext80-mantissa normalizer scan only reaches 2,638/3,158
in the earlier race and does not close the later 336-ULP-sum residual.
Constant correction is refuted. The 14 later tie-break inputs
(`ERF_GAUSS_DIRECT_TINY_TIE_MINING_OFFLINE_CHECKPOINT_20260809.md`) split
the 480 ties into classes of 80 and 400; they have not been answered by
Excel in that checkpoint.

**T1f. Tail `x ≥ 0.5` is public libm erfc plus unsplit `z²`.** Public
libm erfc on decoded GAUSS Q: 773/784 exact. Branch-190-plus-publication
graphs: 778/784, max 1 ULP, six residuals all “candidate Q plus one ULP”,
five of them immediately above `z = 1/8`, one at transported `√2/8`.
That is the tooth fingerprint *isolated before GAUSS publication*. Tail
on the 8,192 GAUSS composite: 3,181/3,557, max 2 ULP. Production's
libm×corr(s) fit is a numerically similar *wrong explanation* of this
±1–2 ULP floor.

### 4.4 Tooth law any T1a/T1b mechanism must reproduce

From oracle-driven bisection (`tooth_positions.json`, z ≈ 2⁻³⁰ binade):

1. Teeth are steps in Excel-vs-true-chain-model ε, −0.85 ULP each, ramp
   +0.145 ULP per 1/128-mantissa step between teeth.
2. Linear in z, constant period within a binade, near-zero-anchored.
   Two teeth at ~2⁻⁶⁶ relative:
   t1 = 1.13327213842e-9, t2 = 1.52563759794e-9 = t1 + exactly 9 periods,
   **p(e=−30) = 4.35961621689e-11 = 2.996·2⁻³⁶, NOT 3·2⁻³⁶**;
   t/p ≡ 0.995 (mod 1).
3. Half-quantum grid slip at mantissa 1.5 exactly; spacing collapse ≈3×
   after V crosses its binade (m = 2/g ≈ 1.7724).
4. Nested sawtooths: at e=−40 both a coarse (~6e-14) and a fine
   (~5.95e-17) structure. Periods are not a smooth function of exponent.
5. Amplitude: `ε_amp · |zl| ≈ 25.7 ULP` constant across binades ⇔ a
   constant-absolute perturbation of `zl²/2`.
6. Generator is P-side / pre-complement.

Ruled out as tooth generators: log-spaced (exp/ln tables, F2XM1
boundaries), x-quantized (1/m spacing contradicts exact arithmetic
progression), decimal round-trips, w-quantization.

**Method caution from b18:** a period estimated from a dense scan is only
real if it reproduces at a 10× finer grid. Earlier “median gap” tables
were the grid echoing itself.

### 4.5 What “extended vs double” is allowed to mean

Different call sites of the *same* compiled statistical function already
spill differently in this campaign:

- gamma a≥1 path 20: **proved** double-rounded log (`sp_both`)
- erf a<1 path 190: all-double staging gives ±8 ULP tiny-z wobble
  (ruled out); extended log/exp keeps `exp(0.5·ln x)` sub-ULP as observed
- gamma series r-site: **chopped** (RZ53) publication of exp
- erf path: chop **refuted**; both-sided flips
- gam1 arithmetic on GAUSS tiny: binary64 per operation **2,822/3,158**
  beats Ext80 continuous **2,324/3,158** and Ext80-with-f64-returned-h
  **1,706/3,158**

So “x87 extended” is not a global flag. It is per-op, per-site. State
which site you are spilling.

---

## 5. Ruled-out catalog (do not revive without a new discriminator)

Publish a new discriminator first if you believe a “ruled out” graph was
only raced on a degenerate grid.

### 5.1 Public erf/erfc implementations

| Candidate | Score / why dead |
|---|---|
| NSWC / CDFLIB `erfc1` | 83/1,508 max 7 ULP on small body; 113/176 erf |
| Cody SPECFUN CALERF | 121/176 erf, 56/176 erfc, both exp models |
| fdlibm `s_erf` / small rational | 160/176 closest of the classics, still ±2; 558/1,508 on small body |
| Boost 1.35–1.42 `erf_imp<53>` | 155–157/176; tables identical across those versions |
| Boost int_<64> tables | ruled out in the coefficient-recovery sweep |
| Microsoft UCRT erf (documented C API) | 146/176 |
| Cephes `ndtr` / erf | coefficient sweep |
| Ooura `gamerf`/`derf` | coefficient sweep |
| Hart 5666 | coefficient sweep |
| SLATEC | coefficient sweep |
| host `libm::erf` / `libm::erfc` | production scaffolding; 9/48 ERFC positives; GAUSS ordinary wrapper refuted as direct half-erf |
| literal GRATIO complementary-return for z<0.5 | 577/1,508, worse than direct 190 |
| distribution-site raw `excel_pow_chain` at this call site | 482/1,508 |
| split-argument `exp(-z²)` (fdlibm/Cody style) | unsplit `exp(-RN53(z·z))` regression |
| Boost z<1e-10 form | tiny-z consistently CR+1, shared R(0⁺) |
| Boost 5.8f saturate-to-1 | Excel stays CR |
| Cody XBIG hard-zero ~26.54 | subnormal flush, last finite in `0x001…` |
| compensated complement `0.5+(0.5−P)` as ERFC wrapper | 649/700 vs 700/700 for `RN53(1−P)` |
| GAUSS as `0.5*erf(x/√2)` at ordinary magnitudes | 21/24 vs 24/24 for Q sign-split; 7,428→7,492/8,192 once wrapper is right |
| GAUSS via `/√2` or recomputed extended sqrt | GAUSS(1) bits discriminate |
| single adjusted `gam1` / normalizer constant | ±2048 and ±4096 Ext80 scans flat at the six-Q and tiny residuals |
| empirical `libm*(1+corr(s))` tables | retired as identification; chaotic 1-ULP residual |

### 5.2 CRT exp as the erf last op

Real-binary CRT sweep (documented C API, 32-bit harness, SxS manifest for
msvcr90 9.0.30729, plus msvcr100/110/120/msvcrt, `_set_SSE2_enable(0)`,
AMD K8/x64 table exps from ReactOS/Open64): **all refuted** for Excel's
internal exp generally. Excel exp ∈ {CR, CR−1}, one-sided low. That chop
was localized to the **gamma series r-site**, not erf. On erf, both-sided
misses kill chop. Do not re-sweep those DLLs for erf last-op unless you
have a new call-site theory.

### 5.3 What the 2026-08-20/21 inverse-identity wave adds (and does not)

Those probes were sparse and wrapper-oriented. They are useful as
*independent replications* of compositions, not as body identifications.

- `GAUSS = 0.5*ERF(x*SQRT(0.5))` 13/22 (helper cell same 13/22): consistent
  with “ordinary GAUSS is not direct half-erf” from the 8,192-row bank.
- `GAUSS = 0.5 − 0.5*CHIDIST(x*x,1)` 10/22: CHIDIST uses `SQRT(x/2)` not
  `x*x` into ERFC; this is a staging miss, not a body miss.
- `GAUSS = NORMSDIST−0.5` 19/22: compatible with GAUSS and NORMSDIST
  sharing Q after different wrappers; the three huge misses are tinies
  where `NORMSDIST−0.5` underflows.
- `GAUSS = x*PHI(0)` 2/22 overall, but bit-exact on dyadic ≤ 2⁻⁵⁰: the
  deep-tiny linear candidate, not a global identity.
- Odd-df `CHIDIST(x,3) = ERFC(s)+2s*EXP(−z)/√π` with helpers: 1–4/10,
  leftover 1–5 ULP. Do not spend this swarm on odd-df recurrence; that is
  GRATIO, not this body.

---

## 6. Frozen corpora (use these; do not silently resample)

GAUSS banks (build 20228 / x64 / CV2 / Value2 / NoCache), hashes from the
2026-08-09 checkpoint:

| Role | Rows | Batch SHA256 | Answers |
|---|---:|---|---|
| exact discovery v1 | 8,192 | `8627F7E248545CB618684EFA24D76336BBE9C6A545B7BCFE2CE2D9CE3F3395A3` | `8BFFAF353EFFDB54F15B82CCA4997E35761E4F65A51A0991B169C1CA75AFBCA8` |
| exact heldout v1 | 4,096 | `D10E8B813BAABD6F7718ED78E6008FDC2D75CC0C2B272AEEC25487949F2E21D4` | **sealed, absent** |
| route discovery v1 | 1,024 | `28F0BEBFBF5354A5624DAC7B0C6A27EF01E74ADD10E85DF513C0DC51E6EE4F93` | `2D225BDB490FC8B6EF980B68B5993ACE4E69F97262D60885F4C7CBDF9E1FD1B1` |
| route heldout v1 | 512 | `E6F737337C3F1661A48E362D9333D4E0B09DF564F6CFEFB1C62BCD80B68DAFF0` | **sealed, absent** |

Historical ERF/ERFC WitnessSets used in the complement audit (700/700
directional) **do not embed** build/CV/plumbing provenance. Strong
black-box evidence for those pairs; not a 20228/CV2 sign-off. Fresh
provenance-rich ERF.PRECISE / ERFC.PRECISE replay is an open axis.

Older but still cited:

- `answers-erfp/erfcp.json` — clean k/32 ladders
- `answers-erfm/erfcm.json` — full-mantissa
- `answers-b7erf/b7erfc.json` — lineage fingerprints
- `answers-b8erf/b8erfc.json` — flush bisect + 0.5 crossing
- `answers-b9train.json` (1190), **`answers-b9heldout.json` (256) NEVER
  RACED — promotion gate, do not score it as a working set**
- `answers-b10.json` (50 max-residue), `answers-b11.json` (256 ladder),
  `answers-b11c.json` (511 erfc complement view)
- `answers-b18.json` (242,474 ERF.PRECISE)
- `answers-b25-erfx.json` (163,840; ln vs exp disambiguation)
- `agentJ_constraints.jsonl` (79,510 miss-row signed residuals)
- `dump-m30.txt`, `tooth_positions.json`
- G3-01 work dir (gitignored): `smart-fuzzer/work/w109/G3-01-dist/`
- inverse-identity scripts (gitignored):
  `smart-fuzzer/work/w109/inverse-decomp/Run-LateralSweep.ps1`,
  `Run-GaussSeam.ps1`, `Run-CrossFamilyIdentities.ps1`

Racers (tracked):

- `check_erf190.rs` — Ext80 branch-190, 512 spill configs, dump mode
- `race_erf_precise_pow_substrate.rs`, `race_erf_precise_public_small.rs`
- `race_gauss_composition.rs`, `generate_gauss_exact_banks.rs`
- `audit_erf_gauss_route_store.rs`
- `mine_erf_gauss_direct_tiny_ties.rs`

Stable bits to keep in any replay:

- `GAUSS(1) = 0x3fd5d897a241a6fc`
- `PHI(0) = 0x3fd9884533d43651`
- `R(0⁺) = 0x3ff20dd750429b6e`
- GAUSS inclusive tiny threshold `0x3cd203af9ee75616` vs successor
  `0x3cd203af9ee75617`
- six decoded-Q residuals listed in §7 of the exact-graph checkpoint
  (`0x3fc0000000000001` … `0x3fc6a09e667f3bcd`)

---

## 7. Current production residual (what “better libm” looks like)

On the 154-row nonnegative CHIDIST(df=1) bank, before the
`ERFC.PRECISE(SQRT(x/2))` *dispatch*, OxFunc matched 63/154 (max 30 ULP).
After dispatch through the still-wrong ERFC body: 88/154 (max 31 ULP).
The dispatch is the identified graph; remaining misses **are the body**.

ERFC correction-fit vs a 48-point positive witness set: 20 exact, 28
blocked. Blocked x (do not overfit a new polynomial to these):

```
1.6, 1.7, 1.75, 1.9, 2.0, 2.05, 2.1, 2.35, 2.45, 2.55, 2.6, 2.65, 2.7,
2.85, 2.9, 2.95, 2.999, 3.005, 3.01, 3.02, 3.5, 3.75, 4.5, 5, 6, 7, 9, 10
```

Matched anchors that a body change must not regress:

```
0, 0.5, 1, 1.25, 1.5, 1.8, 1.85, 1.95, 2.15, 2.25, 2.4, 2.5, 2.75, 2.8,
2.99, 3, 3.001, 3.25, 4, 8
```

and the usual negatives. Near x=3 a 0.001 grid already flips
Matched/Blocked non-monotonically (`2.999 B / 3.000 M / 3.001 M / 3.005 B`).
That chaos is why least-squares corr(s) died.

CHIDIST(1,1) Excel pin `0x3fd44ed0bb7cb209` vs current ERFC body: 1 ULP.
Do not treat a single 1-ULP pin as the whole body.

---

## 8. Timeline of attempts (compressed)

- **Pre-W109 / ERFC_EXCEL_EMULATION.md:** libm; then UCRT FFI at x≥3
  (Windows-only, 12 matches); then cross-platform libm×corr(s) (20/48).
  Explicitly not an identified graph.
- **2026-07-11:** PHI identified and signed off. GAUSS left on libm erf.
- **2026-07-16 G3-01:** GRATIO family for incomplete gamma. Multi-view
  collapse. a=1 exponential dispatch. a=½ called out as “Excel's own erf”.
- **2026-07-17 erf sub-lane:** wiring via GAMMA.DIST / CHIDIST ladders
  160/160. NSWC/Cody/fdlibm/Boost/UCRT ruled out. Unsplit z². No tiny
  shortcut. R(0⁺) pin. Coefficient hunt **dissolved**. Branch 190 named.
  `check_erf190` 663/1218 on z<0.5. Tooth bisection. b18 kills period
  tables, installs phase-gradient. Chop localized to gamma series, not erf.
- **2026-07-18 resume:** erf 190-path parked at C10r 67.65%; b9heldout
  untouched. Next probes: 2^Ez-grid source, j-pipeline park phases,
  parked-vs-register-continuous chain floor.
- **2026-08-09:** GAUSS composition, complement direction 700/700, tiny
  predicate 1e-15 inclusive, branch-190/store 2,822/3,158 with 480 ties,
  six decoded-Q landmarks. Heldouts frozen. Empirical corr tables
  forbidden as candidates.
- **2026-08-09 follow-on:** 14 answer-blind tiny-tie separators (7 sign
  pairs); 480 ties → 80+400 classes. Not yet oracle-answered in that
  checkpoint.
- **2026-08-18..21 inverse-identity wave:** CHIDIST(df=1) and
  GAMMA.DIST(0.5,*) wrappers landed in production as
  `erfc_of_sqrt_half_x` / `erf_of_sqrt_half_x`. Sparse GAUSS/NORMSDIST/
  PHI probes. ASINH/ACOSH/FISHER closed as *other* LN graphs — method
  analogies only (SIGN/ABS, divide-first, 1-minus is not the graph).

---

## 9. Questions the swarm should answer

Phrase answers as *named graphs* plus *the discriminator that would kill
them*. Preference order: a graph that is exact on an existing discovery
bank beats a graph that is merely closer.

### 9.1 Body, z < 0.5 (P-primary)

1. What exact association of `j`, `w`, `g`, and `(0.5+(0.5−j))` plus which
   per-op PC64/PC53 stores reproduces the tooth period
   `4.35961621689e-11` at e=−30 and the m=1.5 half-quantum slip?
2. Is `exp(0.5 * ln(z²))` vs `exp(ln z)` vs `sqrt(z²)` vs `|z|` the w
   recovery, and is ln the identified `excel_log` / FYL2X or a private
   double-error ln? b25's binade-crossing of `ans` at m≈1.7724 is the
   designed split.
3. Why does no public `gam1(1/2)` spill hit mantissa `0x906eba8214db6c6f`,
   and is that mantissa even the right object (it was inferred, not
   measured as a published intermediate)?
4. Can the six decoded-Q landmarks (five just above z=1/8, one at √2/8)
   be produced by a single extra rounding of an otherwise exact 190-path
   Q, or do they require a different recurrence truncation?
5. Does the series stop-rule match NSWC (`|t| < eps * |sum|`) or a fixed
   term count or a fused `(1−j)`?

### 9.2 Body, z ≥ 0.5 (Q-primary)

1. What rational in `s=1/z²` (or continued fraction, or continued
   exponential) with unsplit `exp(−RN53(z·z))` is exact, not 773/784?
2. Where is the 0.5 / 1.375 / ~6 region structure coming from if not
   fdlibm's 0.84375 / 1.25 / 2.857 / 6.0 and not Boost's 1.5 / 2.5 / 4.5?
   Detrended residual was a flat ±1 ULP floor across [0.5, 6] — maybe
   there is **no** extra breakpoint, only complement direction at 0.5
   and saturation/flush at large z.
3. Saturation and subnormal flush: is erfc(z) flushed with the same
   `|v| < DBL_MIN → +0` rule as PHI, or a different cutoff?

### 9.3 GAUSS tiny-direct, abs(x) ≤ 1e-15

1. Is the body the z→0 limit of branch 190 evaluated at
   `z = RN53(|x|*FRAC_1_SQRT_2)`, which already reached 2,822/3,158?
2. What kills the remaining 336 ULP-sum / 170+166 ones? The 14
   tie-separators in the offline checkpoint are the cheapest next
   Excel questions (Value2, inclusive 1e-15 bank, both signs).
3. How does `x * PHI(0)` (exact on dyadic 2⁻⁵⁰…2⁻⁶⁰) sit inside that
   190-limit? PHI(0) is `RN(1/√(2π))`. Branch 190's `g * w / z` limit
   should be `2/√π * z / 2 = z * 2/√π * ½` wait: erf'(0) = 2/√π,
   GAUSS'(0) = 1/√(2π) = PHI(0). Any correct odd body must have that
   derivative. `x * PHI(0)` is the first-order truncation; branch 190
   keeps `j ~ O(z²)`. The quantized `2^{-49}` value
   `0x3cc8000000000000` is a discriminator between linear and series.
4. Subnormal flush to canonical `+0` on 54/54 pairs: same as PHI's
   `|v| < DBL_MIN` or a wider band?

### 9.4 NORMSDIST vs GAUSS

1. Is `NORMSDIST(x)` identically `0.5 + GAUSS(x)` with a fused add that
   still cancels, or a separate `0.5 * (1 + erf(x/√2))` that uses P
   instead of GAUSS's Q sign-split?
2. If they share Q, NORMSDIST's tiny failure is just the add. Then
   identifying GAUSS+ERF closes NORMSDIST for free except at tinies,
   and NORMSDIST tinies may be allowed to be 0.5 exactly.
3. `NORM.S.INV` / `NORMSINV` invert a forward surface. Do not hunt the
   inverse until the forward is identified (same lesson as CHIINV:
   invert the published surface, not 1−P).

### 9.5 Two-arg ERF and negatives

1. `ERF(a,b)`: `erf(b)−erf(a)` with two independent publications, or a
   combined interval that shares cancellation with `erf` differences?
2. Negatives: ERFC matches libm on tested negatives in the old 48-point
   set. GAUSS is odd on the tiny bank. ERF is odd. Confirm whether
   negative ordinary ERF is `−P(|x|)` or `P(x)` through a signed 190-path.

---

## 10. Discriminating experiments (preferred order)

Design each experiment so that *all currently tied graphs disagree on at
least one row*, then ask Excel once.

1. **Answer the 14 tiny-tie separators.** They already split 480 tied
   190-path graphs into 80 vs 400. Seven inputs plus sign mirrors,
   bits listed in
   `ERF_GAUSS_DIRECT_TINY_TIE_MINING_OFFLINE_CHECKPOINT_20260809.md`.
   Inclusive `abs(x) ≤ 1e-15`. This is the highest-leverage unasked
   question in the file.

2. **z = 1/8 neighborhood at high mantissa resolution.** Five of six
   decoded-Q misses sit on `0x3fc000000000000k`. Walk `z` through
   `0x3fc0000000000000` ± 64 ULP, capture ERF, ERFC, GAUSS(z*√2),
   `CHIDIST(2z²,1)`. If the tooth is a stored `z²` beating 1/8, the
   four surfaces will move together.

3. **b25 crossing read.** If `answers-b25-erfx.json` is intact, score
   miss-width in published ULPs across `ans` binade at m≈1.7724.
   Halving ⇒ exp-relative residual; not halving ⇒ ln-absolute.
   Do not recapture 163k rows until that file is shown missing.

4. **Inclusive 1e-15 vs 2⁻⁴⁹ vs NORMSDIST.** One ladder of Value2
   powers of two and of `1e-15 ± 2 ULP`, capturing GAUSS, NORMSDIST,
   `0.5+GAUSS`, `x*PHI(0)`, `ERF.PRECISE(x*FRAC_1_SQRT_2)` with the
   multiply in a helper cell. Keep the August predicate and the
   NORMSDIST-cancellation predicate as separate columns.

5. **Complement direction at 0.5 ± 1 ULP** for ERF and ERFC on a
   provenance-rich 20228 capture, to confirm the 700-row historical
   join still holds.

6. **w-recovery metamers at tiny z:** helper cells for `SQRT(z*z)`,
   `EXP(0.5*LN(z*z))`, `EXP(LN(z))`, `ABS(z)` versus ERF(z). Only
   meaningful on full-mantissa z (not dyadic).

7. **Do not:** fit a new corr(s); re-race the 18,432 190-path graphs
   without a new axis; open b9heldout or GAUSS heldouts; disassemble
   anything; spend the swarm on odd-df chi or BINOM.

Promotion rule (already standing): one coherent exact survivor on
discovery, then and only then unseal the matching heldout. b9heldout
(256) and GAUSS exact heldout (4,096) / route heldout (512) stay sealed.

---

## 11. Analogies from closed graphs (steal structure, not coefficients)

These are method transfers that already paid off elsewhere.

- **Divide-first vs reciprocal.** `SQRT(x/2)` not `SQRT(x)/SQRT(2)`;
  `GAMMA.DIST(x,1,β)=EXPON.DIST(x/β,1)` not `EXPON.DIST(x,1/β)`. GAUSS
  already used this: multiply by `FRAC_1_SQRT_2`, not divide by `√2`.
- **1-minus is usually not the graph.** Chi CDF is not `1−CHIDIST`;
  ERFC below 0.5 *is* `RN53(1−P)` (exception, proven 513/513); above
  0.5 the primary is Q and ERF is `RN53(1−Q)`. Always race both
  directions.
- **Implied inverse.** `CHIDIST(2z²,1)` decodes published ERFC;
  GAUSS(x) and GAUSS(−x) uniquely invert to Q without ERF answers
  (822 unique pairs → 784 distinct z,Q).
- **Tiny-x other body.** ATANH cubic below `0x3f1af82b729c1d83`;
  ACOSH LN form with `x*x` overflow `#NUM!`; GAUSS direct-odd below
  1e-15 inclusive. Always look for a floor *and* an overflow/
  flush.
- **Sibling overflow.** ASINH and ACOSH `#NUM!` when `x*x` overflows,
  not when the analytic result overflows. ERFC's “zero” at ~26.5 is
  flush, not overflow.
- **Site-dependent exp publication.** Same F2XM1 chain, chop only at
  gamma series r, nearest at wrappers, extended delivery hypothesized
  on erf. Do not assume one rounding mode for excel_exp everywhere.
- **Do not land 1-ULP near-identities.** BINOM vs BETA 135/150 with
  implied-x mostly `nextafter(1-p)` still was not BETA.DIST. GAUSS vs
  half-erf 21/24 is the same class of trap.

---

## 12. Anti-patterns specific to this pack

1. **Aliased periods.** If a “comb” rescales when you refine the grid
   10×, it is the grid. b18 is the exhibit.
2. **Dyadic-clean ladders.** k/32 makes `z²` exact and hides unsplit vs
   split `z²`. Use full-mantissa z for staging.
3. **Global x87 flag.** Per-op stores. gam1 binary64-per-op beat
   Ext80-continuous on tiny GAUSS by 500 rows.
4. **Correction polynomials.** They match islands and fail chaotically
   next door (x=3 neighborhood). Forbidden as a claimed graph.
5. **Opening heldouts to shop for a winner.** Discovery first.
6. **Re-proving PHI.** Closed. Use it as a constant source.
7. **Collapsing 1e-15 and 2⁻⁴⁹.** Different predicates, different
   partners (Q sign-split vs NORMSDIST add vs linear PHI(0)).
8. **Reporting libm-plus-delta as identification.**
9. **Binary archaeology** when the wall “looks like it needs a dump”.
   The answer is a better probe (implied-Q, landmark z=1/8, tiny ties).

---

## 13. Success criterion for a swarm proposal

A proposal is useful if it contains:

1. a concrete evaluation tree (ops, constants as bit patterns, store
   sites, predicates with inclusive/exclusive bits);
2. which existing bank it should already match, and why it is not one
   of the ruled-out rows in §5;
3. one new discriminator (input bits + which public formula) that
   separates it from the current tie class (480 tiny graphs, or
   850/1,508 small-body graphs, or 778/784 Q graphs);
4. an explicit statement of what would refute it in one Excel capture.

A proposal is not useful if it says “try more coefficient recovery”,
“maybe it's x87”, “fit ERFC again”, or “disassemble erf.pro”.

---

## 14. Worked mechanics the swarm will need

### 14.1 GAUSS ordinary wrapper, algebraically

Let `c = FRAC_1_SQRT_2` (the stored multiply, not `1/√2` computed as a
divide). Let `z = RN53(|x| * c)`. Let `Q(z)` be the internal complementary
error-function-scale quantity that ERFC publishes for the same `z` (the
decoded-Q work treats it as erfc-scale: `Q(0)=1`, `Q(∞)=0`).

August 2026 sign-split, `abs(x) > 1e-15`:

- `x ≥ 0`: `GAUSS = (1 − 0.5*Q(z)) − 0.5 = 0.5 − 0.5*Q(z)`
- `x < 0`: `GAUSS = 0.5*Q(z) − 0.5`

If `Q = erfc(z)` exactly, then `0.5 − 0.5*erfc(z) = 0.5*erf(z)`, so
positive GAUSS would be half-erf of the *stored* `z`, not half-erf of
`x/√2` with a divide. The 3/24 misses of direct half-erf vs 24/24 of the
Q wrapper are exactly the difference between “call ERF and multiply by
0.5” and “call the Q publication and do the stored complement”. Those
are different last ops even when they agree on most rows.

Check: `GAUSS(1) = 0x3fd5d897a241a6fc`. Any candidate that cannot hit
those bits with `x=1` Value2 is dead, including divide-staged `z`.

### 14.2 Implied Q from a GAUSS pair

For `x > 1e-15` and `y = −x` both captured:

```
Gp = GAUSS(x)     # 0.5 − 0.5 Q
Gn = GAUSS(−x)    # 0.5 Q − 0.5
```

Then `Gp − Gn = 1 − Q`, so `Q = 1 − (Gp − Gn)`, and also
`Q = 1 − 2*Gp` from the positive side alone. The pair is used because
the two publications must agree on one Q; 822 of 888 pairs in
`EPS < x < 0.7` uniquely decoded. The six residuals after 778/784
public-190 graphs are listed as *Q plus one ULP* — meaning the wrapper
is believed exact and the body is 1 ULP high at those z.

You can also decode Q without GAUSS: `Q = ERFC.PRECISE(z)` by definition
of the P/Q theory, or `Q = CHIDIST(2*z*z, 1)` (27/27 and 160/160). If
those three Q's ever disagree, the “one body” thesis is in trouble. They
have not disagreed on any cited overlap.

### 14.3 Branch 190 at a = ½, first-order sanity

`x = z²`. `w = exp(0.5 ln x) = |z|` in exact arithmetic. `g = 1+gam1(½)`
is a constant. `j = O(x) = O(z²)`. The inner `0.5+(0.5−j) = 1−j`.
So `ans ≈ |z| * g * (1 − O(z²))`. For erf, the small-z series is
`(2/√π) z (1 − z²/3 + …)`. Matching derivatives at 0 forces
`g = 2/√π` in exact arithmetic. Measured R(0⁺) is CR(2/√π)+1 ULP, which
is why `g` inferred from Excel is not the correctly-rounded 2/√π and
not fdlibm `efx`. A graph whose `g` is CR(2/√π) is already wrong at 0⁺.

`gam1(a)` in NSWC is a Padé-like rational for `Γ(a+1)⁻¹ − 1` near a=0.
At a=½ it is simply a constant. Evaluating that rational in binary64
vs Ext80 vs mixed stores is the 8,192-config race that never hit the
inferred 80-bit mantissa. Either the inferred mantissa is the wrong
object, or `gam1` is not compiled from those GP/GQ tables at this site.

### 14.4 How to read `check_erf190.rs`

It is the Ext80 emulator of branch 190 under a spill mask (`zz`, series,
`j`, `zl`, `gam1` eval, `gam1` return, `g`, `w`, inner association,
`wg` first, `0.5+(0.5−j)` vs `1−j`, etc.). Control word `0x133F` (PC=64,
RN). Exp via F2XM1/FSCALE, ln via FYL2X. Dump mode writes per-row model
phase used to fit the tooth law. If you add an axis, add it here rather
than forking a second emulator; the 850/1,508 plateau is only meaningful
against this baseline.

Research x87 primitives live behind `feature = "research-x87"` in
`oxfunc_core::excel_numeric::research`. Production kernels must not take
that path.

---

## 15. Suggested swarm split

Do not give every agent the whole wall. Assign one rival theory plus one
discriminator.

| Agent | Owns | Must produce |
|---|---|---|
| A | tiny-direct GAUSS, abs(x)≤1e-15 | which of the 480 tied 190-path graphs (or which new axis) the 14 separators will kill; a 14-row Value2 script |
| B | z<0.5 P-body / tooth law | a generator for p(e=−30)=4.35961621689e-11 and the m=1.5 slip, or a refutation that the period is still aliased |
| C | ln-vs-exp residual (T1c) | a read of b25 crossing, or a 200-row full-mantissa substitute if that file is gone |
| D | six Q landmarks near z=1/8 | a graph that is exact on 778/784 *and* those six, or a proof they are a stored-z² artifact |
| E | z≥0.5 Q-tail | unsplit exp(−z²) × *named* rational; must beat 773/784 without a fitted corr(s) |
| F | NORMSDIST vs GAUSS | whether `0.5+GAUSS` is the CDF graph except tinies; 64-row ladder across 1e-15 and 2⁻⁴⁹ |
| G | complement direction 20228 replay | 40-row provenance-rich ERF/ERFC pair across 0.5±eps, confirming 700/700 still holds |
| H | `gam1(½)` object | is `0x906eba8214db6c6f` measurable (implied from tiny ERF / g), or a phantom of the 190-path fit |

Agents A and D have the best chance of an exact landing this cycle.
Agents B/C/E are research. Agent F is a wrapper that may close NORMSDIST
without closing the body. Agent H is diagnostic: if the mantissa is
phantom, T1a’s “wrong gam1” story is the wrong story.

---

## 16. Pointers (read before inventing a parallel corpus)

Primary:

- this file
- `docs/function-lane/W109_G3-01_GRATIO_IDENTIFICATION_20260716.md` (§erf
  sub-lane, tooth law, b18, b25)
- `docs/function-lane/W109_CAMPAIGN_RESUME_20260718.md` (open wall 3)
- `docs/function-lane/W109_CHIDIST_DF1_ERFC_IDENTITY_20260818.md`
- `smart-fuzzer/tools/calc_graph_racer/ERF_GAUSS_EXACT_GRAPH_DISCOVERY_CHECKPOINT_20260809.md`
- `smart-fuzzer/tools/calc_graph_racer/ERF_GAUSS_ROUTE_STORE_OFFLINE_CHECKPOINT_20260809.md`
- `smart-fuzzer/tools/calc_graph_racer/ERF_GAUSS_DIRECT_TINY_TIE_MINING_OFFLINE_CHECKPOINT_20260809.md`
- `smart-fuzzer/tools/calc_graph_racer/src/bin/check_erf190.rs`
- `crates/oxfunc_core/src/functions/special_dist_family.rs`
- `crates/oxfunc_core/src/functions/normal_dist_common.rs`
- `docs/function-lane/ERFC_EXCEL_EMULATION.md` (history of the failed fit)
- `docs/EXCEL_MATH_DEVIATION_CATALOG.md` XMD-011
- `docs/function-lane/DISCREPANCY_CALCULATION_MAP.csv` rows G3-07, G4-04
- `CHARTER.md` §3–4 and `AGENTS.md` Clean-room / Reverse-Engineering Provenance

Public algorithm texts (allowed): TOMS 654 GRATIO / NSWC `gam1` and
branch 190; TOMS 708 only as contrast (beta); fdlibm `s_erf.c` /
`s_erfc.c` as ruled-out controls; Cody CALERF as ruled-out control.

---

## 17. Example of a useful agent reply (shape only)

Good:

> Graph G-A1: GAUSS tiny-direct, `abs(x) <= 0x3cd203af9ee75616`,
> `z = RN53(abs(x) * 0x3fe6a09e667f3bcd)`, then branch 190 with
> `j` associated as `a*x*(1/(a+1) + x*(sum/6 - 0.5/(a+2)))`,
> `gam1` binary64-per-op, `g = 1+h` in Ext80, `ans = (w*g)*(1-j)`,
> `w = exp(zl)` FYL2X/F2XM1 unsplit. Predicts the 80-member tie
> class on the 14 separators, and ERF(z) = 2*ans at those z.
> Discriminator: Value2 the seven positive separator bit-patterns
> from the tiny-tie checkpoint; if Excel GAUSS on any of them
> disagrees with G-A1, the graph is dead. Must already be exact
> on the 2,822-row plateau or explain each of the 336 ULP-sum
> misses as a different predicate.

Bad:

> Maybe Excel uses a slightly different rational. I would try
> fitting more coefficients, or look at the binary. x87 is
> probably involved.

---

## 18. One-paragraph assignment

Excel's `ERF`/`ERFC`/`ERF.PRECISE`/`ERFC.PRECISE`, the ordinary and tiny
routes of `GAUSS`, and the CDF of `NORMSDIST`/`NORM.S.DIST` (hence
`NORM.DIST` and `LOGNORM.DIST` CDFs) are publications of one unidentified
scalar evaluation tree, already wrapped by identified argument transforms
(`z = |x|/√2` stored; `SQRT(x/2)` into ERFC for chi df=1; complement
direction flipping at 0.5). The small-z family is TOMS 654 branch 190
with `gam1(1/2)`, raced to a 850/1,508 and 2,822/3,158 plateau of ±1 ULP
teeth, not to an exact graph. Public erf libraries, CRT exp, split `z²`,
compensated complements, half-erf GAUSS, and correction polynomials are
dead. Your job is to name the missing op, store, or truncation that
reproduces the tooth law and the six z=1/8 Q landmarks, and to specify
the single Excel capture that would kill your graph if it is wrong.
