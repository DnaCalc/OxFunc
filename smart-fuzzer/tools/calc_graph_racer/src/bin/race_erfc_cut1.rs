//! Cut-1 piecewise: NSWC small-series below cut, Cody x87 C/D above.
//! Follow-up to race_erfc_human 3008/7741 at cut=1.0. Not an identity.
//!
//!   cargo run --release --bin race_erfc_cut1 -- G3-01-dist out-dir

use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;
use std::fs;
use std::io::Write;

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

fn x87_horner(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ext_from_f64(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ext_from_f64(c), CW);
    }
    acc
}

fn small_erfc_x87(x: f64) -> f64 {
    let xe = ext_from_f64(x);
    let t = ext_mul(&xe, &xe, CW);
    let w = x87_horner(&A, t);
    let inner = ext_mul(&xe, &ext_add(&ext_from_f64(1.0), &w, CW), CW);
    ext_to_f64(
        &ext_add(
            &ext_from_f64(0.5),
            &ext_sub(&ext_from_f64(0.5), &inner, CW),
            CW,
        ),
        CW,
    )
}

fn small_f(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        f64::NAN
    } else {
        small_erfc_x87(z) / w
    }
}

fn cody_x87(y: f64) -> f64 {
    let ye = ext_from_f64(y);
    if y <= 4.0 {
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
    } else {
        let ysq = ext_div(&ext_from_f64(1.0), &ext_mul(&ye, &ye, CW), CW);
        let mut xnum = ext_mul(&ext_from_f64(PP[5]), &ysq, CW);
        let mut xden = ysq;
        for i in 0..4 {
            xnum = ext_mul(&ext_add(&xnum, &ext_from_f64(PP[i]), CW), &ysq, CW);
            xden = ext_mul(&ext_add(&xden, &ext_from_f64(QQ[i]), CW), &ysq, CW);
        }
        let r = ext_div(
            &ext_mul(&ysq, &ext_add(&xnum, &ext_from_f64(PP[4]), CW), CW),
            &ext_add(&xden, &ext_from_f64(QQ[4]), CW),
            CW,
        );
        ext_to_f64(
            &ext_div(&ext_sub(&ext_from_f64(f::RPINV), &r, CW), &ye, CW),
            CW,
        )
    }
}

fn piece(z: f64, cut: f64) -> f64 {
    if z < cut {
        small_f(z)
    } else {
        cody_x87(z)
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

fn score(rows: &[f::QRow], eval: impl Fn(f64) -> f64) -> (Acc, Acc, Acc, Acc, [Option<u64>; 5]) {
    let mut mid = Acc::default();
    let mut tail = Acc::default();
    let mut dmid = Acc::default();
    let mut dtail = Acc::default();
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
            if r.direct {
                dtail.add(d);
            }
        }
    }
    let mut pins = [None; 5];
    for (i, &pz) in f::PIN_Z.iter().enumerate() {
        if let Some(fo) = rows.iter().find(|r| r.z == pz).and_then(|r| f::f_or(r.z, r.qbits)) {
            pins[i] = ulp_distance(eval(pz), fo);
        }
    }
    (mid, tail, dmid, dtail, pins)
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
    let out = env::args()
        .nth(2)
        .unwrap_or_else(|| ".".into());
    let rows = f::load_q_rows_tagged(&dir);
    println!("## cut sweep NSWC-small x87 / Cody x87");
    let mut best_mid = 0usize;
    let mut best_cut = 0.0;
    let mut best_dmid = 0usize;
    let mut lines = String::new();
    for k in 50..=160 {
        let cut = k as f64 / 100.0;
        let (m, t, dm, dt, pins) = score(&rows, |z| piece(z, cut));
        if m.exact > best_mid {
            best_mid = m.exact;
            best_cut = cut;
        }
        if dm.exact > best_dmid {
            best_dmid = dm.exact;
        }
        let line = format!(
            "cut={cut:.2} mid {} tail {} dmid {} dtail {} pins {:?}",
            fmt(&m),
            fmt(&t),
            fmt(&dm),
            fmt(&dt),
            pins
        );
        println!("{line}");
        lines.push_str(&line);
        lines.push('\n');
    }
    println!("best_merged_mid={best_mid} at cut={best_cut:.2} best_dmid={best_dmid}");

    println!("\n## compose best-cut with CF / cephes tail");
    for &(cut_m, cut_t, name, tailf) in &[
        (1.0, 4.0, "cutm=1.0 cutt=4.0 as714n80", (|z| f::cf_as714_x87_n(z, 80)) as fn(f64) -> f64),
        (1.0, 4.9, "cutm=1.0 cutt=4.9 as714n80", |z| f::cf_as714_x87_n(z, 80)),
        (1.0, 4.9, "cutm=1.0 cutt=4.9 cephes", f::cephes_f),
        (1.0, 4.0, "cutm=1.0 cutt=4.0 cephes", f::cephes_f),
        (best_cut, 4.9, "bestcut /4.9 cephes", f::cephes_f),
        (best_cut, 4.9, "bestcut /4.9 as714n80", |z| f::cf_as714_x87_n(z, 80)),
    ] {
        let (m, t, dm, dt, pins) = score(&rows, |z| {
            if z < cut_m {
                small_f(z)
            } else if z < cut_t {
                cody_x87(z)
            } else {
                tailf(z)
            }
        });
        println!("{name} mid {} tail {} dmid {} dtail {} pins {:?}", fmt(&m), fmt(&t), fmt(&dm), fmt(&dt), pins);
    }

    println!("\n## one-ULP of A[] small-series (z<1) holding Cody CR");
    let mut a = A;
    let base = score(&rows, |z| piece(z, 1.0)).0;
    println!("base mid {}", fmt(&base));
    let mut best_a = base;
    for i in 0..21 {
        for k in [-2, -1, 1, 2] {
            let old = a[i];
            a[i] = poke(old, k);
            let sc = score(&rows, |z| {
                if z < 1.0 {
                    let w = f::w_rn53(z);
                    if w == 0.0 {
                        f64::NAN
                    } else {
                        let xe = ext_from_f64(z);
                        let t = ext_mul(&xe, &xe, CW);
                        let ww = x87_horner(&a, t);
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
                } else {
                    cody_x87(z)
                }
            })
            .0;
            if sc.exact > best_a.exact
                || (sc.exact == best_a.exact && sc.sum_ulp < best_a.sum_ulp)
            {
                println!("  A[{i}] {k:+} -> mid {}", fmt(&sc));
                best_a = sc;
            } else {
                a[i] = old;
            }
        }
    }
    println!("small-series joint mid {}", fmt(&best_a));

    let path = format!("{out}/CUT1.md");
    let mut w = fs::File::create(&path).unwrap();
    writeln!(
        w,
        "# NSWC-small / Cody-x87 cut sweep\n\nNot an identity. Not landed.\n\nbest_merged_mid={best_mid} at cut={best_cut:.2} (human 3008 was cut=1.0).\nbest_dmid={best_dmid}. Pins remain inexact.\n\n{lines}"
    )
    .ok();
    println!("wrote {path}");
}
