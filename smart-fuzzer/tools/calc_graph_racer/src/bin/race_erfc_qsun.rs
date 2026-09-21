//! Old 4.3BSD/SunOS erf.c (McIlroy 1992) and Numerical Recipes erfc as Q=w*F.
//! Distinct from fdlibm RA/SA. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{excel_exp, ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const LSQRTPI_HI: f64 = 0.5723649429247000819387380943226;

const RB: [f64; 11] = [
    -1.5306508387410807582e-10,
    2.15592846101742183841910806188e-8,
    6.24998557732436510470108714799e-1,
    8.24849222231141787631258921465,
    2.63974967372233173534823436057e1,
    9.86383092541570505318304640241,
    -7.28024154841991322228977878694,
    5.96303287280680116566600190708,
    -4.40070358507372993983608466806,
    2.39923700182518073731330332521,
    -6.89257464785841156285073338950e-1,
];
const SB: [f64; 4] = [
    1.0,
    1.56641558965626774835300238919e1,
    7.20522741000949622502957936376e1,
    9.60121069770492994166488642804e1,
];
const RC: [f64; 11] = [
    -2.47925334685189288817e-7,
    1.28735722546372485255126993930e-5,
    6.24664954087883916855616917019e-1,
    4.69798884785807402408863708843,
    7.61618295853929705430118701770,
    9.15640208659364240872946538730e-1,
    -3.59753040425048631334448145935e-1,
    1.42862267989304403403849619281e-1,
    -4.74392758811439801958087514322e-2,
    1.09964787987580810135757047874e-2,
    -1.28856240494889325194638463046e-3,
];
const SC: [f64; 4] = [
    1.0,
    9.97395106984001955652274773456,
    2.80952153365721279953959310660e1,
    2.19826478142545234106819407316e1,
];
const RD: [f64; 14] = [
    -2.1491361969012978677e-16,
    -4.99999999999640086151350330820e-1,
    6.24999999772906433825880867516e-1,
    -1.54166659428052432723177389562,
    5.51561147405411844601985649206,
    -2.55046307982949826964613748714e1,
    1.43631424382843846387913799845e2,
    -9.45789244999420134263345971704e2,
    6.94834146607051206956384703517e3,
    -5.27176414235983393155038356781e4,
    3.68530281128672766499221324921e5,
    -2.06466642800404317677021026611e6,
    7.78293889471135381609201431274e6,
    -1.42821001129434127360582351685e7,
];

const NR: [f64; 10] = [
    1.26551223,
    1.00002368,
    0.37409196,
    0.09678418,
    -0.18628806,
    0.27886807,
    -1.13520398,
    1.48851587,
    -0.82215223,
    0.17087277,
];

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

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn horner(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
    }
    acc
}
fn cody8(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(C[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW),
        CW,
    )
}
fn sun_corr(x: f64) -> f64 {
    let y = x.abs();
    let ye = ef(y);
    let s = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
    let (r, den, extra) = if y < 2.0 {
        (
            horner(&RC, s),
            horner(&SC, s),
            ext_mul(&ef(-0.5), &s, CW),
        )
    } else if y < 4.0 {
        (
            horner(&RB, s),
            horner(&SB, s),
            ext_mul(&ef(-0.5), &s, CW),
        )
    } else {
        let r = ext_mul(&s, &horner(&RD[1..], s), CW);
        return {
            let ycorr = ext_add(&r, &ef(RD[0]), CW);
            let ycorr = ext_add(&ycorr, &ef(-LSQRTPI_HI), CW);
            ext_to_f64(&ycorr, CW)
        };
    };
    let ycorr = ext_add(
        &ext_add(&ext_div(&r, &den, CW), &extra, CW),
        &ef(-LSQRTPI_HI),
        CW,
    );
    ext_to_f64(&ycorr, CW)
}
fn sun_f(x: f64) -> f64 {
    excel_exp(sun_corr(x)) / x.abs()
}
fn sun_f_libm(x: f64) -> f64 {
    libm::exp(sun_corr(x)) / x.abs()
}
fn nr_f(z: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.5 * z);
    let mut tau = NR[9];
    for &c in NR[1..9].iter().rev() {
        tau = tau * t + c;
    }
    tau = tau * t - NR[0];
    t * excel_exp(tau)
}
fn nr_f_libm(z: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.5 * z);
    let mut tau = NR[9];
    for &c in NR[1..9].iter().rev() {
        tau = tau * t + c;
    }
    tau = tau * t - NR[0];
    t * libm::exp(tau)
}
fn nr_f_x87(z: f64) -> f64 {
    let ze = ef(z);
    let t = ext_div(&ef(1.0), &ext_add(&ef(1.0), &ext_mul(&ef(0.5), &ze, CW), CW), CW);
    let mut tau = ef(NR[9]);
    for &c in NR[1..9].iter().rev() {
        tau = ext_add(&ext_mul(&tau, &t, CW), &ef(c), CW);
    }
    tau = ext_add(&ext_mul(&tau, &t, CW), &ef(-NR[0]), CW);
    let tau64 = ext_to_f64(&tau, CW);
    ext_to_f64(&ext_mul(&t, &ef(excel_exp(tau64)), CW), CW)
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

#[derive(Default)]
struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    dmid: usize,
    dn: usize,
}
impl Acc {
    fn add(&mut self, d: u64, direct: bool) {
        self.n += 1;
        if direct {
            self.dn += 1;
        }
        if d == 0 {
            self.exact += 1;
            if direct {
                self.dmid += 1;
            }
        } else {
            self.max_ulp = self.max_ulp.max(d);
        }
    }
}
fn fmt(a: &Acc) -> String {
    format!("{}/{} max={} d={}/{}", a.exact, a.n, a.max_ulp, a.dmid, a.dn)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let graphs: [(&str, fn(f64) -> f64); 8] = [
        ("SPECFUN C/D x87", |z| qw(z, cody8(z))),
        ("SunBSD F excel_exp corr", |z| qw(z, sun_f(z))),
        ("SunBSD F libm exp corr", |z| qw(z, sun_f_libm(z))),
        ("Cody z<1.25 else SunBSD", |z| {
            if z < 1.25 {
                qw(z, cody8(z))
            } else {
                qw(z, sun_f(z))
            }
        }),
        ("Cody z<2 else SunBSD", |z| {
            if z < 2.0 {
                qw(z, cody8(z))
            } else {
                qw(z, sun_f(z))
            }
        }),
        ("NR F excel_exp", |z| qw(z, nr_f(z))),
        ("NR F libm exp", |z| qw(z, nr_f_libm(z))),
        ("NR F x87 t * excel_exp", |z| qw(z, nr_f_x87(z))),
    ];
    for (name, ev) in graphs {
        let mut mid = Acc::default();
        let mut tail = Acc::default();
        for r in &rows {
            if r.z < 0.5 {
                continue;
            }
            let qg = ev(r.z);
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            if r.z < 4.0 {
                mid.add(d, r.direct);
            } else {
                tail.add(d, r.direct);
            }
        }
        println!("{name:32} mid {} tail {}", fmt(&mid), fmt(&tail));
    }
}
