//! NSWC CCDD HW=1,2 scored by direct-tail. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const NSITE: u32 = 28;
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
const E: [f64; 4] = [
    0.540464821348814822409610122136,
    -0.261515522487415653487049835220e-01,
    -0.288573438386338758794591212600e-02,
    -0.529353396945788057720258856000e-03,
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
fn horner(cs: &[f64], x: Ext80, mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
        acc = maybe(acc, mask, bit);
        bit += 1;
    }
    (acc, bit)
}
fn ccdd(x: f64, mask: u32) -> f64 {
    let xe = ef(x);
    let mut z = ext_div(&ef(1.0), &ext_add(&ef(2.5), &ext_mul(&xe, &xe, CW), CW), CW);
    z = maybe(z, mask, 0);
    let mut t = ext_sub(&ext_mul(&ef(13.0), &z, CW), &ef(1.0), CW);
    t = maybe(t, mask, 1);
    let (n, b) = horner(&CC, z, mask, 2);
    let (d, b) = horner(&DD, z, mask, b);
    let mut acc = ext_div(&n, &d, CW);
    acc = maybe(acc, mask, b);
    let mut bit = b + 1;
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E[3]), CW);
    acc = maybe(acc, mask, bit);
    bit += 1;
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E[2]), CW);
    acc = maybe(acc, mask, bit);
    bit += 1;
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E[1]), CW);
    acc = maybe(acc, mask, bit);
    bit += 1;
    acc = ext_add(&ext_mul(&acc, &t, CW), &ef(E[0]), CW);
    acc = maybe(acc, mask, bit);
    bit += 1;
    acc = ext_div(&acc, &xe, CW);
    acc = maybe(acc, mask, bit);
    ext_to_f64(&acc, CW)
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

#[derive(Clone, Copy, Default)]
struct Sc {
    exact: usize,
    dtail: usize,
    m1d: usize,
    max_d: u64,
}
fn score(rows: &[f::QRow], mask: u32) -> Sc {
    let mut s = Sc::default();
    for r in rows {
        if r.z < 4.0 {
            continue;
        }
        let qg = qw(r.z, ccdd(r.z, mask));
        if !qg.is_finite() {
            continue;
        }
        let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        if d == 0 {
            s.exact += 1;
        }
        if r.direct {
            s.max_d = s.max_d.max(d);
            if d == 0 {
                s.dtail += 1;
            } else if d == 1 {
                s.m1d += 1;
            }
        }
    }
    s
}
fn better(a: Sc, b: Sc) -> bool {
    a.dtail > b.dtail
        || (a.dtail == b.dtail && (a.dtail + a.m1d) > (b.dtail + b.m1d))
        || (a.dtail == b.dtail && a.m1d == b.m1d && a.max_d < b.max_d)
        || (a.dtail == b.dtail && a.m1d == b.m1d && a.max_d == b.max_d && a.exact > b.exact)
}
fn fmt(s: Sc) -> String {
    format!(
        "exact={} dtail={} m1d={} w1d={} maxd={}",
        s.exact,
        s.dtail,
        s.m1d,
        s.dtail + s.m1d,
        s.max_d
    )
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let base = score(&rows, 0);
    println!("CCDD mask0 {}", fmt(base));
    let mut best1 = base;
    let mut lab1 = 0u32;
    println!("## HW=1");
    for b in 0..NSITE {
        let m = 1u32 << b;
        let sc = score(&rows, m);
        if better(sc, best1) {
            best1 = sc;
            lab1 = m;
            println!("HIT bit={b} mask={m:#x} {}", fmt(sc));
        }
    }
    println!("best HW1 mask={lab1:#x} {}", fmt(best1));
    println!("## HW=2");
    let mut best2 = best1;
    let mut lab2 = lab1;
    for i in 0..NSITE {
        for j in (i + 1)..NSITE {
            let m = (1u32 << i) | (1u32 << j);
            let sc = score(&rows, m);
            if better(sc, best2) {
                best2 = sc;
                lab2 = m;
                println!("HIT bits={i},{j} mask={m:#x} {}", fmt(sc));
            }
        }
    }
    println!("best HW2 mask={lab2:#x} {}", fmt(best2));
}
