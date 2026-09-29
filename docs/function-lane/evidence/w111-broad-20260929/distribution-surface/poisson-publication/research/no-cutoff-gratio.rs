pub fn ln_gamma(z: f64) -> f64 {
    const COEFFS: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    const G: f64 = 7.0;

    if z < 0.5 {
        return std::f64::consts::PI.ln()
            - (std::f64::consts::PI * z).sin().ln()
            - ln_gamma(1.0 - z);
    }

    let x = COEFFS
        .iter()
        .enumerate()
        .skip(1)
        .fold(COEFFS[0], |acc, (i, coeff)| {
            acc + coeff / (z - 1.0 + i as f64)
        });
    let t = z - 1.0 + G + 0.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (z - 0.5) * t.ln() - t + x.ln()
}

pub fn gamma(z: f64) -> f64 {
    if z.is_sign_negative() && z.fract() == 0.0 {
        return f64::NAN;
    }
    if z < 0.5 {
        return std::f64::consts::PI / ((std::f64::consts::PI * z).sin() * gamma(1.0 - z));
    }
    crate::excel_numeric::excel_exp(ln_gamma(z))
}

// ---------------------------------------------------------------------------
// GRATIO: faithful port of the DCDFLIB / NSWC incomplete-gamma-ratio kernel
// (TOMS 654, DiDonato & Morris; Alfred H. Morris Jr.), transcribed from the
// scipy v0.14 cdflib Fortran sources via the validated Python reference at
// smart-fuzzer/work/w109/G3-01-dist/cdflib_py.py.
//
// Live-Excel identification (W109) proved this is Excel's actual algorithm for
// the incomplete-gamma family (CHIDIST / GAMMA.DIST / GAMMADIST / CHISQ.*),
// executed in plain SSE2 double with std log/exp. This kernel replaces the
// previous Newton-Raphson-style series/continued-fraction pair, collapsing the
// catastrophic ULP drift (up to ~6224 ULP) those exhibited versus Excel.
//
// IDENTIFICATION-DRIVEN DEVIATIONS FROM VANILLA NSWC (do not "restore"):
//   1. The `a == 0.5` dispatch to erf/erfc (gratio.f statement 390) is OMITTED:
//      live-Excel proof shows Excel routes a = 0.5 through the general a < 1
//      branches (Excel's ERF.PRECISE *is* that path). So a = 0.5 flows into the
//      a < 1 code below.
//   2. The half-integer closed-form branch (statement 220) needs erfc1(0, sqrt(x)).
//      Excel's erfc IS gratio(0.5, z^2).q, so erfc1(0, sqrt(x)) == Q(0.5, x); we
//      implement it as a depth-1 recursive call `gratio(0.5, x).1`.
//   3. Consequently erf_nswc / erfc1 are not ported. The Temme a >= big branch's
//      single scaled-erfc need (erfc1(1, sqrt(y)) == exp(y)*Q(0.5, y)) is likewise
//      wired through the recursion, keeping the structure complete without an
//      erf implementation. (No witness in the identification corpus reaches the
//      Temme branch; it is ported for structural fidelity.)
// ---------------------------------------------------------------------------

const GRATIO_ALOG10: f64 = 2.30258509299405;
const GRATIO_RT2PIN: f64 = 0.398942280401433;
const GRATIO_RTPI: f64 = 1.77245385090552;
const GRATIO_THIRD: f64 = 0.333333333333333;
const GRATIO_E: f64 = 2.220446049250313e-16; // spmpar(1) = 2^-52
const GRATIO_SPMPAR3: f64 = 1.7976931348623157e308;

// Temme expansion coefficient tables (d0..d7 of gratio.f).
const D0: [f64; 13] = [
    0.833333333333333e-01,
    -0.148148148148148e-01,
    0.115740740740741e-02,
    0.352733686067019e-03,
    -0.178755144032922e-03,
    0.391926317852244e-04,
    -0.218544851067999e-05,
    -0.185406221071516e-05,
    0.829671134095309e-06,
    -0.176659527368261e-06,
    0.670785354340150e-08,
    0.102618097842403e-07,
    -0.438203601845335e-08,
];
const D10: f64 = -0.185185185185185e-02;
const D1: [f64; 12] = [
    -0.347222222222222e-02,
    0.264550264550265e-02,
    -0.990226337448560e-03,
    0.205761316872428e-03,
    -0.401877572016461e-06,
    -0.180985503344900e-04,
    0.764916091608111e-05,
    -0.161209008945634e-05,
    0.464712780280743e-08,
    0.137863344691572e-06,
    -0.575254560351770e-07,
    0.119516285997781e-07,
];
const D20: f64 = 0.413359788359788e-02;
const D2: [f64; 10] = [
    -0.268132716049383e-02,
    0.771604938271605e-03,
    0.200938786008230e-05,
    -0.107366532263652e-03,
    0.529234488291201e-04,
    -0.127606351886187e-04,
    0.342357873409614e-07,
    0.137219573090629e-05,
    -0.629899213838006e-06,
    0.142806142060642e-06,
];
const D30: f64 = 0.649434156378601e-03;
const D3: [f64; 8] = [
    0.229472093621399e-03,
    -0.469189494395256e-03,
    0.267720632062839e-03,
    -0.756180167188398e-04,
    -0.239650511386730e-06,
    0.110826541153473e-04,
    -0.567495282699160e-05,
    0.142309007324359e-05,
];
const D40: f64 = -0.861888290916712e-03;
const D4: [f64; 6] = [
    0.784039221720067e-03,
    -0.299072480303190e-03,
    -0.146384525788434e-05,
    0.664149821546512e-04,
    -0.396836504717943e-04,
    0.113757269706784e-04,
];
const D50: f64 = -0.336798553366358e-03;
const D5: [f64; 4] = [
    -0.697281375836586e-04,
    0.277275324495939e-03,
    -0.199325705161888e-03,
    0.679778047793721e-04,
];
const D60: f64 = 0.531307936463992e-03;
const D6: [f64; 2] = [-0.592166437353694e-03, 0.270878209671804e-03];
const D70: f64 = 0.344367606892378e-03;

// gratio.f iop-selected constant vectors, indexed [iop-1]. Excel uses ind = 0
// (unscaled) exclusively, hence iop = 1 throughout; the higher-iop entries are
// retained so the branch routing matches the reference exactly.
const GRATIO_ACC0: [f64; 3] = [5e-15, 5e-7, 5e-4];
const GRATIO_BIG: [f64; 3] = [20.0, 14.0, 10.0];
const GRATIO_E00: [f64; 3] = [0.25e-3, 0.25e-1, 0.14];
const GRATIO_X00: [f64; 3] = [31.0, 17.0, 9.7];

/// CR-quality normalizer Gamma(a) for the gratio prefactor. Identification
/// (W109 agent-B normalizer sweep): Excel divides exp(t1) by a value equal to
/// the correctly-rounded double Gamma(a) to within 1 ULP (= exp of its
/// internal lgamma), decisively NOT the NSWC gamma routine (+22..+33 ULP).
/// Integer a: exact factorial. Half-integer a: exact odd double-factorial
/// times sqrt(pi-double), one rounding. Generic a: exp(ln_gamma) fallback.
fn gratio_norm_gamma(a: f64) -> f64 {
    if a >= 1.0 && a <= 22.0 && a == a.floor() {
        let mut f = 1.0f64;
        let mut k = 2.0f64;
        while k < a {
            f *= k;
            k += 1.0;
        }
        return f;
    }
    let n_half = a - 0.5;
    if a >= 0.5 && a <= 22.0 && n_half == n_half.floor() {
        let n = n_half as i32;
        let mut df = 1.0f64; // (2n-1)!! exact below 2^53 for n <= 21
        let mut m = 1.0f64;
        for _ in 0..n {
            df *= m;
            m += 2.0;
        }
        let sqrt_pi = std::f64::consts::PI.sqrt();
        return df * sqrt_pi / f64::powi(2.0, n);
    }
    crate::excel_numeric::excel_exp(ln_gamma(a))
}

/// exparg(l) of exparg.f (specialized to this x86_64 host's double range).
fn gratio_exparg(l: i32) -> f64 {
    let lnb = 0.69314718055995;
    if l != 0 {
        let m = -1021 - 1;
        0.99999 * (m as f64 * lnb)
    } else {
        let m = 1024;
        0.99999 * (m as f64 * lnb)
    }
}

/// rexp.f: evaluation of exp(x) - 1.
fn gratio_rexp(x: f64) -> f64 {
    const P1: f64 = 0.914041914819518e-09;
    const P2: f64 = 0.238082361044469e-01;
    const Q1: f64 = -0.499999999085958e+00;
    const Q2: f64 = 0.107141568980644e+00;
    const Q3: f64 = -0.119041179760821e-01;
    const Q4: f64 = 0.595130811860248e-03;
    if x.abs() <= 0.15 {
        return x * (((P2 * x + P1) * x + 1.0) / ((((Q4 * x + Q3) * x + Q2) * x + Q1) * x + 1.0));
    }
    let w = crate::excel_numeric::excel_exp(x);
    if x <= 0.0 {
        (w - 0.5) - 0.5
    } else {
        w * (0.5 + (0.5 - 1.0 / w))
    }
}

/// rlog.f: computation of x - 1 - ln(x).
fn gratio_rlog(x: f64) -> f64 {
    const A: f64 = 0.566749439387324e-01;
    const B: f64 = 0.456512608815524e-01;
    const P0: f64 = 0.333333333333333e+00;
    const P1: f64 = -0.224696413112536e+00;
    const P2: f64 = 0.620886815375787e-02;
    const Q1: f64 = -0.127408923933623e+01;
    const Q2: f64 = 0.354508718369557e+00;
    if x < 0.61 || x > 1.57 {
        let r = (x - 0.5) - 0.5;
        return r - x.ln();
    }
    let u;
    let w1;
    if x < 0.82 {
        let uu = (x - 0.7) / 0.7;
        u = uu;
        w1 = A - uu * 0.3;
    } else if x > 1.18 {
        let uu = 0.75 * x - 1.0;
        u = uu;
        w1 = B + uu / 3.0;
    } else {
        u = (x - 0.5) - 0.5;
        w1 = 0.0;
    }
    let r = u / (u + 2.0);
    let t = r * r;
    let w = ((P2 * t + P1) * t + P0) / ((Q2 * t + Q1) * t + 1.0);
    2.0 * t * (1.0 / (1.0 - r) - r * w) + w1
}

/// gam1.f: computation of 1/gamma(a+1) - 1 for -0.5 <= a <= 1.5.
fn gratio_gam1(a: f64) -> f64 {
    const P: [f64; 7] = [
        0.577215664901533e+00,
        -0.409078193005776e+00,
        -0.230975380857675e+00,
        0.597275330452234e-01,
        0.766968181649490e-02,
        -0.514889771323592e-02,
        0.589597428611429e-03,
    ];
    const Q: [f64; 5] = [
        0.100000000000000e+01,
        0.427569613095214e+00,
        0.158451672430138e+00,
        0.261132021441447e-01,
        0.423244297896961e-02,
    ];
    const R: [f64; 9] = [
        -0.422784335098468e+00,
        -0.771330383816272e+00,
        -0.244757765222226e+00,
        0.118378989872749e+00,
        0.930357293360349e-03,
        -0.118290993445146e-01,
        0.223047661158249e-02,
        0.266505979058923e-03,
        -0.132674909766242e-03,
    ];
    const S1: f64 = 0.273076135303957e+00;
    const S2: f64 = 0.559398236957378e-01;
    let mut t = a;
    let d = a - 0.5;
    if d > 0.0 {
        t = d - 0.5;
    }
    if t == 0.0 {
        return 0.0;
    }
    if t > 0.0 {
        let top = (((((P[6] * t + P[5]) * t + P[4]) * t + P[3]) * t + P[2]) * t + P[1]) * t + P[0];
        let bot = (((Q[4] * t + Q[3]) * t + Q[2]) * t + Q[1]) * t + 1.0;
        let w = top / bot;
        if d > 0.0 {
            (t / a) * ((w - 0.5) - 0.5)
        } else {
            a * w
        }
    } else {
        let top = (((((((R[8] * t + R[7]) * t + R[6]) * t + R[5]) * t + R[4]) * t + R[3]) * t
            + R[2])
            * t
            + R[1])
            * t
            + R[0];
        let bot = (S2 * t + S1) * t + 1.0;
        let w = top / bot;
        if d > 0.0 {
            t * w / a
        } else {
            a * ((w + 0.5) + 0.5)
        }
    }
}

// ---- exp_rd: round-toward-zero exp (W109 chopped-exp identification) -------
// Excel's gser-path r = exp(t1)/G publishes a TRUNCATED exp, not nearest:
// floor(true exp) scores 38/45 on the implied-exp corpus vs CR 25/45 and
// fdlibm 28/45 (every real 2010-era MSVC CRT exp refuted by direct binary
// probes: 32-bit SSE2 rounds one-sided HIGH, x87 and x64-AMD are CR here).
// Corpus effect at the series call site: +26/306 training, +3/111 held-out
// b20 (fractional-a normalizer noise caps the held-out margin).
// Implementation: Tang-style 64-entry table exp in double-double (~2^-100),
// then directed truncation. The RD bit can only be wrong if exp(x) sits
// within ~2^-100 relative of a 53-bit boundary.

#[cfg(not(target_arch = "x86_64"))]
const EXP_RD_L1_BITS: u64 = 0x3f862e4200000000; // ln2/64 hi, 32 trailing zero bits
#[cfg(not(target_arch = "x86_64"))]
const EXP_RD_L2A_BITS: u64 = 0x3e3fdf473de6af28;
#[cfg(not(target_arch = "x86_64"))]
const EXP_RD_L2B_BITS: u64 = 0xbadc4c67fc0d0951;
#[cfg(not(target_arch = "x86_64"))]
const EXP_RD_INV_L_BITS: u64 = 0x40571547652b82fe; // 64/ln2

#[cfg(not(target_arch = "x86_64"))]
#[rustfmt::skip]
const EXP_RD_TBL: [(u64, u64); 64] = [
    (0x3ff0000000000000, 0x0000000000000000), (0x3ff02c9a3e778061, 0xbc719083535b085d),
    (0x3ff059b0d3158574, 0x3c8d73e2a475b465), (0x3ff0874518759bc8, 0x3c6186be4bb284ff),
    (0x3ff0b5586cf9890f, 0x3c98a62e4adc610b), (0x3ff0e3ec32d3d1a2, 0x3c403a1727c57b53),
    (0x3ff11301d0125b51, 0xbc96c51039449b3a), (0x3ff1429aaea92de0, 0xbc932fbf9af1369e),
    (0x3ff172b83c7d517b, 0xbc819041b9d78a76), (0x3ff1a35beb6fcb75, 0x3c8e5b4c7b4968e4),
    (0x3ff1d4873168b9aa, 0x3c9e016e00a2643c), (0x3ff2063b88628cd6, 0x3c8dc775814a8495),
    (0x3ff2387a6e756238, 0x3c99b07eb6c70573), (0x3ff26b4565e27cdd, 0x3c82bd339940e9d9),
    (0x3ff29e9df51fdee1, 0x3c8612e8afad1255), (0x3ff2d285a6e4030b, 0x3c90024754db41d5),
    (0x3ff306fe0a31b715, 0x3c86f46ad23182e4), (0x3ff33c08b26416ff, 0x3c932721843659a6),
    (0x3ff371a7373aa9cb, 0xbc963aeabf42eae2), (0x3ff3a7db34e59ff7, 0xbc75e436d661f5e3),
    (0x3ff3dea64c123422, 0x3c8ada0911f09ebc), (0x3ff4160a21f72e2a, 0xbc5ef3691c309278),
    (0x3ff44e086061892d, 0x3c489b7a04ef80d0), (0x3ff486a2b5c13cd0, 0x3c73c1a3b69062f0),
    (0x3ff4bfdad5362a27, 0x3c7d4397afec42e2), (0x3ff4f9b2769d2ca7, 0xbc94b309d25957e3),
    (0x3ff5342b569d4f82, 0xbc807abe1db13cad), (0x3ff56f4736b527da, 0x3c99bb2c011d93ad),
    (0x3ff5ab07dd485429, 0x3c96324c054647ad), (0x3ff5e76f15ad2148, 0x3c9ba6f93080e65e),
    (0x3ff6247eb03a5585, 0xbc9383c17e40b497), (0x3ff6623882552225, 0xbc9bb60987591c34),
    (0x3ff6a09e667f3bcd, 0xbc9bdd3413b26456), (0x3ff6dfb23c651a2f, 0xbc6bbe3a683c88ab),
    (0x3ff71f75e8ec5f74, 0xbc816e4786887a99), (0x3ff75feb564267c9, 0xbc90245957316dd3),
    (0x3ff7a11473eb0187, 0xbc841577ee04992f), (0x3ff7e2f336cf4e62, 0x3c705d02ba15797e),
    (0x3ff82589994cce13, 0xbc9d4c1dd41532d8), (0x3ff868d99b4492ed, 0xbc9fc6f89bd4f6ba),
    (0x3ff8ace5422aa0db, 0x3c96e9f156864b27), (0x3ff8f1ae99157736, 0x3c85cc13a2e3976c),
    (0x3ff93737b0cdc5e5, 0xbc675fc781b57ebc), (0x3ff97d829fde4e50, 0xbc9d185b7c1b85d1),
    (0x3ff9c49182a3f090, 0x3c7c7c46b071f2be), (0x3ffa0c667b5de565, 0xbc9359495d1cd533),
    (0x3ffa5503b23e255d, 0xbc9d2f6edb8d41e1), (0x3ffa9e6b5579fdbf, 0x3c90fac90ef7fd31),
    (0x3ffae89f995ad3ad, 0x3c97a1cd345dcc81), (0x3ffb33a2b84f15fb, 0xbc62805e3084d708),
    (0x3ffb7f76f2fb5e47, 0xbc75584f7e54ac3b), (0x3ffbcc1e904bc1d2, 0x3c823dd07a2d9e84),
    (0x3ffc199bdd85529c, 0x3c811065895048dd), (0x3ffc67f12e57d14b, 0x3c92884dff483cad),
    (0x3ffcb720dcef9069, 0x3c7503cbd1e949db), (0x3ffd072d4a07897c, 0xbc9cbc3743797a9c),
    (0x3ffd5818dcfba487, 0x3c82ed02d75b3707), (0x3ffda9e603db3285, 0x3c9c2300696db532),
    (0x3ffdfc97337b9b5f, 0xbc91a5cd4f184b5c), (0x3ffe502ee78b3ff6, 0x3c839e8980a9cc8f),
    (0x3ffea4afa2a490da, 0xbc9e9c23179c2893), (0x3ffefa1bee615a27, 0x3c9dc7f486a4b6b0),
    (0x3fff50765b6e4540, 0x3c99d3e12dd8a18b), (0x3fffa7c1819e90d8, 0x3c874853f3a5931e),
];

#[cfg(not(target_arch = "x86_64"))]
#[rustfmt::skip]
const EXP_RD_COEF: [(u64, u64); 11] = [ // 1/k!, k = 10..=0, Horner order
    (0x3e927e4fb7789f5c, 0x3b3cbbc05b4fa99a), (0x3ec71de3a556c734, 0xbb6c154f8ddc6c00),
    (0x3efa01a01a01a01a, 0x3b3a01a01a01a01a), (0x3f2a01a01a01a01a, 0x3b6a01a01a01a01a),
    (0x3f56c16c16c16c17, 0xbbef49f49f49f49f), (0x3f81111111111111, 0x3c01111111111111),
    (0x3fa5555555555555, 0x3c45555555555555), (0x3fc5555555555555, 0x3c65555555555555),
    (0x3fe0000000000000, 0x0000000000000000), (0x3ff0000000000000, 0x0000000000000000),
    (0x3ff0000000000000, 0x0000000000000000),
];

#[cfg(not(target_arch = "x86_64"))]
#[inline]
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bb = s - a;
    (s, (a - (s - bb)) + (b - bb))
}

#[cfg(not(target_arch = "x86_64"))]
#[inline]
fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    (p, a.mul_add(b, -p))
}

#[cfg(not(target_arch = "x86_64"))]
#[inline]
fn dd_mul(ah: f64, al: f64, bh: f64, bl: f64) -> (f64, f64) {
    let (p, e) = two_prod(ah, bh);
    two_sum(p, e + ah * bl + al * bh)
}

pub fn exp_rd(x: f64) -> f64 {
    // W109 F2XM1 identification: the series-site chop is RZ53 of the x87 fFEXP
    // chain. On x86_64 run the real chain; elsewhere fall back to the validated
    // double-double floor-of-true core (they differ only on the rare rows where
    // the chain's extended value crosses a rounding boundary the true value
    // does not).
    #[cfg(target_arch = "x86_64")]
    {
        crate::excel_numeric::excel_exp_rz(x)
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        exp_rd_portable(x)
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn exp_rd_portable(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return crate::excel_numeric::excel_exp(x);
    }
    let k = (x * f64::from_bits(EXP_RD_INV_L_BITS)).round();
    let ki = k as i64;
    let m = ki >> 6;
    let j = (ki - (m << 6)) as usize;
    if !(-1022..=1022).contains(&m) {
        // overflow / deep-subnormal territory: not exercised by the kernel's
        // series call site; plain exp keeps the guards upstream working.
        return crate::excel_numeric::excel_exp(x);
    }
    // r = x - k*ln2/64, double-double
    let t = x - k * f64::from_bits(EXP_RD_L1_BITS); // exact: 32 trailing zero bits, |k| < 2^17
    let (p1h, p1l) = two_prod(k, f64::from_bits(EXP_RD_L2A_BITS));
    let (rh0, rl0) = two_sum(t, -p1h);
    let (rh, rl) = two_sum(rh0, rl0 - p1l - k * f64::from_bits(EXP_RD_L2B_BITS));
    // Horner in double-double over 1/k!
    let (c0h, c0l) = EXP_RD_COEF[0];
    let (mut ph, mut pl) = (f64::from_bits(c0h), f64::from_bits(c0l));
    for &(ch, cl) in &EXP_RD_COEF[1..] {
        let (th, tl) = dd_mul(ph, pl, rh, rl);
        let (sh, se) = two_sum(th, f64::from_bits(ch));
        let (h, l) = two_sum(sh, se + tl + f64::from_bits(cl));
        ph = h;
        pl = l;
    }
    let (tbh, tbl) = EXP_RD_TBL[j];
    let (vh, vl) = dd_mul(ph, pl, f64::from_bits(tbh), f64::from_bits(tbl));
    // truncate toward zero BEFORE scaling (vh in [1,2), so vl's sign cannot
    // be lost to underflow), then scale exactly by 2^m.
    let d = if vl < 0.0 {
        f64::from_bits(vh.to_bits() - 1)
    } else {
        vh
    };
    if m == -1022 && d < 1.0 {
        // scaled result would be subnormal; directed rounding not preserved.
        return crate::excel_numeric::excel_exp(x);
    }
    let scale = f64::from_bits(((1023 + m) as u64) << 52);
    d * scale
}

/// gamma_fort.f (Morris's GAMMA): the un-normalized gamma used as the gratio
/// normalizer via r = exp(a*ln x - x) / gamma(a).
#[allow(dead_code)] // superseded by gratio_norm_gamma (identification: Excel != NSWC gamma)
fn gratio_gamma_nswc(a: f64) -> f64 {
    const PI: f64 = 3.1415926535898;
    const D: f64 = 0.41893853320467274178;
    const P: [f64; 7] = [
        0.539637273585445e-03,
        0.261939260042690e-02,
        0.204493667594920e-01,
        0.730981088720487e-01,
        0.279648642639792e+00,
        0.553413866010467e+00,
        1.0,
    ];
    const Q: [f64; 7] = [
        -0.832979206704073e-03,
        0.470059485860584e-02,
        0.225211131035340e-01,
        -0.170458969313360e+00,
        -0.567902761974940e-01,
        0.113062953091122e+01,
        1.0,
    ];
    const R1: f64 = 0.820756370353826e-03;
    const R2: f64 = -0.595156336428591e-03;
    const R3: f64 = 0.793650663183693e-03;
    const R4: f64 = -0.277777777770481e-02;
    const R5: f64 = 0.833333333333333e-01;
    let mut x = a;
    let mut s = 0.0;
    if a.abs() < 15.0 {
        let mut t = 1.0;
        let m = (a as i64) - 1;
        if m >= 0 {
            for _ in 0..m {
                x -= 1.0;
                t = x * t;
            }
            x -= 1.0;
        } else {
            t = a;
            if a <= 0.0 {
                let mm = -m - 1;
                for _ in 0..mm {
                    x += 1.0;
                    t = x * t;
                }
                x = (x + 0.5) + 0.5;
                t = x * t;
                if t == 0.0 {
                    return 0.0;
                }
            }
            if t.abs() < 1e-30 {
                if t.abs() * GRATIO_SPMPAR3 <= 1.0001 {
                    return 0.0;
                }
                return 1.0 / t;
            }
        }
        let mut top = P[0];
        let mut bot = Q[0];
        for i in 1..7 {
            top = P[i] + x * top;
            bot = Q[i] + x * bot;
        }
        let g = top / bot;
        return if a < 1.0 { g / t } else { g * t };
    }
    // |a| >= 15
    if a.abs() >= 1e3 {
        return 0.0;
    }
    if a <= 0.0 {
        x = -a;
        let n = x as i64;
        let mut tt = x - n as f64;
        if tt > 0.9 {
            tt = 1.0 - tt;
        }
        s = (PI * tt).sin() / PI;
        if n % 2 == 0 {
            s = -s;
        }
        if s == 0.0 {
            return 0.0;
        }
    }
    let t = 1.0 / (x * x);
    let mut g = ((((R1 * t + R2) * t + R3) * t + R4) * t + R5) / x;
    let lnx = x.ln();
    let z = x;
    g = (D + g) + (z - 0.5) * (lnx - 1.0);
    let w = g;
    let tail = g - w;
    if w > 0.99999 * gratio_exparg(0) {
        return 0.0;
    }
    let mut gam = crate::excel_numeric::excel_exp(w) * (1.0 + tail);
    if a < 0.0 {
        gam = (1.0 / (gam * s)) / x;
    }
    gam
}

/// Excel's incomplete-gamma-ratio kernel. Returns `(P(a,x), Q(a,x))`.
/// `(2.0, 2.0)` is the NSWC error sentinel for invalid arguments.
pub fn gratio(a: f64, x: f64) -> (f64, f64) {
    let e = GRATIO_E;
    if a < 0.0 || x < 0.0 || (a == 0.0 && x == 0.0) {
        return (2.0, 2.0);
    }
    if a * x == 0.0 {
        return if x <= a { (0.0, 1.0) } else { (1.0, 0.0) };
    }
    if a == 1.0 {
        // Excel dispatches a == 1 to the exponential CDF (W109: proven by the
        // a = 1+2^-20 contradiction probe; emulator race a=1 slice 179/205
        // with this wrapper). Nearest-rounded exp/expm1 — the chopped series
        // exp does NOT apply on this path.
        return (
            -crate::excel_numeric::excel_expm1_internal(-x),
            crate::excel_numeric::excel_exp(-x),
        );
    }
    // Excel uses ind = 0 (unscaled). gratio.f: iop = ind + 1 = 1 (only ind of
    // 1 or 2 would remap to iop = 3), selecting the tight acc = 5e-15, big = 20,
    // x0 = 31, e0 = 0.25e-3 constant set.
    let iop = 1usize;
    let acc = GRATIO_ACC0[iop - 1].max(e);
    let e0 = GRATIO_E00[iop - 1];
    let x0 = GRATIO_X00[iop - 1];

    let mut wk = [0.0f64; 21]; // 1-based, indices 1..=20

    if a < 1.0 {
        // DEVIATION 1: no a == 0.5 special dispatch; 0.5 flows into these
        // general a < 1 branches.
        if x < 1.1 {
            // statement 160: Taylor series for small a.
            let mut an = 3.0;
            let mut c = x;
            let mut summ = x / (a + 3.0);
            let tol = 3.0 * acc / (a + 1.0);
            loop {
                an += 1.0;
                c = -c * (x / an);
                let t = c / (a + an);
                summ += t;
                if t.abs() <= tol {
                    break;
                }
            }
            let j = a * x * ((summ / 6.0 - 0.5 / (a + 2.0)) * x + 1.0 / (a + 1.0));
            let z = a * x.ln();
            let h = gratio_gam1(a);
            let g = 1.0 + h;
            let go200 = if x < 0.25 { z > -0.13394 } else { a < x / 2.59 };
            if go200 {
                // statement 200
                let l = gratio_rexp(z);
                let w = 0.5 + (0.5 + l);
                let qans = (w * j - l) * g - h;
                if qans < 0.0 {
                    return (1.0, 0.0);
                }
                return (0.5 + (0.5 - qans), qans);
            }
            // statement 190
            let w = crate::excel_numeric::excel_exp(z);
            let ans = w * g * (0.5 + (0.5 - j));
            return (ans, 0.5 + (0.5 - ans));
        }
        // a < 1, x >= 1.1
        let t1 = a * x.ln() - x;
        let u = a * crate::excel_numeric::excel_exp(t1);
        if u == 0.0 {
            return (1.0, 0.0);
        }
        let r = u * (1.0 + gratio_gam1(a));
        return gratio_cf(a, x, r, e, acc);
    }

    // a >= 1
    if a < GRATIO_BIG[iop - 1] {
        if a > x || x >= x0 {
            // statement 20. Series arm publishes r from the CHOPPED exp
            // (W109 chopped-exp identification); CF/asymptotic arms keep the
            // nearest-rounded exp (chop measurably hurts there).
            let t1 = a * x.ln() - x;
            let g = gratio_norm_gamma(a);
            return gratio_after40(
                a,
                x,
                exp_rd(t1) / g,
                crate::excel_numeric::excel_exp(t1) / g,
                e,
                acc,
                x0,
                &mut wk,
            );
        }
        let twoa = a + a;
        let m = twoa as i64;
        if twoa == m as f64 {
            let i = m / 2;
            let mut summ;
            let mut t;
            let mut n;
            let mut c;
            if a == i as f64 {
                // statement 210: integer a, finite sum
                summ = crate::excel_numeric::excel_exp(-x);
                t = summ;
                n = 1i64;
                c = 0.0;
            } else {
                // statement 220: half-integer a, finite sum.
                // DEVIATION 2: erfc1(0, sqrt(x)) == Q(0.5, x) == gratio(0.5, x).1
                let rtx = x.sqrt();
                summ = gratio(0.5, x).1;
                t = crate::excel_numeric::excel_exp(-x) / (GRATIO_RTPI * rtx);
                n = 0i64;
                c = -0.5;
            }
            while n != i {
                n += 1;
                c += 1.0;
                t = (x * t) / c;
                summ += t;
            }
            let qans = summ;
            return (0.5 + (0.5 - qans), qans);
        }
        let t1 = a * x.ln() - x;
        let g = gratio_norm_gamma(a);
        return gratio_after40(
            a,
            x,
            exp_rd(t1) / g,
            crate::excel_numeric::excel_exp(t1) / g,
            e,
            acc,
            x0,
            &mut wk,
        );
    }

    // statement 30: a >= big
    let l = x / a;
    if l == 0.0 {
        return (0.0, 1.0);
    }
    let s = 0.5 + (0.5 - l);
    let z = gratio_rlog(l);
    if false && z >= 700.0 / a {
        if s.abs() <= 2.0 * e {
            return (2.0, 2.0);
        }
        return if x <= a { (0.0, 1.0) } else { (1.0, 0.0) };
    }
    let y = a * z;
    let rta = a.sqrt();
    if s.abs() <= e0 / rta {
        return gratio_temme_330(a, l, y, z, e, s, iop);
    }
    if s.abs() <= 0.4 {
        return gratio_temme_270(a, l, y, z, e, s, iop);
    }
    let t = (1.0 / a).powi(2);
    let mut t1 = (((0.75 * t - 1.0) * t + 3.5) * t - 105.0) / (a * 1260.0);
    t1 -= y;
    let r = GRATIO_RT2PIN * rta * crate::excel_numeric::excel_exp(t1);
    // a >= big: no chopped-exp evidence for this r staging; same r both arms.
    gratio_after40(a, x, r, r, e, acc, x0, &mut wk)
}

fn gratio_after40(
    a: f64,
    x: f64,
    r_series: f64,
    r: f64,
    _e: f64,
    acc: f64,
    x0: f64,
    wk: &mut [f64; 21],
) -> (f64, f64) {
    if r == 0.0 {
        return if x <= a { (0.0, 1.0) } else { (1.0, 0.0) };
    }
    if x <= a.max(GRATIO_ALOG10) {
        // Taylor series. Identification (W109 agent-B, a=2 slice): Excel sums
        // FORWARD (1 + c1 + c2 + ...) with 1/a as an OUTER factor — not the
        // NSWC wk[] backward-tail staging (28/45 vs 16/45 bit-exact).
        // Refined (session 4): the exact form is Cephes-igam style — ans
        // accumulates FROM 1.0, stop at c/ans <= MACHEP (2^-53), publication
        // (r/a)*ans. 25/45 at a=2 with CR exp (vs 16/45 for 1+summ staging);
        // the residual is Excel's statically-linked legacy-CRT exp (one-sided
        // low, fdlibm-proxy 28/45).
        let _ = acc;
        let mut rr = a;
        let mut c = 1.0f64;
        let mut ans = 1.0f64;
        loop {
            rr += 1.0;
            c *= x / rr;
            ans += c;
            if c / ans <= 1.110_223_024_625_156_5e-16 {
                break;
            }
        }
        let p = (r_series / a) * ans;
        return (p, 0.5 + (0.5 - p));
    }
    if x < x0 {
        return gratio_cf(a, x, r, _e, acc);
    }
    // statement 100: asymptotic expansion with wk backward-tail summation.
    let mut amn = a - 1.0;
    let mut t = amn / x;
    wk[1] = t;
    let mut n = 20usize;
    let mut broke = false;
    for n_ in 2..=20 {
        amn -= 1.0;
        t *= amn / x;
        if t.abs() <= 1e-3 {
            n = n_;
            broke = true;
            break;
        }
        wk[n_] = t;
    }
    if !broke {
        n = 20;
    }
    let mut summ = t;
    while t.abs() > acc {
        amn -= 1.0;
        t *= amn / x;
        summ += t;
    }
    let mx = n - 1;
    for _ in 0..mx {
        n -= 1;
        summ += wk[n];
    }
    let qans = (r / x) * (1.0 + summ);
    (0.5 + (0.5 - qans), qans)
}

/// statement 250: unnormalized continued fraction (two-term recurrences).
fn gratio_cf(a: f64, x: f64, r: f64, e: f64, acc: f64) -> (f64, f64) {
    let tol = (5.0 * e).max(acc);
    let mut a2nm1 = 1.0;
    let mut a2n = 1.0;
    let mut b2nm1 = x;
    let mut b2n = x + (1.0 - a);
    let mut c = 1.0;
    let mut an0;
    loop {
        a2nm1 = x * a2n + c * a2nm1;
        b2nm1 = x * b2n + c * b2nm1;
        let am0 = a2nm1 / b2nm1;
        c += 1.0;
        let cma = c - a;
        a2n = a2nm1 + cma * a2n;
        b2n = b2nm1 + cma * b2n;
        an0 = a2n / b2n;
        if !((an0 - am0).abs() >= tol * an0) {
            break;
        }
    }
    let qans = r * an0;
    (0.5 + (0.5 - qans), qans)
}

/// statement 270: Temme expansion for a >= big, |1 - x/a| <= 0.4.
fn gratio_temme_270(a: f64, l: f64, y: f64, z: f64, e: f64, s: f64, iop: usize) -> (f64, f64) {
    if s.abs() <= 2.0 * e && a * e * e > 3.28e-3 {
        return (2.0, 2.0);
    }
    let c = crate::excel_numeric::excel_exp(-y);
    // DEVIATION 3: w = 0.5*erfc1(1, sqrt(y)) = 0.5*exp(y)*Q(0.5, y).
    let w = 0.5 * (crate::excel_numeric::excel_exp(y) * gratio(0.5, y).1);
    let u = 1.0 / a;
    let mut zz = (z + z).sqrt();
    if l < 1.0 {
        zz = -zz;
    }
    let z = zz;
    let t;
    if iop == 1 {
        if s.abs() <= 1e-3 {
            return gratio_temme_340(a, l, z, u, c, w);
        }
        let c0 = ((((((((((((D0[12] * z + D0[11]) * z + D0[10]) * z + D0[9]) * z + D0[8])
            * z
            + D0[7])
            * z
            + D0[6])
            * z
            + D0[5])
            * z
            + D0[4])
            * z
            + D0[3])
            * z
            + D0[2])
            * z
            + D0[1])
            * z
            + D0[0])
            * z
            - GRATIO_THIRD;
        let c1 = (((((((((((D1[11] * z + D1[10]) * z + D1[9]) * z + D1[8]) * z + D1[7]) * z
            + D1[6])
            * z
            + D1[5])
            * z
            + D1[4])
            * z
            + D1[3])
            * z
            + D1[2])
            * z
            + D1[1])
            * z
            + D1[0])
            * z
            + D10;
        let c2 =
            (((((((((D2[9] * z + D2[8]) * z + D2[7]) * z + D2[6]) * z + D2[5]) * z + D2[4]) * z
                + D2[3])
                * z
                + D2[2])
                * z
                + D2[1])
                * z
                + D2[0])
                * z
                + D20;
        let c3 = (((((((D3[7] * z + D3[6]) * z + D3[5]) * z + D3[4]) * z + D3[3]) * z + D3[2])
            * z
            + D3[1])
            * z
            + D3[0])
            * z
            + D30;
        let c4 =
            (((((D4[5] * z + D4[4]) * z + D4[3]) * z + D4[2]) * z + D4[1]) * z + D4[0]) * z + D40;
        let c5 = (((D5[3] * z + D5[2]) * z + D5[1]) * z + D5[0]) * z + D50;
        let c6 = (D6[1] * z + D6[0]) * z + D60;
        t = ((((((D70 * u + c6) * u + c5) * u + c4) * u + c3) * u + c2) * u + c1) * u + c0;
    } else if iop == 2 {
        let c0 = (((((D0[5] * z + D0[4]) * z + D0[3]) * z + D0[2]) * z + D0[1]) * z + D0[0]) * z
            - GRATIO_THIRD;
        let c1 = (((D1[3] * z + D1[2]) * z + D1[1]) * z + D1[0]) * z + D10;
        let c2 = D2[0] * z + D20;
        t = (c2 * u + c1) * u + c0;
    } else {
        t = ((D0[2] * z + D0[1]) * z + D0[0]) * z - GRATIO_THIRD;
    }
    gratio_temme_finish(l, c, w, t, a)
}

fn gratio_temme_340(a: f64, l: f64, z: f64, u: f64, c: f64, w: f64) -> (f64, f64) {
    let c0 = ((((((D0[6] * z + D0[5]) * z + D0[4]) * z + D0[3]) * z + D0[2]) * z + D0[1]) * z
        + D0[0])
        * z
        - GRATIO_THIRD;
    let c1 = (((((D1[5] * z + D1[4]) * z + D1[3]) * z + D1[2]) * z + D1[1]) * z + D1[0]) * z + D10;
    let c2 = ((((D2[4] * z + D2[3]) * z + D2[2]) * z + D2[1]) * z + D2[0]) * z + D20;
    let c3 = (((D3[3] * z + D3[2]) * z + D3[1]) * z + D3[0]) * z + D30;
    let c4 = (D4[1] * z + D4[0]) * z + D40;
    let c5 = (D5[1] * z + D5[0]) * z + D50;
    let c6 = D6[0] * z + D60;
    let t = ((((((D70 * u + c6) * u + c5) * u + c4) * u + c3) * u + c2) * u + c1) * u + c0;
    gratio_temme_finish(l, c, w, t, a)
}

/// statement 330: Temme expansion for a >= big, |1 - x/a| <= e0/sqrt(a).
fn gratio_temme_330(a: f64, l: f64, y: f64, z: f64, e: f64, _s: f64, iop: usize) -> (f64, f64) {
    if a * e * e > 3.28e-3 {
        return (2.0, 2.0);
    }
    let c = 0.5 + (0.5 - y);
    let w = (0.5 - y.sqrt() * (0.5 + (0.5 - y / 3.0)) / GRATIO_RTPI) / c;
    let u = 1.0 / a;
    let mut zz = (z + z).sqrt();
    if l < 1.0 {
        zz = -zz;
    }
    let z = zz;
    if iop == 1 {
        return gratio_temme_340(a, l, z, u, c, w);
    }
    let t;
    if iop == 2 {
        let c0 = (D0[1] * z + D0[0]) * z - GRATIO_THIRD;
        let c1 = D1[0] * z + D10;
        t = (D20 * u + c1) * u + c0;
    } else {
        t = D0[0] * z - GRATIO_THIRD;
    }
    gratio_temme_finish(l, c, w, t, a)
}

fn gratio_temme_finish(l: f64, c: f64, w: f64, t: f64, a: f64) -> (f64, f64) {
    let rta = a.sqrt();
    if l >= 1.0 {
        let qans = c * (w + GRATIO_RT2PIN * t / rta);
        (0.5 + (0.5 - qans), qans)
    } else {
        let ans = c * (w - GRATIO_RT2PIN * t / rta);
        (ans, 0.5 + (0.5 - ans))
    }
}

pub fn regularized_gamma_p(a: f64, x: f64) -> f64 {
    if !(a > 0.0) || !(x >= 0.0) || !a.is_finite() || !x.is_finite() {
        return f64::NAN;
    }
    if x == 0.0 {
        return 0.0;
    }
    gratio(a, x).0
}

pub fn regularized_gamma_q(a: f64, x: f64) -> f64 {
    if !(a > 0.0) || !(x >= 0.0) || !a.is_finite() || !x.is_finite() {
        return f64::NAN;
    }
    if x == 0.0 {
        return 1.0;
    }
    gratio(a, x).1
}

