# ERFC / ERFC.PRECISE — Excel emulation notes

## Policy

DnaCalc is an Excel calculation emulator. For direct-function exactness
work, the acceptance criterion is bit-exact reproduction of Excel's
observed output doubles, even when Excel is numerically inferior to
correctly-rounded `libm` / scientific math results. Mathematical
correctness is diagnostic, not the acceptance criterion.

## Regime map (after commits `8e435fb` + `<this>`)

Evidence sources:

- `DnaOneCalc/target/triage/erfc-regime-map-after-8e435fb-normal-batch-output`
- `DnaOneCalc/target/triage/erfc-threshold-map-2-after-8e435fb-normal-batch-output`
- In-tree probe: `cargo test -p oxfunc_core --lib score_erfc_rule_families -- --ignored --nocapture`
  (probe deleted between rounds; rerun by temporarily re-adding it.)

Matched means the owner-local kernel reproduces Excel bit-exactly.

### Positive x

| x | Excel bits | libm Δ | UCRT Δ | current kernel |
|---|---|---|---|---|
| 0 | `0x3ff0000000000000` | 0 | 0 | libm ✓ |
| 0.5 | `0x3fdeb02147ce245c` | 0 | −1 | libm ✓ |
| 1 | `0x3fc4226162fbddd5` | 0 | −1 | libm ✓ |
| 1.25 | `0x3fb3bcd133aa0ffc` | 0 | 0 | libm ✓ |
| 1.5 | `0x3fa15aaa8ec85204` | +1 | +1 | libm ✗ |
| 1.75 | `0x3f8b4be201caa4b4` | −1 | −1 | libm ✗ |
| 1.9 | `0x3f7d87c86a71bbac` | +5 | +5 | libm ✗ |
| 2.0 | `0x3f7328f5ec350e65` | +2 | +1 | libm ✗ |
| 2.1 | `0x3f686864fb26b019` | −2 | −2 | libm ✗ |
| 2.25 | `0x3f57f713f9cc9783` | +1 | +1 | libm ✗ |
| 2.5 | `0x3f3aab859b20ac9d` | +1 | +2 | libm ✗ |
| 2.6 | `0x3f2ef000330f0609` | +3 | +3 | libm ✗ |
| 2.7 | `0x3f219b75731c79ae` | +1 | +1 | libm ✗ |
| 2.75 | `0x3f1a609f7584d32c` | 0 | −1 | libm ✓ |
| 2.8 | `0x3f13aa0cdf15cedb` | 0 | −2 | libm ✓ |
| 2.9 | `0x3f058c1056c73870` | +2 | +3 | libm ✗ |
| 3.0 | `0x3ef729df6503422a` | −1 | 0 | UCRT ✓ (win-msvc) |
| 3.5 | `0x3ea8ef2a9a18d858` | −1 | −3 | UCRT ✗ (worse) |
| 4.0 | `0x3e508ddd13bd35e6` | +1 | 0 | UCRT ✓ (win-msvc) |
| 5.0 | `0x3d7b0c1a759f7734` | +6 | +6 | UCRT ✗ |
| 6.0 | `0x3c78cf81557d20b8` | −1 | −2 | UCRT ✗ (worse) |
| 8.0 | `0x39ec74fc41217dfd` | −2 | 0 | UCRT ✓ (win-msvc) |
| 10.0 | `0x36a7d8a7f2a8a2cf` | +1 | +1 | UCRT ✗ |

### Negative x

All tested negatives (−1.25, −1.5, −1.75, −1.9, −2, −2.1, −2.25, −2.5,
−2.6, −2.7, −2.75, −2.8, −2.9, −3, −3.5, −4, −5, −6, −8, −10) match
Excel exactly via `libm::erfc`. Kept on libm.

## Implemented rule

`crates/oxfunc_core/src/functions/special_dist_family.rs::excel_erfc`:

```
if cfg!(all(target_os = "windows", target_env = "msvc")) && x >= 3.0 {
    ucrt_erfc(x)            // via `#[link(name = "ucrt")]`
} else {
    libm::erfc(x)
}
```

### Why `x >= 3.0` specifically

Rule-family scoring across the full 23-point widened positive witness set:

| rule | exact matches |
|---|---|
| libm-only | 6 |
| UCRT-only | 5 (regresses x=0.5, 1, 2.75, 2.8) |
| min(libm, UCRT) | 3 |
| max(libm, UCRT) | 8 |
| **UCRT if x>=3 else libm** | **9** |
| UCRT if x>=2.9 else libm | 9 |
| UCRT if x>=4 else libm | 8 |
| UCRT if x>=8 else libm | 7 |
| libm−1 ULP on [1.5,2.7] else libm | 10 (rejected — speculative bit arithmetic, no evidence for untested x in [1.5,2.7]) |
| libm+1 ULP on [1.5,2.7] else libm | 7 |
| (libm+UCRT)/2 rounded | 3 |

`x >= 3.0` is the simplest threshold that maximises exact-bit matches
without regressing a single Excel-matched anchor.

### Platform conditionality

- Windows + MSVC toolchain: UCRT is available via `#[link(name =
  "ucrt")]`. Rule active.
- Any other platform: UCRT unavailable, fall back to `libm::erfc` for the
  full domain. The x>=3 positive tail has accepted residual divergence
  from Excel there — acknowledged as a known platform gap until/unless
  a cross-platform Excel-emulation polynomial is developed.

## Known gaps (not yet reproduced)

The following positive-x witness points remain blocked under every rule
family we tested. Their Excel bits are captured in
`special_dist_family::tests::erfc_known_blocked_excel_witnesses`
(`#[ignore]`d; run via `cargo test -- --ignored`).

```
1.5, 1.75, 1.9, 2.0, 2.1, 2.25, 2.5, 2.6, 2.7, 2.9, 3.5, 5.0, 6.0, 10.0
```

Excel uses a polynomial that neither `libm::erfc` (Sun msun port) nor
MSVC's UCRT `erfc` reproduces at these inputs. Closing this gap requires
identifying/porting Excel's specific approximation — out of scope for an
owner-local bounded fix.

## Verification protocol

After any change to `excel_erfc`:

1. Run the bit-exact in-tree witnesses:
   ```
   cargo test -p oxfunc_core --lib special_dist_family::tests::erfc
   ```
   All non-ignored tests must pass on Windows-MSVC. Off-Windows the
   large-tail test is compile-time gated off.

2. Request a DnaOneCalc proof batch at minimum:
   ```
   =ERFC(0), =ERFC(0.5), =ERFC(1), =ERFC(1.25),
   =ERFC(2.75), =ERFC(2.8),
   =ERFC(3), =ERFC(4), =ERFC(8),
   =ERFC(-1), =ERFC(-2), =ERFC(-4),
   +  the ERFC.PRECISE mirrors,
   +  the currently-blocked set if the change targets it:
   =ERFC(1.5), =ERFC(1.75), =ERFC(1.9), =ERFC(2), =ERFC(2.1),
   =ERFC(2.25), =ERFC(2.5), =ERFC(2.6), =ERFC(2.7), =ERFC(2.9),
   =ERFC(3.5), =ERFC(5), =ERFC(6), =ERFC(10).
   ```
   Any flip from Matched→Blocked at a previously-matched anchor is a
   regression that blocks the commit.

## Reversibility

The change is one helper function in one file. Reverting is
`git revert` or restoring `erfc_kernel` to `Ok(libm::erfc(x))` and
dropping the extern block.
