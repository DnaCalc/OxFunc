# W109 — G6 bond-schedule identifications (2026-07-20)

Cluster push after the G3-01 BINOM landing. Two lanes opened the same day;
both cracked at the SEMANTIC level within hours by exact-constraint methods
(single-witness hypothesis first, lattice capture second, bit-race third).
Work dir: `smart-fuzzer/work/w109/G6-b2b3/` (gitignored). Agents: agent-V
(PRICE staging + landing), agent-W (ACCRINT identification).

## Lane A — G6-03d: PRICE at Actual/360 & Actual/365 (material, ~cents)

**Identity: Excel's PRICE derives the settlement discount fraction with a
UNIVERSAL rule `dsc = E − A`** — it never counts days settlement→next-coupon
directly.

- `E` = coupon period length by basis: `360/f` (bases 0, 2, 4), `365/f`
  (basis 3), actual days (basis 1).
- `A` = accrued days PCD→settlement by basis (actual for 1/2/3, 30/360 for
  0/4).
- On bases 0/1/4, `E − A` coincides with the direct count (except
  settlement-on-the-31st 30/360 rows — where `E − A` is ALSO the fix), which
  is why the rule stayed invisible: it only diverges on Actual/360 and
  Actual/365, where `E` is a fiction (180 / 182.5) and `A` is real days.
- First-try witness check: `PRICE(2020-09-20, 2025-01-01, .06, .03, 103, 2,
  basis 2)`: off = (180−81)/180 = 0.55 → 114.588731 = Excel's 114.5887
  (OxFunc/faithful-formula: 103/180 → 114.550379, 3.8 cents low).

**The oracle disagrees with itself:** Excel's published `COUPDAYSNC` returns
ACTUAL days (103) at bases 2/3 on the same arguments — 458/458 captured rows
at each of bases 2/3 have `pub ≠ E−A`, while all 144 basis-0/1/4 rows have
`pub == E−A`. PRICE's internal DSC is not COUPDAYSNC. (OxFunc's COUP*
functions were never the problem.)

**b37 battery** (7,472 PRICE rows: bonds BE mat 45658 r.06 red103 f2 /
BL mat 45717 r.045 f2 leap-Feb period / BQ mat 45658 r.08 f4; settlement
day-ladders × 8 yields × bases 2,3 + sparse 0,1,4 controls; plus 6×1,060
COUP* rows). Four-variant race (`race_b37.py`, plain-double emulation):

| variant | b2 exact | b3 exact | b0 | b1 | b4 |
|---|---|---|---|---|---|
| direct-dsc / powf | 0/3664 | 0/3664 | 43/48 | 46/48 | 43/48 |
| direct-dsc / binexp | 0/3664 | 0/3664 | 45/48 | 48/48 | 45/48 |
| **e−a / binexp** | **3474/3664** | **3446/3664** | **48/48** | **48/48** | 45/48* |

\* the 3 b4 misses are the emulator's day-count shortcut (US 30/360 used for
European dates), not Excel. Residual after the semantic fix: ±1..4 ULP
staging micro-detail (~190 b2 + ~218 b3 rows) — agent-V racing (fractional
pow realization, per-op x87 DR staging, summation order), then landing
`dsc = e − a` in `pcomp_disc` + fresh b38 held-out gate.

Direction of error vs the F# ExcelFinancialFunctions lineage: EFF (and
OxFunc's faithful port) implements the DOCUMENTED quantity (actual DSR);
Excel's ATP implements the derived one. Same porting-hole pattern as
msvcr100's missing expm1/log1p: the published description is not the code.

## Lane B — G6-02: ACCRINT calc_method semantics (material, was "1 ULP")

The catalog row said one 1-ULP witness. The b39 lattice (25,410 rows: bonds
W/Q/A/E, settlement every 3rd day across ~2.5 periods, bases 0-4, rates
.05/.037/.0615, par 1000/997.5, calc_method both) says the row was hiding a
semantic divergence:

- **Excel calc_method=FALSE is the LEGACY FLAT computation**: one fraction,
  no period structure — `a = days(issue→settlement, by basis) / canonical`,
  result `par·rate/f·a`. Verified e.g. W bond settlement 43833 basis 2 par
  997.5: Excel 37.129167 = 24.9375·(268/180), 268 actual days issue→settle.
- **Excel calc_method=TRUE = the period-aware walk** (partial + 1.0 per
  whole period + tail partial) — what OxFunc implements for TRUE.
- The two are bit-identical on 7,752/12,705 captured pairs (pre-first
  regime, and 30/360-aligned post rows where flat ≡ sum), differ by ±1-2 ULP
  on staging elsewhere pre-first, and split MATERIALLY post-first on actual
  bases (4,227 pairs, e.g. 268/180 flat vs 84/180+1 walk).
- **OxFunc's FALSE (accrue from one period before first) matches NEITHER**
  — production scores 17,967/25,410 with ~6,100 material calc=FALSE rows.
- Remaining sub-classes for agent-W: ±1-2 ULP staging both paths (~1,100
  rows); a material 219-row TRUE-path class on bond Q (quarterly, basis 1,
  post-first) — suspect canonical/period handling; several 3-row
  (settlement × 3 rates) boundary groups.

Production scorer: `check_accrint` bin (calc_graph_racer). Residual bank:
`b39_resid.json`.

## Method note (for the ledger)

Both lanes cracked on the SAME move: treat the single catalog witness as an
exact equation, hand-test the 2-3 structurally distinct conventions against
it in plain arithmetic BEFORE capturing anything, then confirm the winning
hypothesis at the bit level on a lattice. The b37/b39 lattices were designed
AFTER the hypothesis, as confirmation + staging instruments, not as search
corpora. Contrast with the G3 walls, where the op-graph family is genuinely
outside parametrization — here the divergences were one source line each
(`dsc = e − a`; flat-vs-walk behind `calc_method`).

## Lane A — LANDED + GATED (2026-07-20, agent-V)

Staging identification on top of `dsc = e − a`:

1. **Fractional discount pow = the x87 CRT chain** `exp(RN53(RN64(y·ln x)))`
   — byte-identical to production `excel_numeric::excel_pow_chain`, i.e. the
   SAME pow recovered independently in the G3 lane-1 distribution-substrate
   work. Cross-lane confirmation, verified against hardware `x87_serve`.
   Alone lifts b2 3474→3656, b3 3446→3658.
2. **Body = plain single-rounded double**, coupons ascending + separate
   redemption term. Refuted (b2+b3 exact of 7,328): sep+powf 7,120; folded
   redemption 5,117; reversed 6,493; extended-accumulator family 4,665-5,500;
   reciprocal-multiply 4,817; x87-per-op-DR body worse. Winner sep+x87chain
   7,314 (emulator), production 7,458/7,472 overall.

Landed: `pcomp_disc` `dsc = e − a` (universal, all callers);
`excel_bond_pow` fractional branch → `excel_pow_chain` (PRICE-only via
`binexp=true`). Pins `price_dsc_e_minus_a_and_x87_pow_chain_pins` (catalog
witness `0x405ca5adc69c74fb`, basis-3 sibling, settle-on-31st basis-0, pow
discriminator). Full suite 1,510 green, zero pin movement, YIELD pins
byte-stable. **Fresh b38 held-out: 945/945 across all five bases** (5 new
bonds, month-end/leap/n=1/extreme-yield). Coordinator re-ran the gate +
suite independently — reproduced.

Open residual (NOT accepted): 14 extreme-yield rows ±1 ULP
(`agentV_residual14.json`, with 250-bit true-sum bits per row — Excel's
published value sits up to +3 ULP ABOVE the correctly-rounded true sum, an
accumulation op-graph fact). Next probe: adjacent-double yld bracket +
truncated-ladder partial sums (battery b41). Blast radius: DURATION now
inherits the `E−A` dirty on bases 2/3 (its own open lane G6-03c); YIELD
unchanged on its pinned witnesses.

Canonical agent record: `smart-fuzzer/work/w109/G6-b2b3/agentV_results.md`.
