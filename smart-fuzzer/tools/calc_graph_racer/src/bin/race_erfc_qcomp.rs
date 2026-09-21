//! Q from 1-erf on [0.5, c) vs Cody C/D. Complement at the 0.5 pin.
//! Not an identity.
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
const AA: [f64; 5] = [
    3.16112374387056560,
    113.864154151050156,
    377.485237685302021,
    3209.37758913846947,
    0.185777706184603153,
];
const BB: [f64; 4] = [
    23.6012909523441209,
    244.024637934444173,
    1282.61652607737228,
    2844.23683343917062,
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
fn cody_cd(y: f64) -> f64 {
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
fn a21_erf(z: f64) -> f64 {
    let ze = ef(z.abs());
    let zz = ext_mul(&ze, &ze, CW);
    let w = horner(&AS, zz);
    let p = ext_add(&ze, &ext_mul(&ze, &w, CW), CW);
    ext_to_f64(&p, CW)
}
fn cody_ab_erf(z: f64) -> f64 {
    let ze = ef(z.abs());
    let ysq = ext_mul(&ze, &ze, CW);
    let num = ext_add(&horner(&AA[..4], ysq), &ext_mul(&ef(AA[4]), &ysq, CW), CW);
    // SPECFUN: xnum starts A[4]*ysq? use (A3 + ysq*(A2+...)) * z
    let n = horner(&[AA[3], AA[2], AA[1], AA[0], AA[4]], ysq);
    let d = ext_add(&horner(&BB, ysq), &ef(1.0), CW);
    ext_to_f64(&ext_mul(&ze, &ext_div(&n, &d, CW), CW), CW)
}
fn q_from_p(p: f64) -> f64 {
    // RN64(1-P) then RN53, approx: x87 1-p stored f64
    let d = ext_sub(&ef(1.0), &ef(p), CW);
    ext_to_f64(&d, CW)
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let cuts = [0.5f64, 0.501, 0.6174, 0.75, 0.84375, 1.0];
    println!("## 1-A21 below cut else Cody w*F, Q bits");
    for &cut in &cuts {
        let mut exact = 0usize;
        let mut dmid = 0usize;
        let mut dn = 0usize;
        let mut n = 0usize;
        let mut max_d = 0u64;
        let mut pin05 = None;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let qg = if r.z < cut {
                q_from_p(a21_erf(r.z))
            } else {
                qw(r.z, cody_cd(r.z))
            };
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                exact += 1;
            }
            if r.direct {
                dn += 1;
                max_d = max_d.max(d);
                if d == 0 {
                    dmid += 1;
                }
            }
            if (r.z - 0.5).abs() < 1e-15 {
                pin05 = Some((d, r.direct));
            }
        }
        println!(
            "cut={cut} exact={exact}/{n} dmid={dmid}/{dn} maxd={max_d} pin0.5={pin05:?}"
        );
    }
    println!("## 1-CodyAB below cut else Cody C/D");
    for &cut in &[0.5f64, 0.46875, 0.84375] {
        let mut exact = 0usize;
        let mut dmid = 0usize;
        let mut dn = 0usize;
        let mut n = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let qg = if r.z < cut {
                q_from_p(cody_ab_erf(r.z))
            } else {
                qw(r.z, cody_cd(r.z))
            };
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                exact += 1;
            }
            if r.direct {
                dn += 1;
                if d == 0 {
                    dmid += 1;
                }
            }
        }
        println!("AB cut={cut} exact={exact}/{n} dmid={dmid}/{dn}");
    }
    // pin 0.5 only: C/D vs 1-A21 vs 1-AB vs libm
    println!("## z~0.5 pin graphs");
    for r in &rows {
        if (r.z - 0.5).abs() > 1e-12 {
            continue;
        }
        let qo = f64::from_bits(r.qbits);
        let g = [
            ("cody w*F", qw(r.z, cody_cd(r.z))),
            ("1-A21 x87", q_from_p(a21_erf(r.z))),
            ("1-AB x87", q_from_p(cody_ab_erf(r.z))),
            ("libm erfc", libm::erfc(r.z)),
            ("1-libm erf", 1.0 - libm::erf(r.z)),
        ];
        println!("z={:.17} direct={} qo={:016x}", r.z, r.direct, r.qbits);
        for (n, v) in g {
            let d = ulp_distance(v, qo).unwrap_or(u64::MAX);
            println!("  {n:16} bits={:016x} ulp={d}", v.to_bits());
        }
    }
}
