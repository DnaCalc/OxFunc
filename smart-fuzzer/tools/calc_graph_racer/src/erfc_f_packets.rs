//! Public ERFC F-packets and implied-F oracle (`w_rn53`).
//! Heldouts unnamed. Does not land kernels.

use crate::eval::parse_bits_hex;
use crate::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research::{excel_exp, x87_mul};
use std::collections::BTreeMap;
use std::fs;

pub const FRAC_1_SQRT_2: f64 = f64::from_bits(0x3fe6a09e667f3bcd);
pub const PIN_Z: [f64; 5] = [0.75, 1.28125, 1.875, 2.125, 5.0];
pub const RPINV: f64 = 0.56418958354775628695;
const ERFC_BANKS: [&str; 5] = [
    "answers-erfcp.json",
    "answers-erfcm.json",
    "answers-b7erfc.json",
    "answers-b8erfc.json",
    "answers-b11c.json",
];
const ULP_CAP: u64 = 1 << 20;

pub fn horner(cs: &[f64], x: f64) -> f64 {
    let mut acc = 0.0;
    for &c in cs.iter().rev() {
        acc = acc * x + c;
    }
    acc
}

pub fn w_rn53(z: f64) -> f64 {
    excel_exp(-x87_mul(z, z))
}

pub fn f_or(z: f64, qbits: u64) -> Option<f64> {
    let w = w_rn53(z);
    if w == 0.0 || !w.is_finite() {
        return None;
    }
    let f = f64::from_bits(qbits) / w;
    if f.is_finite() { Some(f) } else { None }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Acc {
    pub exact: usize,
    pub n: usize,
    pub max_ulp: u64,
    pub sum_ulp: u128,
}

impl Acc {
    pub fn add(&mut self, d: u64) {
        self.n += 1;
        if d == 0 {
            self.exact += 1;
        } else {
            self.max_ulp = self.max_ulp.max(d);
            self.sum_ulp += d as u128;
        }
    }
}

pub fn fmt_acc(a: &Acc) -> String {
    if a.n == 0 {
        return "—".into();
    }
    format!("{}/{} max={} sum={}", a.exact, a.n, a.max_ulp, a.sum_ulp)
}

pub fn score_f(rows: &[(f64, u64)], eval: impl Fn(f64) -> f64) -> (Acc, Acc) {
    let mut mid = Acc::default();
    let mut tail = Acc::default();
    for &(z, qbits) in rows {
        if z < 0.5 {
            continue;
        }
        let Some(fo) = f_or(z, qbits) else {
            continue;
        };
        let fg = eval(z);
        if !fg.is_finite() {
            continue;
        }
        let d = ulp_distance(fg, fo).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        if z < 4.0 {
            mid.add(d);
        } else {
            tail.add(d);
        }
    }
    (mid, tail)
}

pub fn load_q_rows(dir: &str) -> Vec<(f64, u64)> {
    let mut rows = BTreeMap::new();
    for name in ERFC_BANKS {
        assert!(!name.contains("heldout"));
        let path = format!("{dir}/{name}");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
        for w in &bank.witnesses {
            let z = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("z"),
                _ => continue,
            };
            let Some(q) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if z.is_finite() && z >= 0.0 {
                rows.entry(z.to_bits()).or_insert(q.to_bits());
            }
        }
    }
    let path = format!("{dir}/answers-b24-normref.json");
    if let Ok(text) = fs::read_to_string(&path) {
        let bank: WitnessSet = serde_json::from_str(&text).expect("normref");
        for w in &bank.witnesses {
            if w.args.len() < 2 {
                continue;
            }
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("x"),
                _ => continue,
            };
            let cum = match &w.args[1] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("cum"),
                _ => continue,
            };
            if cum != 1.0 || !x.is_sign_negative() {
                continue;
            }
            let Some(ns) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            let z = x.abs() * FRAC_1_SQRT_2;
            if z.is_finite() && z >= 0.0 {
                rows.entry(z.to_bits()).or_insert((ns * 2.0).to_bits());
            }
        }
    }
    rows.into_iter()
        .map(|(zb, qb)| (f64::from_bits(zb), qb))
        .collect()
}

pub fn nswc_pqr_f(x: f64) -> f64 {
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
        0.145589721275038539045668824025,
        -0.273421931495426482902320421863,
        0.226008066916621506788789064272,
        -0.163571895523923805648814425592,
        0.102604312032193978662297299832,
        -0.548023266949835519254211506880e-01,
        0.241432239725390106956523668160e-01,
        -0.822062115403915116036874169600e-02,
        0.180296241564687154310619200000e-02,
    ];
    let u = horner(&P, x);
    let v = horner(&Q, x);
    let t = (x - 3.75) / (x + 3.75);
    let mut acc = u / v;
    for &r in R.iter().rev() {
        acc = acc * t + r;
    }
    acc
}

pub fn nswc_pqr_t(x: f64, t: f64) -> f64 {
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
        0.145589721275038539045668824025,
        -0.273421931495426482902320421863,
        0.226008066916621506788789064272,
        -0.163571895523923805648814425592,
        0.102604312032193978662297299832,
        -0.548023266949835519254211506880e-01,
        0.241432239725390106956523668160e-01,
        -0.822062115403915116036874169600e-02,
        0.180296241564687154310619200000e-02,
    ];
    let u = horner(&P, x);
    let v = horner(&Q, x);
    let mut acc = u / v;
    for &r in R.iter().rev() {
        acc = acc * t + r;
    }
    acc
}

pub fn nswc_derfc0(x: f64) -> f64 {
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
    const E0: f64 = 0.540464821348814822409610122136;
    const E1: f64 = -0.261515522487415653487049835220e-01;
    const E2: f64 = -0.288573438386338758794591212600e-02;
    if x <= 2.0 {
        return nswc_pqr_f(x);
    }
    if x <= 4.0 {
        let z = 1.0 / (2.5 + x * x);
        let t = 13.0 * z - 1.0;
        let acc = ((horner(&AA, z) / horner(&BB, z) * t + E2) * t + E1) * t + E0;
        return acc / x;
    }
    nswc_ccdd_f(x)
}

pub fn nswc_ccdd_f(x: f64) -> f64 {
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
    const E0: f64 = 0.540464821348814822409610122136;
    const E1: f64 = -0.261515522487415653487049835220e-01;
    const E2: f64 = -0.288573438386338758794591212600e-02;
    const E3: f64 = -0.529353396945788057720258856000e-03;
    let z = 1.0 / (2.5 + x * x);
    let t = 13.0 * z - 1.0;
    let acc = (((horner(&CC, z) / horner(&DD, z) * t + E3) * t + E2) * t + E1) * t + E0;
    acc / x
}

pub fn cody_erfcx_f(y: f64) -> f64 {
    const C: [f64; 9] = [
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
    const D: [f64; 8] = [
        15.7449261107098347,
        117.693950891312499,
        537.181101862009858,
        1621.38957456669019,
        3290.79923573345963,
        4362.61909014324716,
        3439.36767414372164,
        1230.33935480374942,
    ];
    const P: [f64; 6] = [
        0.305326634961232344,
        0.360344899949804439,
        0.125781726111229246,
        0.0160837851487422766,
        6.58749161529837803e-4,
        0.0163153871373020978,
    ];
    const Q: [f64; 5] = [
        2.56852019228982242,
        1.87295284992346047,
        0.527905102951428412,
        0.0605183413124413191,
        0.00233520497626869185,
    ];
    if y <= 4.0 {
        let mut xnum = C[8] * y;
        let mut xden = y;
        for i in 0..7 {
            xnum = (xnum + C[i]) * y;
            xden = (xden + D[i]) * y;
        }
        (xnum + C[7]) / (xden + D[7])
    } else {
        let ysq = 1.0 / (y * y);
        let mut xnum = P[5] * ysq;
        let mut xden = ysq;
        for i in 0..4 {
            xnum = (xnum + P[i]) * ysq;
            xden = (xden + Q[i]) * ysq;
        }
        let r = ysq * (xnum + P[4]) / (xden + Q[4]);
        (RPINV - r) / y
    }
}

fn polevl(x: f64, coef: &[f64]) -> f64 {
    let mut ans = coef[0];
    for &c in &coef[1..] {
        ans = ans * x + c;
    }
    ans
}
fn p1evl(x: f64, coef: &[f64]) -> f64 {
    let mut ans = x + coef[0];
    for &c in &coef[1..] {
        ans = ans * x + c;
    }
    ans
}

pub fn cephes_f(x: f64) -> f64 {
    const P: [f64; 9] = [
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
    const Q: [f64; 8] = [
        1.32281951154744992508e1,
        8.67072140885989742329e1,
        3.54937778887819891062e2,
        9.75708501743205489753e2,
        1.82390916687909736289e3,
        2.24633760818710981792e3,
        1.65666309194161350182e3,
        5.57535340817727675546e2,
    ];
    const R: [f64; 6] = [
        5.64189583547755073984e-1,
        1.27536670759978104416e0,
        5.01905042251180477414e0,
        6.16021097993053585195e0,
        7.40974269950448939160e0,
        2.97886665372100240670e0,
    ];
    const S: [f64; 6] = [
        2.26052863220117276590e0,
        9.39603524938001434673e0,
        1.20489539808096656605e1,
        1.70814450747565897222e1,
        9.60896809063285878198e0,
        3.36907645100081516050e0,
    ];
    const T: [f64; 5] = [
        9.60497373987051638749e0,
        9.00260197203842689217e1,
        2.23200534594684319226e3,
        7.00332514112805075473e3,
        5.55923013010394962768e4,
    ];
    const U: [f64; 5] = [
        3.35617141647503099647e1,
        5.21357949780152679795e2,
        4.59432382970980127987e3,
        2.26290000613890934246e4,
        4.92673942608635921086e4,
    ];
    let ax = x.abs();
    if ax < 1.0 {
        let z = ax * ax;
        let erf = ax * polevl(z, &T) / p1evl(z, &U);
        let w = w_rn53(ax);
        if w == 0.0 {
            return f64::NAN;
        }
        return (1.0 - erf) / w;
    }
    if ax < 8.0 {
        polevl(ax, &P) / p1evl(ax, &Q)
    } else {
        polevl(ax, &R) / p1evl(ax, &S)
    }
}

pub fn cdflib_erfc1_f(x: f64) -> f64 {
    const P: [f64; 8] = [
        -1.36864857382717e-7,
        5.64195517478974e-1,
        7.21175825088309e0,
        4.31622272220567e1,
        1.52989285046940e2,
        3.39320816734344e2,
        4.51918953711873e2,
        3.00459261020162e2,
    ];
    const Q: [f64; 8] = [
        1.0,
        1.27827273196294e1,
        7.70001529352295e1,
        2.77585444743988e2,
        6.38980264465631e2,
        9.31354094850610e2,
        7.90950925327898e2,
        3.00459260956983e2,
    ];
    const R: [f64; 5] = [
        2.10144126479064e0,
        2.62370141675169e1,
        2.13688200555087e1,
        4.65807828718470e0,
        2.82094791773523e-1,
    ];
    const S: [f64; 4] = [
        9.41537750555460e1,
        1.87114811799590e2,
        9.90191814623914e1,
        1.80124575948747e1,
    ];
    let ax = x.abs();
    if ax <= 0.5 {
        return f64::NAN;
    }
    if ax <= 4.0 {
        return polevl(ax, &P) / polevl(ax, &Q);
    }
    let t = 1.0 / (ax * ax);
    let top = (((R[0] * t + R[1]) * t + R[2]) * t + R[3]) * t + R[4];
    let bot = (((S[0] * t + S[1]) * t + S[2]) * t + S[3]) * t + 1.0;
    (RPINV - t * top / bot) / ax
}

pub fn cf_as714_n(x: f64, nterms: u32) -> f64 {
    let a_scale = 0.5 / (x * x);
    let mut den = 1.0;
    for n in (1..=nterms).rev() {
        den = 1.0 + (n as f64) * a_scale / den;
    }
    RPINV / x / den
}

pub fn cf_gautschi_n(x: f64, nterms: u32) -> f64 {
    let mut f = x;
    for n in (1..=nterms).rev() {
        f = x + (n as f64) * 0.5 / f;
    }
    RPINV / f
}

pub fn cf_as714_f(x: f64) -> f64 {
    cf_as714_n(x, 80)
}
pub fn cf_gautschi_f(x: f64) -> f64 {
    cf_gautschi_n(x, 80)
}
