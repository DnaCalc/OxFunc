# Excel annuity functions (PMT/FV/PV/IPMT/PPMT/CUM): algorithm + W108 Phase-C findings

*Status: CHARACTERIZED, not yet bit-exact. The authoritative algorithm is pinned; the residual
is a POWER-class transcendental-substrate problem (Excel's exact internal `log1p`/`expm1`
rounding), still open. The crate financial family stays frozen on the portable correctly-rounded
core pending the crack. This document is the seed for a dedicated reverse-engineering pass, in the
same shape as [`EXCEL_POWER_SPEC_AND_TEST_CASES.md`](EXCEL_POWER_SPEC_AND_TEST_CASES.md) was for
POWER.*

Target: Excel 16.0 build 20131, 64-bit, Windows, AMD Zen2. All inputs fed as exact doubles via
`Range.Value2`; results read as raw bit patterns.

---

## 1. The authoritative algorithm

Excel's time-value functions share their heritage with OpenOffice/LibreOffice (the LibreOffice
implementations were "borrowed from OpenOffice 1.0" and mirror Excel). The exact formulas are in
LibreOffice `sc/source/core/tool/interpr2.cxx`
([source](https://cgit.freedesktop.org/libreoffice/core/tree/sc/source/core/tool/interpr2.cxx)):

```cpp
// PMT — forward form via exp/log1p/expm1 (NOT pow):
if (bPayInAdvance)  // type = 1
    pmt = (fv + pv*exp(nper*log1p(rate))) * rate /
          (expm1((nper+1)*log1p(rate)) - rate);
else                // type = 0
    pmt = (fv + pv*exp(nper*log1p(rate))) * rate / expm1(nper*log1p(rate));
return -pmt;

// FV — pow-based:
term = pow(1.0+rate, nper);
if (bPayInAdvance) fv = pv*term + pmt*(1.0+rate)*(term-1.0)/rate;
else               fv = pv*term + pmt*(term-1.0)/rate;
return -fv;

// PV — pow(1+r, -n):
if (bPayInAdvance)
    pv = fv*pow(1+rate,-nper) + pmt*(1 - pow(1+rate,-nper+1))/rate + pmt;
else
    pv = fv*pow(1+rate,-nper) + pmt*(1 - pow(1+rate,-nper))/rate;
return -pv;
```

The two families use **different** power methods, confirmed against live Excel:

- **PMT** uses `exp(n·log1p(r))` and `expm1(n·log1p(r))` — the forward `(1+r)^n` via the x87
  `exp`/`log1p`/`expm1`, NOT `pow`.
- **FV / PV** use `pow(1+r, ±n)`, which is the Excel `POWER` routine
  (`crate::excel_numeric::excel_pow_positive` etc.).

IPMT/PPMT/CUMPRINC/CUMIPMT are built on PMT plus a running balance (`fv_exp`), so they inherit
whatever PMT and the balance recurrence use.

---

## 2. Confirmed (live Excel, this campaign)

1. **The FV/PV factor `(1+r)^n` is exactly `power_kernel` (Excel POWER)** — 120/120 on an
   integer+fractional-n sweep isolated via `FV(r,n,0,-1,0) = (1+r)^n`. Not `exp(n·ln(1+r))`
   (56/120) and not `exp(n·log1p(r))` (10/120). So FV/PV's `pow` = the bit-exact POWER routine.
2. **PMT is the forward form** `-(fv+pv·exp(n·log1p(r)))·r / expm1(n·log1p(r))`, not the discount
   form the crate currently uses. The x87 substrate `log1p` = `FYL2XP1` and `expm1` = `F2XM1`-based
   is the right direction (M1 45/150 exact; `ln(1+r)` for log1p → 11/150; `exp(x)-1` for expm1 →
   24/150 — both worse).
3. **FV's annuity term prefers divide-first** `pmt·tf·((q-1)/r)` (74/90) over mult-first
   `pmt·tf·(q-1)/r` (65/90).

## 3. Open (the residual)

With the authoritative formulas + best x87 substrate, best-candidate accuracy on a 290-row
realistic+adversarial sweep:

| function | bit-exact | residual |
|----------|-----------|----------|
| FV  | 74/90  | mostly 1 ULP |
| PMT | 45/150 | 52 rows @1 ULP, 25 @2 ULP |
| PV  | 16/50  | wide — the reconstructed PV op-order is not yet Excel's |

The PMT/FV residual is **1-2 ULP**, concentrated where the transcendental substrate is exercised.
It is the same class as the pre-crack POWER residual: Excel's internal `log1p` and `expm1` have
their own bespoke rounding (they may be `FYL2XP1`/`F2XM1`, or a software routine — undetermined),
and the multi-step composition compounds sub-ULP differences from `log1p`, `expm1`, and the exact
f64 arithmetic order. PV additionally needs its exact op-order pinned (the LibreOffice `ScGetPV`
form reproduced here is not yet matching Excel's — Excel likely diverged from OpenOffice on PV).

**What a dedicated pass needs to nail:** (a) Excel's internal `log1p(r)` bit pattern (isolate via
`PMT(r,n,0,1,0) = -r/expm1(n·log1p r)` and `exp(n·log1p r)`); (b) Excel's internal `expm1`
rounding across the `|x| ≤ ln2` (`F2XM1`) and `|x| > ln2` regimes; (c) the exact f64 op-order of
each function (esp. PV); (d) then IPMT/PPMT/CUM via the balance recurrence.

---

## 4. Test witnesses (all `fv=0`, so PMT reduces to `-pv·(1+r)^n·r/expm1(n·log1p r)`)

`x87fwd` = the forward-form x87 model (§1 PMT, `FYL2XP1`+`F2XM1`).

### PMT — AGREE (model reproduces Excel)
```
r=0.00375   n=24  pv=92838.45   type=1  excel=0xc0af8a1cc7814075
r=0.00666667 n=60 pv=671341.59  type=0  excel=0xc0ca9631820a9d6e
r=0.0165833 n=24  pv=168673.67  type=1  excel=0xc0c07a5178b2b075
```
### PMT — DIVERGE (1-2 ULP; the residual to crack)
```
r=0.00375   n=24  pv=655719.05  type=1  excel=0xc0dbd87197774a52  x87fwd=0xc0dbd87197774a53
r=0.005     n=60  pv=182607.64  type=0  excel=0xc0ab94a2702615e0  x87fwd=0xc0ab94a2702615e2
r=0.0025    n=180 pv=890329.61  type=1  excel=0xc0b7f51eb9edec0e  x87fwd=0xc0b7f51eb9edec0d
r=0.0165833 n=360 pv=888359.13  type=0  excel=0xc0ccd9ca1d3c302a  x87fwd=0xc0ccd9ca1d3c3029
r=0.00208333 n=12 pv=713383.46  type=1  excel=0xc0ed5c6e7d21dbc6  x87fwd=0xc0ed5c6e7d21dbc8
r=0.005     n=36  pv=207968.39  type=0  excel=0xc0b8b6cd256fa8d5  x87fwd=0xc0b8b6cd256fa8d8
```

---

## 5. Current crate state (unchanged)

The financial family (`financial_time_value_family.rs`, `cumulative_finance_family.rs`) stays on
the portable correctly-rounded core (W108 Bead C): PMT/IPMT/PPMT/NPER/CUM use the **discount**
form `exp(-n·log1p(r))`/`expm1` with `exp_portable`/`log_portable`/`excel_log1p`/`excel_expm1`
(glibc-CR). This is structurally different from Excel's forward form but tuned to a ≤2-3 ULP
residual and is not regressed by this campaign. FV/PV already route `(1+r)^n` through the
bit-exact `power_kernel` (via `growth`).

**Do not partially migrate** to the forward form until the substrate is cracked — the forward-form
x87 model is not yet more bit-exact than the frozen discount form on realistic inputs, so a swap
would trade one ≤2-3 ULP approximation for another without a net win.

## 6. Reproduce

Scratch harness `x87lab` (`x87fin.rs` = FV/PV/PMT candidate compositions with x87 exp/ln/log1p/
expm1/mul/recip; `x87fac.rs` = factor isolation; `compare_fin.ps1` / `compare_fac.ps1` drive Excel
via `smart-fuzzer/tools/CellRefBatch.psm1`).
