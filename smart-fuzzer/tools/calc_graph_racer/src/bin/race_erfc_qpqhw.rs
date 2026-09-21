//! Cody SPECFUN P/Q HW=1,2 stores on Q-tail. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const NSITE: u32 = 16;
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
fn cody_pq(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut ysq = ext_mul(&ye, &ye, CW);
    ysq = maybe(ysq, mask, 0);
    ysq = ext_div(&ef(1.0), &ysq, CW);
    ysq = maybe(ysq, mask, 1);
    let mut xnum = ext_mul(&ef(PP[5]), &ysq, CW);
    let mut xden = ysq;
    xnum = maybe(xnum, mask, 2);
    xden = maybe(xden, mask, 3);
    for i in 0..4 {
        xnum = ext_mul(&ext_add(&xnum, &ef(PP[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(QQ[i]), CW), &ysq, CW);
        xnum = maybe(xnum, mask, 4 + 2 * i as u32);
        xden = maybe(xden, mask, 5 + 2 * i as u32);
    }
    let mut r = ext_div(
        &ext_mul(&ysq, &ext_add(&xnum, &ef(PP[4]), CW), CW),
        &ext_add(&xden, &ef(QQ[4]), CW),
        CW,
    );
    r = maybe(r, mask, 12);
    let mut t = ext_sub(&ef(f::RPINV), &r, CW);
    t = maybe(t, mask, 13);
    let mut f80 = ext_div(&t, &ye, CW);
    f80 = maybe(f80, mask, 14);
    ext_to_f64(&f80, CW)
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
struct Tail {
    exact: usize,
    d: usize,
    max_ulp: u64,
}
fn score(rows: &[f::QRow], mask: u32) -> Tail {
    let mut exact = 0usize;
    let mut d = 0usize;
    let mut max_ulp = 0u64;
    for r in rows {
        if r.z < 4.0 {
            continue;
        }
        let qg = qw(r.z, cody_pq(r.z, mask));
        if !qg.is_finite() {
            continue;
        }
        let dist = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if dist > ULP_CAP {
            continue;
        }
        if dist == 0 {
            exact += 1;
            if r.direct {
                d += 1;
            }
        } else {
            max_ulp = max_ulp.max(dist);
        }
    }
    Tail {
        exact,
        d,
        max_ulp,
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let base = score(&rows, 0);
    println!("mask0 {} d={} max={}", base.exact, base.d, base.max_ulp);
    let mut best = base;
    let mut lab = 0u32;
    println!("## HW=1");
    for b in 0..NSITE {
        let m = 1u32 << b;
        let sc = score(&rows, m);
        if sc.exact > best.exact || (sc.exact == best.exact && sc.d > best.d) {
            best = sc;
            lab = m;
            println!("HIT bit={b} mask={m:#x} {} d={}", sc.exact, sc.d);
        }
    }
    println!("best HW1 mask={lab:#x} {} d={}", best.exact, best.d);
    println!("## HW=2");
    for i in 0..NSITE {
        for j in (i + 1)..NSITE {
            let m = (1u32 << i) | (1u32 << j);
            let sc = score(&rows, m);
            if sc.exact > best.exact || (sc.exact == best.exact && sc.d > best.d) {
                best = sc;
                lab = m;
                println!("HIT bits={i},{j} mask={m:#x} {} d={}", sc.exact, sc.d);
            }
        }
    }
    println!(
        "best HW<=2 mask={lab:#x} {} d={} max={}",
        best.exact, best.d, best.max_ulp
    );
}
