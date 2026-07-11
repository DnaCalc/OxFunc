# W109 Trig Identification And Sign-Off (G4-01) — 2026-07-11

Row `G4-01` (SIN/COS/TAN/COT/SEC/CSC, 1–5664 ULP argument-dependent drift) is
closed. All six calculation graphs were identified by the W109 search and
landed as `excel_sin` / `excel_cos` / `excel_tan` in
`oxfunc_core::excel_numeric` (routed by the six worksheet kernels; the
`|x| >= 2^27 -> #NUM!` guard was already aligned and is unchanged).

## Identified calculation graphs (live Excel 16.0 build 20131, x86-64)

```
SIN(x) = (-1)^Q · FSIN(r),  (r, Q) = FPREM1(x, FLDPI)           [fFSIN]
COS(x) = 1.0 exactly           if |x| < 2^-26  (live bit-ladder; 2^-26 itself
                                takes the chain and publishes 1 - ulp)
       = {FCOS, -FSIN, -FCOS, FSIN}[Q mod 4](r),
         (r, Q) = FPREM1(|x|, FLDPI/2)                          [fFCOS]
TAN(x) = FPTAN(r)              on even Q
       = -(1 / FPTAN(r))       on odd Q, reciprocal in EXTENDED before the
         single binary64 store; (r, Q) = FPREM1(x, FLDPI/2)     [fFTAN]
COT(x) = RN53(RN64(1 / TAN(x)))     (double-rounded recip of published TAN)
CSC(x) = RN53(RN64(1 / SIN(x)))
SEC(x) = RN53(RN64(1 / COS-chain(x)))   (composes to 1.0 at tiny x because
         recip-dr of 1-ulp ties back to exactly 1.0 — no branch needed)
```

**The reduction constant is the 64-bit ROM `FLDPI` π** — not the hardware's
internal 66-bit reduction (raw `FSIN`/`FPTAN` are ruled out by the recon
witnesses) and not any correctly-rounded π (platform libm ruled out, up to
1.27e9 ULP apart on large arguments). This is the whole source of Excel's
large-argument trig "error": δπ = RN64(π) − π ≈ +5.0e-20 scales linearly with
the quotient. The δπ back-solve from the 5664-ULP witness predicted the
`FLDPI` constant before the candidate was run — both recon witnesses then
matched bit-for-bit on the first try.

Key identification details discovered along the way:
- `FPREM1` reports quotient-MAGNITUDE bits, so COS (even) must reduce `|x|`
  for the mod-4 dispatch to be valid — negative-`x` probes caught this.
- COS is NOT `sin(x + π/2)` through the fFSIN chain (`87trig`-plausible but
  138/200 wrong) and NOT π-parity (1-ULP misses at residues near ±π/2).
- The COS `1.0` shortcut threshold was pinned to bit resolution:
  `2^-26 - ulp -> 1.0`, `2^-26 -> FCOS` (adjacent-ulp live ladder).

## Sign-off

- Per-function unique survivors over the live rounds; validation sweeps
  (560 discovery + 460 held-out each): SIN/TAN/COT/CSC/SEC **1020/1020**,
  COS **1044/1044** (incl. the threshold ladder), max ULP 0.
- Production-kernel replay over every deduplicated answered witness:
  **5425/5425 bit-exact** (`verify_trig_promotion`).
- In-crate pins: `trig_matches_live_excel_pinned_witnesses` (the former
  catalog witnesses — 5664/719/351/230/1-ULP rows — plus the threshold).
- Workspace green (1495 lib tests).

## Downstream inheritors (rows stay open, now unblocked)

- **G3-02 GAMMA reflection** and the **Bessel residuals (BUG-FUNC-024)**
  inherit this lane: their kernels' internal trig should now be re-raced
  against `excel_sin`/`excel_cos` staging variants.
- The `raw` x87 primitive layer is now compiled unconditionally (the
  `research-x87` feature only gates the public research re-export).

## Ruled out (ledger updated)

Platform libm trig; raw hardware FSIN/FCOS/FPTAN (66-bit internal reduction);
FPREM/FPREM1 against 2π (with residue extended or stored); π-parity for
COS/TAN; `sin(x+π/2)` COS; strict (single-rounded) reciprocals for
COT/CSC/SEC; cos/sin ratio forms for COT.
