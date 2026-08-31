//! W109 ERFC.PRECISE named-F race (public sources only).
//!
//! Frozen discovery banks only. `answers-b9heldout.json` and both GAUSS
//! heldouts are neither named nor accepted. Coefficients come from:
//! - Cody 1969 / SPECFUN CALERF (netlib specfun/erf, via the published
//!   transportable packet)
//! - NSWC TR 92/425 DERFC / DERFC0 (public NSWC library, Morris 1993)
//! - Cephes 2.9 `ndtr.c` erfc (netlib cephes/cprob, Moshier)
//! - fdlibm `s_erf.c` (SunSoft 1993; G3-01-dist/cdflib copy)
//! - A&S 7.1.14 / Gautschi continued fraction for erfcx
//!
//! Usage (from this crate):
//!   cargo run --release --bin race_erfc_named_f -- ../../work/w109/G3-01-dist

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet, ulp_distance};
use oxfunc_core::excel_numeric::research::{excel_exp, x87_mul};
use oxfunc_core::functions::special_dist_family::erfc_precise_kernel;
use std::collections::BTreeMap;

const FRAC_1_SQRT_2: f64 = f64::from_bits(0x3fe6a09e667f3bcd);

const ERFC_BANKS: [&str; 5] = [
    "answers-erfcp.json",
    "answers-erfcm.json",
    "answers-b7erfc.json",
    "answers-b8erfc.json",
    "answers-b11c.json",
];

const PIN_Z: [f64; 5] = [0.75, 1.28125, 1.875, 2.125, 5.0];

fn flush(v: f64) -> f64 {
    if v.abs() < f64::MIN_POSITIVE { 0.0 } else { v }
}

fn horner(cs: &[f64], x: f64) -> f64 {
    let mut acc = 0.0;
    for &c in cs.iter().rev() {
        acc = acc * x + c;
    }
    acc
}

// --- Cody SPECFUN CALERF (Math. Comp. 1969; netlib specfun/erf) ---

const CODY_A: [f64; 5] = [
    3.1611237438705656,
    113.864154151050156,
    377.485237685302021,
    3209.37758913846947,
    0.185777706184603153,
];
const CODY_B: [f64; 4] = [
    23.6012909523441209,
    244.024637934444173,
    1282.61652607737228,
    2844.23683343917062,
];
const CODY_C: [f64; 9] = [
    0.564188496988670089,
    8.88314979438837594,
    66.1191906371416295,
    298.635138197400131,
    881.95222124176909,
    1712.04761263407058,
    2051.07837782607147,
    1230.33935479799725,
    2.15311535474403846e-8,
];
const CODY_D: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];
const CODY_P: [f64; 6] = [
    0.305326634961232344,
    0.360344899949804439,
    0.125781726111229246,
    0.0160837851487422766,
    6.58749161529837803e-4,
    0.0163153871373020978,
];
const CODY_Q: [f64; 5] = [
    2.56852019228982242,
    1.87295284992346047,
    0.527905102951428412,
    0.0605183413124413191,
    0.00233520497626869185,
];
const CODY_SQRPI: f64 = 0.56418958354775628695;
const CODY_THRESH: f64 = 0.46875;
const CODY_XBIG: f64 = 26.543;

fn cody_erf_small(x: f64) -> f64 {
    let y = x.abs();
    let ysq = if y > 1.11e-16 { y * y } else { 0.0 };
    let mut xnum = CODY_A[4] * ysq;
    let mut xden = ysq;
    for i in 0..3 {
        xnum = (xnum + CODY_A[i]) * ysq;
        xden = (xden + CODY_B[i]) * ysq;
    }
    x * (xnum + CODY_A[3]) / (xden + CODY_B[3])
}

fn cody_erfcx_large(y: f64) -> f64 {
    if y <= 4.0 {
        let mut xnum = CODY_C[8] * y;
        let mut xden = y;
        for i in 0..7 {
            xnum = (xnum + CODY_C[i]) * y;
            xden = (xden + CODY_D[i]) * y;
        }
        (xnum + CODY_C[7]) / (xden + CODY_D[7])
    } else {
        let ysq = 1.0 / (y * y);
        let mut xnum = CODY_P[5] * ysq;
        let mut xden = ysq;
        for i in 0..4 {
            xnum = (xnum + CODY_P[i]) * ysq;
            xden = (xden + CODY_Q[i]) * ysq;
        }
        let r = ysq * (xnum + CODY_P[4]) / (xden + CODY_Q[4]);
        (CODY_SQRPI - r) / y
    }
}

fn cody_split_exp(y: f64) -> f64 {
    let ysq = (y * 16.0).floor() / 16.0;
    let del = (y - ysq) * (y + ysq);
    (-ysq * ysq).exp() * (-del).exp()
}

fn cody_erfc_split(z: f64) -> f64 {
    let y = z.abs();
    let q = if y <= CODY_THRESH {
        1.0 - cody_erf_small(y)
    } else if y >= CODY_XBIG {
        0.0
    } else {
        cody_split_exp(y) * cody_erfcx_large(y)
    };
    flush(if z < 0.0 { 2.0 - q } else { q })
}

fn cody_erfc_unsplit(z: f64, exp_fn: fn(f64) -> f64) -> f64 {
    let y = z.abs();
    let q = if y <= CODY_THRESH {
        1.0 - cody_erf_small(y)
    } else {
        let w = exp_fn(-(y * y));
        flush(w * cody_erfcx_large(y))
    };
    flush(if z < 0.0 { 2.0 - q } else { q })
}

// --- NSWC TR 92/425 DERFC / DERFC0 (public NSWC library) ---

const NSWC_A: [f64; 21] = [
    0.1283791670955125738961589031215e+00,
    -0.3761263890318375246320529677070e+00,
    0.1128379167095512573896158902931e+00,
    -0.2686617064513125175943235372542e-01,
    0.5223977625442187842111812447877e-02,
    -0.8548327023450852832540164081187e-03,
    0.1205533298178966425020717182498e-03,
    -0.1492565035840625090430728526820e-04,
    0.1646211436588924261080723578109e-05,
    -0.1636584469123468757408968429674e-06,
    0.1480719281587021715400818627811e-07,
    -0.1229055530145120140800510155331e-08,
    0.9422759058437197017313055084212e-10,
    -0.6711366740969385085896257227159e-11,
    0.4463222608295664017461758843550e-12,
    -0.2783497395542995487275065856998e-13,
    0.1634095572365337143933023780777e-14,
    -0.9052845786901123985710019387938e-16,
    0.4708274559689744439341671426731e-17,
    -0.2187159356685015949749948252160e-18,
    0.7043407712019701609635599701333e-20,
];

fn nswc_small_erfc(x: f64) -> f64 {
    let t = x * x;
    let w = horner(&NSWC_A, t);
    0.5 + (0.5 - x * (1.0 + w))
}

fn nswc_derfc0(x: f64) -> f64 {
    const P: [f64; 8] = [
        0.16506148041280876191828601e-03,
        0.15471455377139313353998665e-03,
        0.44852548090298868465196794e-04,
        -0.49177280017226285450486205e-05,
        -0.69353602078656412367801676e-05,
        -0.20508667787746282746857743e-05,
        -0.28982842617824971177267380e-06,
        -0.17272433544836633301127174e-07,
    ];
    const Q: [f64; 8] = [
        1.0,
        0.16272656776533322859856317e+01,
        0.12040996037066026106794322e+01,
        0.52400246352158386907601472e+00,
        0.14497345252798672362384241e+00,
        0.25592517111042546492590736e-01,
        0.26869088293991371028123158e-02,
        0.13133767840925681614496481e-03,
    ];
    const R: [f64; 9] = [
        0.145589721275038539045668824025e+00,
        -0.273421931495426482902320421863e+00,
        0.226008066916621506788789064272e+00,
        -0.163571895523923805648814425592e+00,
        0.102604312032193978662297299832e+00,
        -0.548023266949835519254211506880e-01,
        0.241432239725390106956523668160e-01,
        -0.822062115403915116036874169600e-02,
        0.180296241564687154310619200000e-02,
    ];
    const AA: [f64; 9] = [
        -0.45894433406309678202825375e-03,
        -0.12281298722544724287816236e-01,
        -0.91144359512342900801764781e-01,
        -0.28412489223839285652511367e-01,
        0.14083827189977123530129812e+01,
        0.11532175281537044570477189e+01,
        -0.72170903389442152112483632e+01,
        -0.19685597805218214001309225e+01,
        0.93846891504541841150916038e+01,
    ];
    const BB: [f64; 12] = [
        1.0,
        0.25136329960926527692263725e+02,
        0.15349442087145759184067981e+03,
        -0.29971215958498680905476402e+03,
        -0.33876477506888115226730368e+04,
        0.28301829314924804988873701e+04,
        0.22979620942196507068034887e+05,
        -0.24280681522998071562462041e+05,
        -0.36680620673264731899504580e+05,
        0.42278731622295627627042436e+05,
        0.28834257644413614344549790e+03,
        0.70226293775648358646587341e+03,
    ];
    const CC: [f64; 9] = [
        -0.7040906288250128001000086e-04,
        -0.3858822461760510359506941e-02,
        -0.7708202127512212359395078e-01,
        -0.6713655014557429480440263e+00,
        -0.2081992124162995545731882e+01,
        0.2898831421475282558867888e+01,
        0.2199509380600429331650192e+02,
        0.2907064664404115316722996e+01,
        -0.4766208741588182425380950e+02,
    ];
    const DD: [f64; 10] = [
        1.0,
        0.5238852785508439144747174e+02,
        0.9646843357714742409535148e+03,
        0.7007152775135939601804416e+04,
        0.8515386792259821780601162e+04,
        -0.1002360095177164564992134e+06,
        -0.2065250031331232815791912e+06,
        0.5695324805290370358175984e+06,
        0.6589752493461331195697873e+06,
        -0.1192930193156561957631462e+07,
    ];
    const E0: f64 = 0.540464821348814822409610122136e+00;
    const E1: f64 = -0.261515522487415653487049835220e-01;
    const E2: f64 = -0.288573438386338758794591212600e-02;
    const E3: f64 = -0.529353396945788057720258856000e-03;
    const S: [f64; 12] = [
        1.0,
        -0.5,
        0.75,
        -1.875,
        6.5625,
        -29.53125,
        162.421875,
        -1055.7421875,
        7918.06640625,
        -67303.564453125,
        639383.8623046875,
        -6713530.55419921875,
    ];
    const S11: f64 = 77205601.373291015625;
    const RPINV: f64 = 0.56418958354775628694807945156077259;

    if x <= 2.0 {
        let u = horner(&P, x);
        let v = horner(&Q, x);
        let t = (x - 3.75) / (x + 3.75);
        // (((((((((U/V)*T + R8)*T + ... + R1)*T + R0
        let mut acc = u / v;
        for &r in R.iter().rev() {
            acc = acc * t + r;
        }
        return acc;
    }
    if x <= 4.0 {
        let z = 1.0 / (2.5 + x * x);
        let u = horner(&AA, z);
        let v = horner(&BB, z);
        let t = 13.0 * z - 1.0;
        // ((((U/V)*T + E2)*T + E1)*T + E0)/X
        let acc = ((u / v * t + E2) * t + E1) * t + E0;
        return acc / x;
    }
    if x < 50.0 {
        let z = 1.0 / (2.5 + x * x);
        let u = horner(&CC, z);
        let v = horner(&DD, z);
        let t = 13.0 * z - 1.0;
        let acc = (((u / v * t + E3) * t + E2) * t + E1) * t + E0;
        return acc / x;
    }
    let t = (1.0 / x) * (1.0 / x);
    let mut z = S11;
    for &s in S.iter().rev() {
        z = z * t + s;
    }
    RPINV * (z / x)
}

/// NSWC DERFC0 tail F: CC/DD + E3..E0 on z = 1/(2.5+x²), published for 4<x<50.
fn nswc_ccdd_f(x: f64) -> f64 {
    const CC: [f64; 9] = [
        -0.7040906288250128001000086e-04,
        -0.3858822461760510359506941e-02,
        -0.7708202127512212359395078e-01,
        -0.6713655014557429480440263e+00,
        -0.2081992124162995545731882e+01,
        0.2898831421475282558867888e+01,
        0.2199509380600429331650192e+02,
        0.2907064664404115316722996e+01,
        -0.4766208741588182425380950e+02,
    ];
    const DD: [f64; 10] = [
        1.0,
        0.5238852785508439144747174e+02,
        0.9646843357714742409535148e+03,
        0.7007152775135939601804416e+04,
        0.8515386792259821780601162e+04,
        -0.1002360095177164564992134e+06,
        -0.2065250031331232815791912e+06,
        0.5695324805290370358175984e+06,
        0.6589752493461331195697873e+06,
        -0.1192930193156561957631462e+07,
    ];
    const E0: f64 = 0.540464821348814822409610122136e+00;
    const E1: f64 = -0.261515522487415653487049835220e-01;
    const E2: f64 = -0.288573438386338758794591212600e-02;
    const E3: f64 = -0.529353396945788057720258856000e-03;
    let z = 1.0 / (2.5 + x * x);
    let u = horner(&CC, z);
    let v = horner(&DD, z);
    let t = 13.0 * z - 1.0;
    let acc = (((u / v * t + E3) * t + E2) * t + E1) * t + E0;
    acc / x
}

// Cephes 2.9 ndtr.c / erfc (netlib cephes/cprob). polevl: coef[0] = highest.
const CEPHES_P: [f64; 9] = [
    2.46196981473530512524e-10,
    5.64189564831068821977e-1,
    7.46321056442269912687e0,
    4.86371970985681366614e1,
    1.96520832956077098242e2,
    5.26445194995477358631e2,
    9.34528527171957607540e2,
    1.02755188689515710272e3,
    5.57535335369399327526e2,
];
const CEPHES_Q: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
const CEPHES_R: [f64; 6] = [
    5.64189583547755073984e-1,
    1.27536670759978104416e0,
    5.01905042251180477414e0,
    6.16021097993053585195e0,
    7.40974269950448939160e0,
    2.97886665372100240670e0,
];
const CEPHES_S: [f64; 6] = [
    2.26052863220117276590e0,
    9.39603524938001434673e0,
    1.20489539808096656605e1,
    1.70814450747565897222e1,
    9.60896809063285878198e0,
    3.36907645100081516050e0,
];
const CEPHES_T: [f64; 5] = [
    9.60497373987051638749e0,
    9.00260197203842689217e1,
    2.23200534594684319226e3,
    7.00332514112805075473e3,
    5.55923013010394962768e4,
];
const CEPHES_U: [f64; 5] = [
    3.35617141647503099647e1,
    5.21357949780152679795e2,
    4.59432382970980127987e3,
    2.26290000613890934246e4,
    4.92673942608635921086e4,
];

fn cephes_polevl(x: f64, coef: &[f64]) -> f64 {
    let mut ans = coef[0];
    for &c in &coef[1..] {
        ans = ans * x + c;
    }
    ans
}
fn cephes_p1evl(x: f64, coef: &[f64]) -> f64 {
    let mut ans = x + coef[0];
    for &c in &coef[1..] {
        ans = ans * x + c;
    }
    ans
}

/// Cephes erfce = exp(x²) erfc(x) for x>1; below 1 uses (1-erf)/w.
fn cephes_f(x: f64) -> f64 {
    let ax = x.abs();
    if ax < 1.0 {
        let z = ax * ax;
        let erf = ax * cephes_polevl(z, &CEPHES_T) / cephes_p1evl(z, &CEPHES_U);
        let w = excel_exp(-z);
        if w == 0.0 {
            return f64::NAN;
        }
        return (1.0 - erf) / w;
    }
    if ax < 8.0 {
        cephes_polevl(ax, &CEPHES_P) / cephes_p1evl(ax, &CEPHES_Q)
    } else {
        cephes_polevl(ax, &CEPHES_R) / cephes_p1evl(ax, &CEPHES_S)
    }
}

fn fdlibm_ix(x: f64) -> u32 {
    ((x.to_bits() >> 32) as u32) & 0x7fffffff
}

/// fdlibm s_erf.c F: for |x|>=1.25, Q = exp(-z²-0.5625)*exp((z-x)(z+x)+R/S)/x
/// with z = x after clearing the low 32 bits. F = Q / excel_exp(-x²).
fn fdlibm_f(x: f64) -> f64 {
    const PP: [f64; 5] = [
        1.28379167095512558561e-01,
        -3.25042107247001499370e-01,
        -2.84817495755985104766e-02,
        -5.77027029648944159157e-03,
        -2.37630166566501626084e-05,
    ];
    const QQ: [f64; 5] = [
        3.97917223959155352819e-01,
        6.50222499887672944485e-02,
        5.08130628187576562776e-03,
        1.32494738004321644526e-04,
        -3.96022827877536812320e-06,
    ];
    const PA: [f64; 7] = [
        -2.36211856075265944077e-03,
        4.14856118683748331666e-01,
        -3.72207876035701323847e-01,
        3.18346619901161753674e-01,
        -1.10894694282396677476e-01,
        3.54783043256182359371e-02,
        -2.16637559486879084300e-03,
    ];
    const QA: [f64; 6] = [
        1.06420880400844228286e-01,
        5.40397917702171048937e-01,
        7.18286544141962662868e-02,
        1.26171219808761642112e-01,
        1.36370839120290507362e-02,
        1.19844998467991074170e-02,
    ];
    const ERX: f64 = 8.45062911510467529297e-01;
    let ax = x.abs();
    let ix = fdlibm_ix(ax);
    if ix < 0x3feb0000 {
        let z = ax * ax;
        let r = PP[0] + z * (PP[1] + z * (PP[2] + z * (PP[3] + z * PP[4])));
        let s = 1.0 + z * (QQ[0] + z * (QQ[1] + z * (QQ[2] + z * (QQ[3] + z * QQ[4]))));
        let y = r / s;
        let erfc = if ix < 0x3fd00000 {
            1.0 - (ax + ax * y)
        } else {
            0.5 - (ax * y + (ax - 0.5))
        };
        let w = excel_exp(-z);
        return if w == 0.0 { f64::NAN } else { erfc / w };
    }
    if ix < 0x3ff40000 {
        let s = ax - 1.0;
        let p = PA[0]
            + s * (PA[1]
                + s * (PA[2] + s * (PA[3] + s * (PA[4] + s * (PA[5] + s * PA[6])))));
        let q = 1.0
            + s * (QA[0] + s * (QA[1] + s * (QA[2] + s * (QA[3] + s * (QA[4] + s * QA[5])))));
        let erfc = (1.0 - ERX) - p / q;
        let w = excel_exp(-(ax * ax));
        return if w == 0.0 { f64::NAN } else { erfc / w };
    }
    if ix >= 0x403c0000 {
        return 0.0;
    }
    let s = 1.0 / (ax * ax);
    let (rr, ss) = if ix < 0x4006db6d {
        const RA: [f64; 8] = [
            -9.86494403484714822705e-03,
            -6.93858572707181764372e-01,
            -1.05586262253232909814e+01,
            -6.23753324503260060396e+01,
            -1.62396669462573470355e+02,
            -1.84605092906711035994e+02,
            -8.12874355063065934246e+01,
            -9.81432934416914548592e+00,
        ];
        const SA: [f64; 8] = [
            1.96512716674392571292e+01,
            1.37657754143519042600e+02,
            4.34565877475229228821e+02,
            6.45387271733267880336e+02,
            4.29008140027567833386e+02,
            1.08635005541779435134e+02,
            6.57024977031928170135e+00,
            -6.04244152148580987438e-02,
        ];
        let r = RA[0]
            + s * (RA[1]
                + s * (RA[2]
                    + s * (RA[3] + s * (RA[4] + s * (RA[5] + s * (RA[6] + s * RA[7]))))));
        let st = 1.0
            + s * (SA[0]
                + s * (SA[1]
                    + s * (SA[2]
                        + s * (SA[3] + s * (SA[4] + s * (SA[5] + s * (SA[6] + s * SA[7])))))));
        (r, st)
    } else {
        const RB: [f64; 7] = [
            -9.86494292470009928597e-03,
            -7.99283237680523006574e-01,
            -1.77579549177547519889e+01,
            -1.60636384855821916062e+02,
            -6.37566443368389627722e+02,
            -1.02509513161107724954e+03,
            -4.83519191608651397019e+02,
        ];
        const SB: [f64; 7] = [
            3.03380607434824582924e+01,
            3.25792512996573918826e+02,
            1.53672958608443695994e+03,
            3.19985821950859553908e+03,
            2.55305040643316442583e+03,
            4.74528541206955367215e+02,
            -2.24409524465858183362e+01,
        ];
        let r = RB[0]
            + s * (RB[1] + s * (RB[2] + s * (RB[3] + s * (RB[4] + s * (RB[5] + s * RB[6])))));
        let st = 1.0
            + s * (SB[0]
                + s * (SB[1] + s * (SB[2] + s * (SB[3] + s * (SB[4] + s * (SB[5] + s * SB[6]))))));
        (r, st)
    };
    let z = f64::from_bits(ax.to_bits() & 0xffff_ffff_0000_0000);
    let q = excel_exp(-z * z - 0.5625) * excel_exp((z - ax) * (z + ax) + rr / ss) / ax;
    let w = excel_exp(-(ax * ax));
    if w == 0.0 {
        f64::NAN
    } else {
        q / w
    }
}

/// A&S 7.1.14: F = 1/(x√π) * 1/(1 + (1/2)/z² / (1 + 1/z² / (1 + (3/2)/z² / …)))
fn cf_as714_f(x: f64) -> f64 {
    let a_scale = 0.5 / (x * x);
    let mut den = 1.0;
    for n in (1..=80).rev() {
        den = 1.0 + (n as f64) * a_scale / den;
    }
    0.56418958354775628695 / x / den
}

/// Gautschi CF: F = 1 / (√π (x + 1/2 / (x + 1 / (x + 3/2 / (x + …)))))
fn cf_gautschi_f(x: f64) -> f64 {
    let mut f = x;
    for n in (1..=80).rev() {
        f = x + (n as f64) * 0.5 / f;
    }
    0.56418958354775628695 / f
}

fn nswc_erfc(z: f64, exp_fn: fn(f64) -> f64) -> f64 {
    let y = z.abs();
    let q = if y <= 1.0 {
        nswc_small_erfc(y)
    } else if y > 100.0 {
        0.0
    } else {
        flush(exp_fn(-(y * y)) * nswc_derfc0(y))
    };
    flush(if z < 0.0 { 2.0 - q } else { q })
}

fn libm_erfc(z: f64) -> f64 {
    flush(libm::erfc(z))
}

fn production_erfc(z: f64) -> f64 {
    flush(erfc_precise_kernel(z).unwrap_or(f64::NAN))
}

fn host_exp(x: f64) -> f64 {
    x.exp()
}

#[derive(Clone, Copy)]
struct Graph {
    name: &'static str,
    eval: fn(f64) -> f64,
}

const GRAPHS: [Graph; 7] = [
    Graph {
        name: "libm::erfc",
        eval: libm_erfc,
    },
    Graph {
        name: "production_excel_erfc",
        eval: production_erfc,
    },
    Graph {
        name: "cody_split_hostexp",
        eval: cody_erfc_split,
    },
    Graph {
        name: "cody_unsplit_hostexp",
        eval: cody_unsplit_host,
    },
    Graph {
        name: "cody_unsplit_excelexp",
        eval: cody_unsplit_excel,
    },
    Graph {
        name: "nswc_hostexp",
        eval: nswc_host,
    },
    Graph {
        name: "nswc_excelexp",
        eval: nswc_excel,
    },
];

fn cody_unsplit_host(z: f64) -> f64 {
    cody_erfc_unsplit(z, host_exp)
}
fn cody_unsplit_excel(z: f64) -> f64 {
    cody_erfc_unsplit(z, excel_exp)
}
fn nswc_host(z: f64) -> f64 {
    nswc_erfc(z, host_exp)
}
fn nswc_excel(z: f64) -> f64 {
    nswc_erfc(z, excel_exp)
}

#[derive(Clone, Copy, Debug)]
enum Band {
    Small,
    Mid,
    Tail,
}

fn band(z: f64) -> Band {
    if z < 0.5 {
        Band::Small
    } else if z < 4.0 {
        Band::Mid
    } else {
        Band::Tail
    }
}

struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    sum_ulp: u128,
}

impl Acc {
    fn new() -> Self {
        Self {
            exact: 0,
            n: 0,
            max_ulp: 0,
            sum_ulp: 0,
        }
    }
    fn add(&mut self, d: u64) {
        self.n += 1;
        if d == 0 {
            self.exact += 1;
        } else {
            self.max_ulp = self.max_ulp.max(d);
            self.sum_ulp += d as u128;
        }
    }
}

fn insert_row(rows: &mut BTreeMap<u64, u64>, z: f64, q_bits: u64, src: &str) {
    if !z.is_finite() || z < 0.0 {
        return;
    }
    let key = z.to_bits();
    if let Some(&old) = rows.get(&key) {
        if old != q_bits {
            eprintln!("conflict at z={z} ({src}): {old:#x} vs {q_bits:#x}");
        }
    } else {
        rows.insert(key, q_bits);
    }
}

fn load_erfc_banks(dir: &str) -> BTreeMap<u64, u64> {
    let mut rows = BTreeMap::new();
    for name in ERFC_BANKS {
        let path = format!("{dir}/{name}");
        let Ok(text) = std::fs::read_to_string(&path) else {
            eprintln!("skip missing {path}");
            continue;
        };
        let bank: WitnessSet =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
        for w in &bank.witnesses {
            let z = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("z bits"),
                _ => continue,
            };
            let Some(q) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            insert_row(&mut rows, z, q.to_bits(), name);
        }
        eprintln!("loaded {name}: {} witnesses", bank.witnesses.len());
    }
    rows
}

fn load_normref_implied_q(dir: &str, rows: &mut BTreeMap<u64, u64>) {
    let path = format!("{dir}/answers-b24-normref.json");
    let Ok(text) = std::fs::read_to_string(&path) else {
        eprintln!("skip missing {path}");
        return;
    };
    let bank: WitnessSet = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut n = 0usize;
    for w in &bank.witnesses {
        if w.args.len() < 2 {
            continue;
        }
        let x = match &w.args[0] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).expect("x bits"),
            _ => continue,
        };
        let cum = match &w.args[1] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).expect("cum bits"),
            _ => continue,
        };
        if cum != 1.0 || !x.is_sign_negative() {
            continue;
        }
        let Some(ns) = parse_bits_hex(&w.expected_bits) else {
            continue;
        };
        let q = ns * 2.0;
        let z = x.abs() * FRAC_1_SQRT_2;
        insert_row(rows, z, q.to_bits(), "b24-normref");
        n += 1;
    }
    eprintln!("loaded answers-b24-normref.json: {n} implied-Q rows");
}

fn score_graph(rows: &[(f64, u64)], eval: fn(f64) -> f64) -> ([Acc; 3], Acc) {
    let mut bands = [Acc::new(), Acc::new(), Acc::new()];
    let mut all = Acc::new();
    for &(z, expected) in rows {
        let got = eval(z);
        let d = ulp_distance(got, f64::from_bits(expected)).unwrap_or(u64::MAX);
        all.add(d);
        match band(z) {
            Band::Small => bands[0].add(d),
            Band::Mid => bands[1].add(d),
            Band::Tail => bands[2].add(d),
        }
    }
    (bands, all)
}

fn fmt_acc(a: &Acc) -> String {
    if a.n == 0 {
        return "—".into();
    }
    format!("{}/{} max={} sum={}", a.exact, a.n, a.max_ulp, a.sum_ulp)
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("G3-01-dist directory (heldout files are not named)");
    assert!(!dir.contains("heldout"), "heldout path refused");

    let mut map = load_erfc_banks(&dir);
    let direct_n = map.len();
    load_normref_implied_q(&dir, &mut map);
    let rows: Vec<(f64, u64)> = map
        .into_iter()
        .map(|(zb, qb)| (f64::from_bits(zb), qb))
        .collect();
    println!(
        "distinct nonnegative z rows: {} ({} direct ERFC + implied-Q merge); heldout absent",
        rows.len(),
        direct_n
    );

    println!();
    println!(
        "{:<24} {:>22} {:>22} {:>22} {:>22}",
        "graph", "small z<0.5", "mid [0.5,4)", "tail z>=4", "all"
    );
    let mut ranked: Vec<(usize, usize, &'static str, [Acc; 3], Acc)> = Vec::new();
    for g in GRAPHS {
        let (bands, all) = score_graph(&rows, g.eval);
        println!(
            "{:<24} {:>22} {:>22} {:>22} {:>22}",
            g.name,
            fmt_acc(&bands[0]),
            fmt_acc(&bands[1]),
            fmt_acc(&bands[2]),
            fmt_acc(&all)
        );
        ranked.push((all.exact, all.n, g.name, bands, all));
    }
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then(a.4.max_ulp.cmp(&b.4.max_ulp)));
    println!();
    println!(
        "rank by exact: {}",
        ranked.iter().map(|r| r.2).collect::<Vec<_>>().join(" > ")
    );

    println!();
    println!("F-body pin witnesses (Excel Q vs each graph; F-rel in units of 2^-53):");
    println!(
        "{:<10} {:<24} {:>16} {:>10} {:>12}",
        "z", "graph", "Q bits", "Q ulp", "F rel"
    );
    for &z in &PIN_Z {
        let Some((_, expected)) = rows.iter().find(|(zz, _)| *zz == z) else {
            println!("{z:<10} (no frozen Excel Q at this exact z)");
            continue;
        };
        let q_ex = f64::from_bits(*expected);
        let w = excel_exp(-(z * z));
        let f_ex = if w != 0.0 { q_ex / w } else { f64::NAN };
        for g in GRAPHS {
            let got = (g.eval)(z);
            let d = ulp_distance(got, q_ex).unwrap_or(u64::MAX);
            let f_got = if w != 0.0 { got / w } else { f64::NAN };
            let f_rel = if f_ex.is_finite() && f_ex != 0.0 && f_got.is_finite() {
                (f_got - f_ex) / f_ex * (1u64 << 53) as f64
            } else {
                f64::NAN
            };
            println!(
                "{z:<10} {:<24} {:>16x} {:>10} {:>12.3}",
                g.name,
                got.to_bits(),
                d,
                f_rel
            );
        }
    }

    let z_one = FRAC_1_SQRT_2;
    if let Some((_, qbits)) = rows.iter().find(|(zz, _)| zz.to_bits() == z_one.to_bits()) {
        println!();
        println!(
            "Q at z=RN(1/sqrt(2)): excel={:#x} (CHIDIST(1,1) witness is 0x3fd44ed0bb7cb209)",
            qbits
        );
    }

    println!();
    println!("worst 8 rows, nswc_excelexp:");
    let mut worst: Vec<(u64, f64, u64, u64)> = rows
        .iter()
        .map(|&(z, expected)| {
            let got = nswc_excel(z);
            let d = ulp_distance(got, f64::from_bits(expected)).unwrap_or(u64::MAX);
            (d, z, got.to_bits(), expected)
        })
        .collect();
    worst.sort_by(|a, b| b.0.cmp(&a.0));
    for (d, z, got, exp) in worst.iter().take(8) {
        println!("  ulp={d} z={z} got={got:#x} excel={exp:#x}");
    }

    println!();
    println!("Cody C/D ±1 ULP scan on mid-band [0.5,4) vs unsplit excel_exp baseline:");
    let mid: Vec<(f64, u64)> = rows
        .iter()
        .copied()
        .filter(|(z, _)| *z >= 0.5 && *z < 4.0)
        .collect();
    let base = score_mid(&mid, &CODY_C, &CODY_D);
    println!("  baseline mid exact={}/{}", base, mid.len());
    let mut c = CODY_C;
    for i in 0..9 {
        for dir in [-1i32, 1] {
            c[i] = f64::from_bits(CODY_C[i].to_bits().wrapping_add_signed(dir as i64));
            let got = score_mid(&mid, &c, &CODY_D);
            let delta = got as i64 - base as i64;
            if delta != 0 {
                println!("  C[{i}] {dir:+}: exact={got} delta={delta:+}");
            }
            c[i] = CODY_C[i];
        }
    }
    let mut d = CODY_D;
    for i in 0..8 {
        for dir in [-1i32, 1] {
            d[i] = f64::from_bits(CODY_D[i].to_bits().wrapping_add_signed(dir as i64));
            let got = score_mid(&mid, &CODY_C, &d);
            let delta = got as i64 - base as i64;
            if delta != 0 {
                println!("  D[{i}] {dir:+}: exact={got} delta={delta:+}");
            }
            d[i] = CODY_D[i];
        }
    }

    println!();
    println!("implied-F form race (z>=0.5): F_or = Q / w");
    println!("  w_native = excel_exp(-(z*z))           // today's table, SSE z^2");
    println!("  w_rn53   = excel_exp(-RN53(RN64(z*z))) // identified unsplit arg (x87-DR square)");
    println!("  w_split  = excel_exp(-z)*excel_exp(-z) // split-exp control, should lose");
    let f_graphs: [(&str, fn(f64) -> f64); 7] = [
        ("nswc_derfc0", nswc_derfc0),
        ("nswc_ccdd", nswc_ccdd_f),
        ("cody_erfcx", cody_erfcx_large),
        ("cephes_erfce", cephes_f),
        ("fdlibm_F", fdlibm_f),
        ("cf_as714", cf_as714_f),
        ("cf_gautschi", cf_gautschi_f),
    ];
    let oracles: [(&str, fn(f64) -> f64); 3] = [
        ("w_native", |z| excel_exp(-(z * z))),
        ("w_rn53", |z| excel_exp(-x87_mul(z, z))),
        ("w_split", |z| excel_exp(-z) * excel_exp(-z)),
    ];
    for (w_name, w_fn) in oracles {
        println!();
        println!("oracle {w_name}");
        println!(
            "{:<24} {:>22} {:>22}",
            "F graph", "mid [0.5,4)", "tail z>=4"
        );
        for (name, eval) in f_graphs {
            let mut mid_a = Acc::new();
            let mut tail_a = Acc::new();
            for &(z, qbits) in &rows {
                if z < 0.5 {
                    continue;
                }
                let w = w_fn(z);
                if w == 0.0 || !w.is_finite() {
                    continue;
                }
                let f_or = f64::from_bits(qbits) / w;
                let fg = eval(z);
                if !f_or.is_finite() || !fg.is_finite() {
                    continue;
                }
                let d = ulp_distance(fg, f_or).unwrap_or(u64::MAX);
                if d > (1u64 << 20) {
                    continue;
                }
                if z < 4.0 {
                    mid_a.add(d);
                } else {
                    tail_a.add(d);
                }
            }
            println!(
                "{:<24} {:>22} {:>22}",
                name,
                fmt_acc(&mid_a),
                fmt_acc(&tail_a)
            );
        }
        println!("  pins F ulp vs this F_or:");
        for &z in &PIN_Z {
            let Some((_, expected)) = rows.iter().find(|(zz, _)| *zz == z) else {
                continue;
            };
            let w = w_fn(z);
            if w == 0.0 || !w.is_finite() {
                println!("    z={z}: w=0/nonfinite");
                continue;
            }
            let f_or = f64::from_bits(*expected) / w;
            print!("    z={z}:");
            for (name, eval) in f_graphs {
                let d = ulp_distance(eval(z), f_or).unwrap_or(u64::MAX);
                print!(" {name}={d}");
            }
            println!();
        }
    }

    println!();
    println!("F_or disagreement (z>=0.5, finite w):");
    let mut n = 0usize;
    let mut n_nat_rn = 0usize;
    let mut n_nat_sp = 0usize;
    let mut max_nat_rn = 0u64;
    let mut max_nat_sp = 0u64;
    for &(z, qbits) in &rows {
        if z < 0.5 {
            continue;
        }
        let wn = excel_exp(-(z * z));
        let wr = excel_exp(-x87_mul(z, z));
        let ws = excel_exp(-z) * excel_exp(-z);
        if wn == 0.0 || wr == 0.0 || !wn.is_finite() || !wr.is_finite() {
            continue;
        }
        let q = f64::from_bits(qbits);
        let fnat = q / wn;
        let frn = q / wr;
        n += 1;
        if fnat.to_bits() != frn.to_bits() {
            n_nat_rn += 1;
            if let Some(d) = ulp_distance(fnat, frn) {
                max_nat_rn = max_nat_rn.max(d);
            }
        }
        if ws != 0.0 && ws.is_finite() {
            let fsp = q / ws;
            if fnat.to_bits() != fsp.to_bits() {
                n_nat_sp += 1;
                if let Some(d) = ulp_distance(fnat, fsp) {
                    max_nat_sp = max_nat_sp.max(d);
                }
            }
        }
    }
    println!("  rows compared: {n}");
    println!("  native vs rn53  F_or bits differ: {n_nat_rn}  max ULP={max_nat_rn}");
    println!("  native vs split F_or bits differ: {n_nat_sp}  max ULP={max_nat_sp}");
}

fn score_mid(mid: &[(f64, u64)], c: &[f64; 9], d: &[f64; 8]) -> usize {
    let mut exact = 0usize;
    for &(z, expected) in mid {
        let y = z.abs();
        let f = if y <= 4.0 {
            let mut xnum = c[8] * y;
            let mut xden = y;
            for i in 0..7 {
                xnum = (xnum + c[i]) * y;
                xden = (xden + d[i]) * y;
            }
            (xnum + c[7]) / (xden + d[7])
        } else {
            cody_erfcx_large(y)
        };
        let q = flush(excel_exp(-(y * y)) * f);
        if q.to_bits() == expected {
            exact += 1;
        }
    }
    exact
}
