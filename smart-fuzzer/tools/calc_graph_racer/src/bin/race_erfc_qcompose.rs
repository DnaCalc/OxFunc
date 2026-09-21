//! Score ERFC.PRECISE Q bits, not F_or=Q/w. Small piece is erfc-direct
//! (NSWC |x|<=1). Cody piece is w⊗F. Not an identity.
//!
//!   cargo run --release --bin race_erfc_qcompose -- G3-01-dist

use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;
use std::fs;
use std::io::Write;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn horner_hi(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
    }
    acc
}

fn cody_f80(y: f64) -> Ext80 {
    let ye = ef(y);
    if y <= 4.0 {
        let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
        let mut xden = ye;
        for i in 0..7 {
            xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
            xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
        }
        ext_div(
            &ext_add(&xnum, &ef(C[7]), CW),
            &ext_add(&xden, &ef(D[7]), CW),
            CW,
        )
    } else {
        let ysq = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
        let mut xnum = ext_mul(&ef(PP[5]), &ysq, CW);
        let mut xden = ysq;
        for i in 0..4 {
            xnum = ext_mul(&ext_add(&xnum, &ef(PP[i]), CW), &ysq, CW);
            xden = ext_mul(&ext_add(&xden, &ef(QQ[i]), CW), &ysq, CW);
        }
        let r = ext_div(
            &ext_mul(&ysq, &ext_add(&xnum, &ef(PP[4]), CW), CW),
            &ext_add(&xden, &ef(QQ[4]), CW),
            CW,
        );
        ext_div(&ext_sub(&ef(f::RPINV), &r, CW), &ye, CW)
    }
}

fn small_erfc80(z: f64) -> Ext80 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let w = horner_hi(&AS, t);
    let inner = ext_mul(&xe, &ext_add(&ef(1.0), &w, CW), CW);
    ext_add(&ef(0.5), &ext_sub(&ef(0.5), &inner, CW), CW)
}

#[derive(Clone, Copy)]
enum Comp {
    /// Q = RN53(w_f64 * F_f64)
    Wf64Ff64,
    /// Q = RN53(x87(w_f64 * F_80))
    Wx87F80,
    /// Q = RN53(x87(w_80 * F_80))
    W80F80,
}

fn q_from_f(z: f64, f80: Ext80, comp: Comp) -> f64 {
    let w = f::w_rn53(z);
    match comp {
        Comp::Wf64Ff64 => w * ext_to_f64(&f80, CW),
        Comp::Wx87F80 => ext_to_f64(&ext_mul(&ef(w), &f80, CW), CW),
        Comp::W80F80 => ext_to_f64(&ext_mul(&ef(w), &f80, CW), CW),
    }
}

fn q_small_direct(z: f64) -> f64 {
    ext_to_f64(&small_erfc80(z), CW)
}
fn q_small_via_f(z: f64, comp: Comp) -> f64 {
    let erfc = small_erfc80(z);
    let w = f::w_rn53(z);
    if w == 0.0 {
        return f64::NAN;
    }
    let f80 = ext_div(&erfc, &ef(w), CW);
    q_from_f(z, f80, comp)
}

#[derive(Default, Clone, Copy)]
struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    sum_ulp: u128,
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
            self.sum_ulp += d as u128;
        }
    }
}
fn fmt(a: &Acc) -> String {
    format!(
        "{}/{} max={} sum={} d={}/{}",
        a.exact, a.n, a.max_ulp, a.sum_ulp, a.dmid, a.dn
    )
}

fn score_q(rows: &[f::QRow], eval: impl Fn(f64) -> f64) -> (Acc, Acc) {
    let mut mid = Acc::default();
    let mut tail = Acc::default();
    for r in rows {
        if r.z < 0.5 {
            continue;
        }
        let qg = eval(r.z);
        if !qg.is_finite() {
            continue;
        }
        let qo = f64::from_bits(r.qbits);
        let d = ulp_distance(qg, qo).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        if r.z < 4.0 {
            mid.add(d, r.direct);
        } else {
            tail.add(d, r.direct);
        }
    }
    (mid, tail)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let out = env::args().nth(2).unwrap_or_else(|| ".".into());
    let rows = f::load_q_rows_tagged(&dir);
    let n_dmid = rows.iter().filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0).count();
    println!("rows={} direct_mid_z={n_dmid}", rows.len());

    println!("\n## Cody C/D as F, compose Q=w⊗F (global z>=0.5, C/D<=4 else P/Q)");
    for (name, comp) in [
        ("Q=RN53(w*F64)", Comp::Wf64Ff64),
        ("Q=RN53(x87 w*F80)", Comp::Wx87F80),
    ] {
        let (m, t) = score_q(&rows, |z| q_from_f(z, cody_f80(z), comp));
        println!("{name:28} mid {} tail {}", fmt(&m), fmt(&t));
    }

    println!("\n## documented NSWC: z<cut erfc-direct, else w⊗Cody");
    for cut in [0.5, 0.8, 1.0, 1.25, 1.347, 1.5] {
        for (tag, comp) in [("wF64", Comp::Wf64Ff64), ("x87", Comp::Wx87F80)] {
            let (m, t) = score_q(&rows, |z| {
                if z < cut {
                    q_small_direct(z)
                } else {
                    q_from_f(z, cody_f80(z), comp)
                }
            });
            println!("cut={cut:.3} small-direct Cody-{tag:5} mid {} tail {}", fmt(&m), fmt(&t));
        }
        let (m, t) = score_q(&rows, |z| {
            if z < cut {
                q_small_via_f(z, Comp::Wf64Ff64)
            } else {
                q_from_f(z, cody_f80(z), Comp::Wf64Ff64)
            }
        });
        println!("cut={cut:.3} small-via-F   Cody-wF64 mid {} tail {}", fmt(&m), fmt(&t));
    }

    println!("\n## NSWC packet: z<1 erfc-direct, 1<=z<=2 skip (use derfc0 F via w), compare nswc_derfc0 compose");
    let (m, t) = score_q(&rows, |z| {
        if z <= 1.0 {
            q_small_direct(z)
        } else {
            let ff = f::nswc_derfc0(z);
            f::w_rn53(z) * ff
        }
    });
    println!("NSWC-faithful Q=w*F64     mid {} tail {}", fmt(&m), fmt(&t));
    let (m, t) = score_q(&rows, |z| {
        if z <= 1.0 {
            q_small_direct(z)
        } else {
            q_from_f(z, ef(f::nswc_derfc0(z)), Comp::Wx87F80)
        }
    });
    println!("NSWC-faithful Q=x87 w*F   mid {} tail {}", fmt(&m), fmt(&t));

    let (m, t) = score_q(&rows, |z| f::w_rn53(z) * f::nswc_derfc0(z));
    println!("nswc_derfc0 global w*F64  mid {} tail {}", fmt(&m), fmt(&t));
    let (m, t) = score_q(&rows, |z| f::w_rn53(z) * f::cody_erfcx_f(z));
    println!("cody native F global w*F64 mid {} tail {}", fmt(&m), fmt(&t));

    println!("\n## direct-mid misses cut=1 small-direct + Cody w*F64 (first 12)");
    let mut nmiss = 0usize;
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let qg = if r.z < 1.0 {
            q_small_direct(r.z)
        } else {
            q_from_f(r.z, cody_f80(r.z), Comp::Wf64Ff64)
        };
        let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if d != 0 {
            nmiss += 1;
            if nmiss <= 12 {
                println!("  z={:.17} ulp={d} qg={:016x} qo={:016x}", r.z, qg.to_bits(), r.qbits);
            }
        }
    }
    println!("direct-mid misses={nmiss}");

    let path = format!("{out}/QCOMPOSE.md");
    let mut w = fs::File::create(&path).unwrap();
    writeln!(w, "# Q-compose (not F_or)\n\nNot an identity. Not landed.\nSee race_erfc_qcompose.out.\n").ok();
    println!("wrote {path}");
}
