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
    ln_gamma(z).exp()
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
    ln_gamma(a).exp()
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
    let w = x.exp();
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
        let top =
            (((((P[6] * t + P[5]) * t + P[4]) * t + P[3]) * t + P[2]) * t + P[1]) * t + P[0];
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
    let mut gam = w.exp() * (1.0 + tail);
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
            let w = z.exp();
            let ans = w * g * (0.5 + (0.5 - j));
            return (ans, 0.5 + (0.5 - ans));
        }
        // a < 1, x >= 1.1
        let t1 = a * x.ln() - x;
        let u = a * t1.exp();
        if u == 0.0 {
            return (1.0, 0.0);
        }
        let r = u * (1.0 + gratio_gam1(a));
        return gratio_cf(a, x, r, e, acc);
    }

    // a >= 1
    if a < GRATIO_BIG[iop - 1] {
        if a > x || x >= x0 {
            // statement 20
            let t1 = a * x.ln() - x;
            let r = t1.exp() / gratio_norm_gamma(a);
            return gratio_after40(a, x, r, e, acc, x0, &mut wk);
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
                summ = (-x).exp();
                t = summ;
                n = 1i64;
                c = 0.0;
            } else {
                // statement 220: half-integer a, finite sum.
                // DEVIATION 2: erfc1(0, sqrt(x)) == Q(0.5, x) == gratio(0.5, x).1
                let rtx = x.sqrt();
                summ = gratio(0.5, x).1;
                t = (-x).exp() / (GRATIO_RTPI * rtx);
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
        let r = t1.exp() / gratio_norm_gamma(a);
        return gratio_after40(a, x, r, e, acc, x0, &mut wk);
    }

    // statement 30: a >= big
    let l = x / a;
    if l == 0.0 {
        return (0.0, 1.0);
    }
    let s = 0.5 + (0.5 - l);
    let z = gratio_rlog(l);
    if z >= 700.0 / a {
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
    let r = GRATIO_RT2PIN * rta * t1.exp();
    gratio_after40(a, x, r, e, acc, x0, &mut wk)
}

fn gratio_after40(
    a: f64,
    x: f64,
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
        let p = (r / a) * ans;
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
    let c = (-y).exp();
    // DEVIATION 3: w = 0.5*erfc1(1, sqrt(y)) = 0.5*exp(y)*Q(0.5, y).
    let w = 0.5 * (y.exp() * gratio(0.5, y).1);
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
        let c0 = ((((((((((((D0[12] * z + D0[11]) * z + D0[10]) * z + D0[9]) * z + D0[8]) * z
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
        let c2 = (((((((((D2[9] * z + D2[8]) * z + D2[7]) * z + D2[6]) * z + D2[5]) * z + D2[4])
            * z
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
        let c4 = (((((D4[5] * z + D4[4]) * z + D4[3]) * z + D4[2]) * z + D4[1]) * z + D4[0]) * z
            + D40;
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

fn beta_continued_fraction(a: f64, b: f64, x: f64) -> f64 {
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < f64::MIN_POSITIVE {
        d = f64::MIN_POSITIVE;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..=200 {
        let m2 = 2.0 * m as f64;
        let aa = m as f64 * (b - m as f64) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < f64::MIN_POSITIVE {
            d = f64::MIN_POSITIVE;
        }
        c = 1.0 + aa / c;
        if c.abs() < f64::MIN_POSITIVE {
            c = f64::MIN_POSITIVE;
        }
        d = 1.0 / d;
        h *= d * c;

        let aa = -(a + m as f64) * (qab + m as f64) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < f64::MIN_POSITIVE {
            d = f64::MIN_POSITIVE;
        }
        c = 1.0 + aa / c;
        if c.abs() < f64::MIN_POSITIVE {
            c = f64::MIN_POSITIVE;
        }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;
        if (delta - 1.0).abs() < 1e-15 {
            break;
        }
    }
    h
}

pub fn regularized_beta(x: f64, a: f64, b: f64) -> f64 {
    if !(0.0..=1.0).contains(&x) || !(a > 0.0) || !(b > 0.0) {
        return f64::NAN;
    }
    if x == 0.0 {
        return 0.0;
    }
    if x == 1.0 {
        return 1.0;
    }
    let bt = (ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b) + a * x.ln() + b * (1.0 - x).ln()).exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        bt * beta_continued_fraction(a, b, x) / a
    } else {
        1.0 - bt * beta_continued_fraction(b, a, 1.0 - x) / b
    }
}

pub fn bisect_inverse<F>(target: f64, mut lo: f64, mut hi: f64, f: F) -> f64
where
    F: Fn(f64) -> f64,
{
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let value = f(mid);
        if value >= target {
            hi = mid;
        } else {
            lo = mid;
        }
        if (hi - lo).abs() <= 4.0 * f64::EPSILON * (1.0 + mid.abs()) {
            return hi;
        }
    }
    hi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamma_and_ln_gamma_match_known_values() {
        assert!((gamma(5.0) - 24.0).abs() < 1e-10);
        assert!((gamma(0.5) - std::f64::consts::PI.sqrt()).abs() < 1e-10);
        assert!((ln_gamma(5.0) - 24.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn regularized_gamma_matches_known_values() {
        assert!(regularized_gamma_p(0.5, 0.5).is_finite());
        assert!((regularized_gamma_p(2.0, 2.0) - 0.593_994_150_290_161_9).abs() < 1e-10);
        assert!((regularized_gamma_q(2.0, 2.0) - 0.406_005_849_709_838_1).abs() < 1e-10);
    }

    #[test]
    fn regularized_beta_matches_known_values() {
        assert!((regularized_beta(0.5, 2.0, 2.0) - 0.5).abs() < 1e-12);
        assert!((regularized_beta(0.25, 2.0, 3.0) - 0.261_718_75).abs() < 1e-12);
    }

    fn ulp_gap(a: f64, x: f64) -> u64 {
        let (ab, bb) = (a.to_bits(), x.to_bits());
        if ab >= bb { ab - bb } else { bb - ab }
    }

    // Pinned live-Excel (16.0) witnesses proving the GRATIO port reproduces
    // Excel's incomplete-gamma-ratio kernel bit-for-bit. Both cases have a > x
    // (statement 20 -> gamma_nswc normalizer -> statement 50 Taylor tail), the
    // Q side being returned as 0.5 + (0.5 - P). Q(5, 0.5) and Q(1, 0.5) are the
    // internal arguments behind CHIDIST(1,10) and CHIDIST(1,2) respectively.
    #[test]
    fn gratio_pinned_live_excel_witnesses() {
        // CHIDIST(1,10) => Q(df/2=5, x/2=0.5).
        let (p5, q5) = gratio(5.0, 0.5);
        assert_eq!(
            q5.to_bits(),
            0x3fef_fe97_0c1f_f154,
            "Q(5,0.5) bits {:#018x} (gap {} ULP vs live Excel)",
            q5.to_bits(),
            ulp_gap(q5, f64::from_bits(0x3fef_fe97_0c1f_f154))
        );
        // CHIDIST(1,2) => Q(df/2=1, x/2=0.5).
        let (p1, q1) = gratio(1.0, 0.5);
        assert_eq!(
            q1.to_bits(),
            0x3fe3_68b2_fc6f_960a,
            "Q(1,0.5) bits {:#018x} (gap {} ULP vs live Excel)",
            q1.to_bits(),
            ulp_gap(q1, f64::from_bits(0x3fe3_68b2_fc6f_960a))
        );
        // The returned (P, Q) pair is complementary to within rounding.
        assert!((p5 + q5 - 1.0).abs() < 1e-15);
        assert!((p1 + q1 - 1.0).abs() < 1e-15);
    }

    // The public wrappers must preserve the previous kernel's invalid-input and
    // boundary contract (NAN for out-of-domain, 0/1 at x == 0).
    #[test]
    fn regularized_gamma_domain_contract_preserved() {
        assert!(regularized_gamma_p(-1.0, 1.0).is_nan());
        assert!(regularized_gamma_q(-1.0, 1.0).is_nan());
        assert!(regularized_gamma_p(1.0, -1.0).is_nan());
        assert!(regularized_gamma_q(1.0, -1.0).is_nan());
        assert!(regularized_gamma_p(f64::NAN, 1.0).is_nan());
        assert!(regularized_gamma_q(1.0, f64::INFINITY).is_nan());
        assert_eq!(regularized_gamma_p(2.5, 0.0), 0.0);
        assert_eq!(regularized_gamma_q(2.5, 0.0), 1.0);
    }

    // --- OLD Newton-Raphson kernel (verbatim pre-port copy), retained ONLY for
    //     the head-to-head comparison in the ignored `gratio_dominates_old_kernel`
    //     test below. Not used by any shipping code path. ---
    fn old_nr_gamma_p(a: f64, x: f64) -> f64 {
        if !(a > 0.0) || !(x >= 0.0) || !a.is_finite() || !x.is_finite() {
            return f64::NAN;
        }
        if x == 0.0 {
            return 0.0;
        }
        if x < a + 1.0 {
            let gln = ln_gamma(a);
            let mut sum = 1.0 / a;
            let mut term = sum;
            let mut ap = a;
            for _ in 0..200 {
                ap += 1.0;
                term *= x / ap;
                sum += term;
                if term.abs() < sum.abs() * 1e-15 {
                    break;
                }
            }
            sum * (-x + a * x.ln() - gln).exp()
        } else {
            1.0 - old_nr_gamma_q(a, x)
        }
    }
    fn old_nr_gamma_q(a: f64, x: f64) -> f64 {
        if !(a > 0.0) || !(x >= 0.0) || !a.is_finite() || !x.is_finite() {
            return f64::NAN;
        }
        if x == 0.0 {
            return 1.0;
        }
        let gln = ln_gamma(a);
        let mut b = x + 1.0 - a;
        let mut c = 1.0 / f64::MIN_POSITIVE;
        let mut d = 1.0 / b;
        let mut h = d;
        for i in 1..=200 {
            let an = -(i as f64) * ((i as f64) - a);
            b += 2.0;
            d = an * d + b;
            if d.abs() < f64::MIN_POSITIVE {
                d = f64::MIN_POSITIVE;
            }
            c = b + an / c;
            if c.abs() < f64::MIN_POSITIVE {
                c = f64::MIN_POSITIVE;
            }
            d = 1.0 / d;
            let delta = d * c;
            h *= delta;
            if (delta - 1.0).abs() < 1e-15 {
                break;
            }
        }
        (-x + a * x.ln() - gln).exp() * h
    }

    // Head-to-head over the full live-Excel witness corpus. Ignored by default
    // because it reads the identification JSON from the smart-fuzzer work tree
    // (present on the identification host only). Run with:
    //   cargo test -p oxfunc_core gratio_dominates_old_kernel -- --ignored --nocapture
    #[test]
    #[ignore = "reads live-Excel witness JSON from smart-fuzzer/work/w109 (identification host only)"]
    fn gratio_dominates_old_kernel() {
        use serde_json::Value;
        fn bits(h: &str) -> f64 {
            f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
        }
        fn run(path: &str, func: &str) -> (usize, usize, u64, u64, usize) {
            let text = match std::fs::read_to_string(path) {
                Ok(t) => t,
                Err(_) => return (0, 0, 0, 0, 0),
            };
            let v: Value = serde_json::from_str(&text).unwrap();
            let mut old_exact = 0;
            let mut new_exact = 0;
            let mut old_max = 0u64;
            let mut new_max = 0u64;
            let mut tot = 0;
            for w in v["witnesses"].as_array().unwrap() {
                let args: Vec<f64> = w["args"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|a| bits(a.as_str().unwrap()))
                    .collect();
                let exp = bits(w["expected_bits"].as_str().unwrap());
                let (vnew, vold) = match func {
                    "CHIDIST" => {
                        let (a, xx) = (args[1] / 2.0, args[0] / 2.0);
                        (gratio(a, xx).1, old_nr_gamma_q(a, xx))
                    }
                    "GAMMA.DIST" => {
                        let a = args[1];
                        if a == 1.0 {
                            continue; // Excel dispatches a==1 to expm1 at the wrapper
                        }
                        let xx = args[0] / args[2];
                        (gratio(a, xx).0, old_nr_gamma_p(a, xx))
                    }
                    _ => unreachable!(),
                };
                tot += 1;
                if vnew.to_bits() == exp.to_bits() {
                    new_exact += 1;
                } else {
                    new_max = new_max.max(ulp_gap(vnew, exp));
                }
                if vold.to_bits() == exp.to_bits() {
                    old_exact += 1;
                } else if vold.is_finite() {
                    old_max = old_max.max(ulp_gap(vold, exp));
                } else {
                    old_max = u64::MAX; // non-finite: catastrophic divergence
                }
            }
            (old_exact, new_exact, old_max, new_max, tot)
        }
        let base = r"C:\Work\DnaCalc\OxFunc\smart-fuzzer\work\w109\G3-01-dist";
        for (file, func) in [
            (r"answers-chidist.json", "CHIDIST"),
            (r"answers-gammadist-modern.json", "GAMMA.DIST"),
        ] {
            let path = format!(r"{}\{}", base, file);
            let (oe, ne, om, nm, tot) = run(&path, func);
            if tot == 0 {
                eprintln!("SKIP {func}: witness JSON not present at {path}");
                continue;
            }
            eprintln!(
                "{func}: OLD exact {oe}/{tot} maxULP {om} | NEW exact {ne}/{tot} maxULP {nm}"
            );
            assert!(ne > oe, "{func}: new kernel not more exact ({ne} !> {oe})");
            assert!(nm <= om, "{func}: new kernel max ULP {nm} not <= old {om}");
        }
    }
}
