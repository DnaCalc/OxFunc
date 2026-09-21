//! Three-piece documented NSWC cuts + fine cut around 1.35.
//! Follow-up. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const A: [f64; 21] = [
    0.1283791670955125738961589031215,
    -0.3761263890318375246320529677070,
    0.1128379167095512573896158902931,
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

fn x87_horner(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ext_from_f64(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ext_from_f64(c), CW);
    }
    acc
}
fn small_f(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let xe = ext_from_f64(z);
    let t = ext_mul(&xe, &xe, CW);
    let ww = x87_horner(&A, t);
    let inner = ext_mul(&xe, &ext_add(&ext_from_f64(1.0), &ww, CW), CW);
    ext_to_f64(
        &ext_add(
            &ext_from_f64(0.5),
            &ext_sub(&ext_from_f64(0.5), &inner, CW),
            CW,
        ),
        CW,
    ) / w
}
fn pqr_x87(x: f64) -> f64 {
    let xe = ext_from_f64(x);
    let t = ext_div(
        &ext_sub(&xe, &ext_from_f64(3.75), CW),
        &ext_add(&xe, &ext_from_f64(3.75), CW),
        CW,
    );
    let mut acc = ext_div(&x87_horner(&P, xe), &x87_horner(&Q, xe), CW);
    for &ri in R.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ext_from_f64(ri), CW);
    }
    ext_to_f64(&acc, CW)
}
fn cody_cd(y: f64) -> f64 {
    let ye = ext_from_f64(y);
    let mut xnum = ext_mul(&ext_from_f64(C[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ext_from_f64(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ext_from_f64(D[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ext_from_f64(C[7]), CW),
            &ext_add(&xden, &ext_from_f64(D[7]), CW),
            CW,
        ),
        CW,
    )
}
fn cody_f(y: f64) -> f64 {
    if y <= 4.0 {
        cody_cd(y)
    } else {
        f::cody_erfcx_f(y)
    }
}

#[derive(Default, Clone, Copy)]
struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    sum_ulp: u128,
}
impl Acc {
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
fn fmt(a: &Acc) -> String {
    format!("{}/{} max={} sum={}", a.exact, a.n, a.max_ulp, a.sum_ulp)
}
fn score(rows: &[f::QRow], eval: impl Fn(f64) -> f64) -> (Acc, Acc, Acc) {
    let mut mid = Acc::default();
    let mut tail = Acc::default();
    let mut dmid = Acc::default();
    for r in rows {
        if r.z < 0.5 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let fg = eval(r.z);
        if !fg.is_finite() {
            continue;
        }
        let d = ulp_distance(fg, fo).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        if r.z < 4.0 {
            mid.add(d);
            if r.direct {
                dmid.add(d);
            }
        } else {
            tail.add(d);
        }
    }
    (mid, tail, dmid)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    println!("## fine cut 1.28 .. 1.42 step 0.001");
    let mut best = 0usize;
    let mut best_c = 0.0;
    let mut best_sum = u128::MAX;
    for k in 1280..=1420 {
        let cut = k as f64 / 1000.0;
        let (m, t, dm) = score(&rows, |z| if z < cut { small_f(z) } else { cody_f(z) });
        if m.exact > best || (m.exact == best && m.sum_ulp < best_sum) {
            best = m.exact;
            best_c = cut;
            best_sum = m.sum_ulp;
            println!(
                "NEW cut={cut:.3} mid {} tail {} dmid {}",
                fmt(&m),
                fmt(&t),
                fmt(&dm)
            );
        }
    }
    println!("best fine cut={best_c:.3} mid={best}");

    println!("\n## three-piece documented");
    let graphs: [(&str, fn(f64) -> f64); 6] = [
        ("small<1 PQR[1,2] Cody>2", |z| {
            if z < 1.0 {
                small_f(z)
            } else if z <= 2.0 {
                pqr_x87(z)
            } else {
                cody_f(z)
            }
        }),
        ("small<1 PQR[1,2] AABB(2,4] CCDD>4", |z| {
            if z < 1.0 {
                small_f(z)
            } else if z <= 2.0 {
                pqr_x87(z)
            } else {
                f::nswc_derfc0(z)
            }
        }),
        ("small<1 Cody[1,4] CF>4 n80", |z| {
            if z < 1.0 {
                small_f(z)
            } else if z < 4.0 {
                cody_cd(z)
            } else {
                f::cf_as714_x87_n(z, 80)
            }
        }),
        ("small<1.35 Cody CF>4.9 n80", |z| {
            if z < 1.35 {
                small_f(z)
            } else if z < 4.9 {
                cody_f(z)
            } else {
                f::cf_as714_x87_n(z, 80)
            }
        }),
        ("PQR x87 z<2 Cody z>=2", |z| {
            if z < 2.0 {
                pqr_x87(z)
            } else {
                cody_f(z)
            }
        }),
        ("small<1 PQR x87 z>=1 (no AABB)", |z| {
            if z < 1.0 {
                small_f(z)
            } else {
                pqr_x87(z)
            }
        }),
    ];
    for (name, g) in graphs {
        let (m, t, dm) = score(&rows, g);
        println!("{name:48} mid {} tail {} dmid {}", fmt(&m), fmt(&t), fmt(&dm));
    }
}
