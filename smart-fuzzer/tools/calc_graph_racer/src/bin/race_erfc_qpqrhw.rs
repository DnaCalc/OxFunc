//! NSWC PQR HW=1,2 stores on Q=w*F for z in [0.5,2). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const NSITE: u32 = 29;
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

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn horner_mask(cs: &[f64], x: Ext80, mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
        acc = maybe(acc, mask, bit);
        bit += 1;
    }
    (acc, bit)
}
fn pqr(x: f64, mask: u32) -> f64 {
    let xe = ef(x);
    let mut num = ext_sub(&xe, &ef(3.75), CW);
    num = maybe(num, mask, 0);
    let mut den = ext_add(&xe, &ef(3.75), CW);
    den = maybe(den, mask, 1);
    let mut t = ext_div(&num, &den, CW);
    t = maybe(t, mask, 2);
    let (p, b) = horner_mask(&P, xe, mask, 3);
    let (q, b) = horner_mask(&Q, xe, mask, b);
    let mut acc = ext_div(&p, &q, CW);
    acc = maybe(acc, mask, b);
    let (r, _) = horner_mask(&R, t, mask, b + 1);
    // R is added as Chebyshev-like: acc starts as P/Q then Horner R on t
    // NSWC: acc = P/Q; for ri in R.rev() { acc = acc*t + ri }
    // horner_mask on R from high already does that if we use R as coeffs of t.
    // Wait: we computed r = horner(R, t) separately. The FORTRAN is
    // acc = P/Q then acc = ((... (P/Q)*t + R8)*t + ... + R0)
    // That's NOT horner(R,t)+P/Q. It's horner with P/Q as the leading extra.
    // Correct: start acc = P/Q, then for ri in R.rev(): acc = acc*t+ri
    let _ = r;
    let mut acc2 = acc;
    let mut bit = b + 1;
    for &ri in R.iter().rev() {
        acc2 = ext_add(&ext_mul(&acc2, &t, CW), &ef(ri), CW);
        acc2 = maybe(acc2, mask, bit);
        bit += 1;
    }
    ext_to_f64(&acc2, CW)
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

#[derive(Clone, Copy)]
struct Acc {
    exact: usize,
    n: usize,
    d: usize,
}
fn score(rows: &[f::QRow], mask: u32) -> Acc {
    let mut exact = 0usize;
    let mut n = 0usize;
    let mut d = 0usize;
    for r in rows {
        if r.z < 0.5 || r.z >= 2.0 {
            continue;
        }
        let qg = qw(r.z, pqr(r.z, mask));
        if !qg.is_finite() {
            continue;
        }
        let dist = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if dist > ULP_CAP {
            continue;
        }
        n += 1;
        if dist == 0 {
            exact += 1;
            if r.direct {
                d += 1;
            }
        }
    }
    Acc { exact, n, d }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let base = score(&rows, 0);
    println!("mask0 {}/{} d={}", base.exact, base.n, base.d);
    let mut best = base;
    let mut lab = 0u32;
    println!("## HW=1");
    for b in 0..NSITE {
        let m = 1u32 << b;
        let sc = score(&rows, m);
        if sc.exact > best.exact || (sc.exact == best.exact && sc.d > best.d) {
            best = sc;
            lab = m;
            println!("HIT bit={b} mask={m:#x} {}/{} d={}", sc.exact, sc.n, sc.d);
        }
    }
    println!("best HW1 mask={lab:#x} {}/{} d={}", best.exact, best.n, best.d);
    println!("## HW=2");
    for i in 0..NSITE {
        for j in (i + 1)..NSITE {
            let m = (1u32 << i) | (1u32 << j);
            let sc = score(&rows, m);
            if sc.exact > best.exact || (sc.exact == best.exact && sc.d > best.d) {
                best = sc;
                lab = m;
                println!(
                    "HIT bits={i},{j} mask={m:#x} {}/{} d={}",
                    sc.exact, sc.n, sc.d
                );
            }
        }
    }
    println!(
        "best HW<=2 mask={lab:#x} {}/{} d={}",
        best.exact, best.n, best.d
    );
}
