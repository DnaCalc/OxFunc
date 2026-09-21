//! fdlibm documented 3-piece unsplit-w, NSWC-small+PA/QA+Cody,
//! A[] term-count, Cody P/Q tail joint. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const AS: [f64; 21] = [
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
const PP: [f64; 6] = [
    0.305326634961232344,
    0.360344899949804439,
    0.125781726111229246,
    0.0160837851487422766,
    6.58749161529837803e-4,
    0.0163153871373020978,
];
const QQ: [f64; 5] = [
    2.56852019228982242,
    1.87295284992346047,
    0.527905102951428412,
    0.0605183413124413191,
    0.00233520497626869185,
];
const FDLIBM_PP: [f64; 5] = [
    1.28379167095512558561e-01,
    -3.25042107247001499370e-01,
    -2.84817495755985104766e-02,
    -5.77027029648944159157e-03,
    -2.37630166566501626084e-05,
];
const FDLIBM_QQ: [f64; 5] = [
    3.97917223959155352819e-01,
    6.50222499887672944485e-02,
    5.08130628187576562776e-03,
    1.32494738004321644526e-04,
    -3.96022827877536812320e-06,
];
const FDLIBM_PA: [f64; 7] = [
    -2.36211856075265944077e-03,
    4.14856118683748331666e-01,
    -3.72207876035701323847e-01,
    3.18346619901161753674e-01,
    -1.10894694282396677476e-01,
    3.54783043256182359371e-02,
    -2.16637559486879084300e-03,
];
const FDLIBM_QA: [f64; 6] = [
    1.06420880400844228286e-01,
    5.40397917702171048937e-01,
    7.18286544141962662868e-02,
    1.26171219808761642112e-01,
    1.36370839120290507362e-02,
    1.19844998467991074170e-02,
];
const ERX: f64 = 8.45062911510467529297e-01;

fn x87_horner_hi(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ext_from_f64(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ext_from_f64(c), CW);
    }
    acc
}
fn horner_lo(cs: &[f64], x: f64) -> f64 {
    let mut a = cs[0];
    for &c in &cs[1..] {
        a = a * x + c;
    }
    a
}
fn small_nswc_n(z: f64, n: usize) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let xe = ext_from_f64(z);
    let t = ext_mul(&xe, &xe, CW);
    let ww = x87_horner_hi(&AS[..n], t);
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
fn cody_pq(y: f64, p: &[f64; 6], q: &[f64; 5]) -> f64 {
    let ye = ext_from_f64(y);
    let ysq = ext_div(&ext_from_f64(1.0), &ext_mul(&ye, &ye, CW), CW);
    let mut xnum = ext_mul(&ext_from_f64(p[5]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..4 {
        xnum = ext_mul(&ext_add(&xnum, &ext_from_f64(p[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ext_from_f64(q[i]), CW), &ysq, CW);
    }
    let r = ext_div(
        &ext_mul(&ysq, &ext_add(&xnum, &ext_from_f64(p[4]), CW), CW),
        &ext_add(&xden, &ext_from_f64(q[4]), CW),
        CW,
    );
    ext_to_f64(
        &ext_div(&ext_sub(&ext_from_f64(f::RPINV), &r, CW), &ye, CW),
        CW,
    )
}
fn fd_pp(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let zz = z * z;
    let r = horner_lo(&FDLIBM_PP, zz);
    let s = 1.0 + zz * horner_lo(&FDLIBM_QQ, zz);
    let y = r / s;
    (0.5 - (z * y + (z - 0.5))) / w
}
fn fd_pa(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let s = z - 1.0;
    let p = horner_lo(&FDLIBM_PA, s);
    let q = 1.0 + s * horner_lo(&FDLIBM_QA, s);
    ((1.0 - ERX) - p / q) / w
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
fn score(rows: &[f::QRow], eval: impl Fn(f64) -> f64) -> (Acc, Acc) {
    let mut mid = Acc::default();
    let mut tail = Acc::default();
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
        } else {
            tail.add(d);
        }
    }
    (mid, tail)
}

fn poke(x: f64, k: i32) -> f64 {
    let mut v = x;
    if k > 0 {
        for _ in 0..k {
            v = v.next_up();
        }
    } else {
        for _ in 0..(-k) {
            v = v.next_down();
        }
    }
    v
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);

    println!("## fdlibm documented 3-piece unsplit-w");
    let (m, t) = score(&rows, |z| {
        if z < 0.84375 {
            fd_pp(z)
        } else if z < 1.25 {
            fd_pa(z)
        } else {
            cody_cd(z)
        }
    });
    println!("PP<0.84375 PA<1.25 Cody  mid {} tail {}", fmt(&m), fmt(&t));
    let (m, t) = score(&rows, |z| {
        if z < 0.84375 {
            small_nswc_n(z, 21)
        } else if z < 1.25 {
            fd_pa(z)
        } else {
            cody_cd(z)
        }
    });
    println!("NSWCsmall<0.84375 PA<1.25 Cody  mid {} tail {}", fmt(&m), fmt(&t));
    let (m, t) = score(&rows, |z| {
        if z < 0.84375 {
            small_nswc_n(z, 21)
        } else if z < 1.25 {
            fd_pa(z)
        } else if z < 4.0 {
            cody_cd(z)
        } else {
            f::cody_erfcx_f(z)
        }
    });
    println!("same + Cody P/Q tail  mid {} tail {}", fmt(&m), fmt(&t));

    println!("\n## NSWC A[] term-count at cut=1 + Cody C/D");
    for n in 6..=21 {
        let (m, t) = score(&rows, |z| {
            if z < 1.0 {
                small_nswc_n(z, n)
            } else {
                cody_cd(z)
            }
        });
        println!("n={n:2} mid {} tail {}", fmt(&m), fmt(&t));
    }

    println!("\n## Cody P/Q tail joint ±1,2 (z>=4 only)");
    let mut p = PP;
    let mut q = QQ;
    let tail_now = |p: &[f64; 6], q: &[f64; 5]| score(&rows, |z| cody_pq(z, p, q)).1;
    let mut best = tail_now(&p, &q);
    println!("base tail {}", fmt(&best));
    let deltas = [1i32, -1, 2, -2];
    loop {
        let mut moved = false;
        for i in 0..6 {
            for &k in &deltas {
                let old = p[i];
                p[i] = poke(old, k);
                let sc = tail_now(&p, &q);
                if sc.exact > best.exact || (sc.exact == best.exact && sc.sum_ulp < best.sum_ulp) {
                    best = sc;
                    moved = true;
                    println!("  keep P[{i}] {k:+} tail {}", fmt(&sc));
                } else {
                    p[i] = old;
                }
            }
        }
        for i in 0..5 {
            for &k in &deltas {
                let old = q[i];
                q[i] = poke(old, k);
                let sc = tail_now(&p, &q);
                if sc.exact > best.exact || (sc.exact == best.exact && sc.sum_ulp < best.sum_ulp) {
                    best = sc;
                    moved = true;
                    println!("  keep Q[{i}] {k:+} tail {}", fmt(&sc));
                } else {
                    q[i] = old;
                }
            }
        }
        if !moved {
            break;
        }
    }
    println!("joint P/Q tail {}", fmt(&best));
}
