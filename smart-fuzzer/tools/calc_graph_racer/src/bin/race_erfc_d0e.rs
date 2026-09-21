//! derfc0 E0/E1/E2 ±1 and /x last-store vs 6 leftover-low. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use std::env;

const SIX: [f64; 6] = [
    4.0520833333333330,
    4.0625000000000000,
    5.2395833333333330,
    5.2708333333333330,
    5.3333333333333330,
    6.0000000000000000,
];
const AA0: [f64; 9] = [
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
const BB0: [f64; 12] = [
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
fn horner(cs: &[f64], x: f64) -> f64 {
    let mut a = 0.0;
    for &c in cs.iter().rev() {
        a = a * x + c;
    }
    a
}
fn acc_of(x: f64, e0: f64, e1: f64, e2: f64) -> f64 {
    let z = 1.0 / (2.5 + x * x);
    let t = 13.0 * z - 1.0;
    ((horner(&AA0, z) / horner(&BB0, z) * t + e2) * t + e1) * t + e0
}
fn derfc0_e(x: f64, e0: f64, e1: f64, e2: f64, last: &str) -> f64 {
    if x <= 2.0 {
        return f::nswc_pqr_f(x);
    }
    if x <= 4.0 {
        let a = acc_of(x, e0, e1, e2);
        return match last {
            "up" => (a / x).next_up(),
            "dn" => (a / x).next_down(),
            "upacc" => a.next_up() / x,
            "dnacc" => a.next_down() / x,
            _ => a / x,
        };
    }
    f::nswc_ccdd_f(x)
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
    let tail: Vec<&f::QRow> = rows.iter().filter(|r| r.direct && r.z >= 4.0).collect();
    let sc = |e0: f64, e1: f64, e2: f64, last: &str| {
        let mut qd = 0usize;
        let mut qe = 0usize;
        let mut hit = 0usize;
        let mut zs = Vec::new();
        for row in rows.iter().filter(|rr| rr.z >= 4.0) {
            let g = qw(row.z, derfc0_e(row.z, e0, e1, e2, last));
            if !g.is_finite() {
                continue;
            }
            if ulp_distance(g, f64::from_bits(row.qbits)).unwrap_or(99) == 0 {
                qe += 1;
                if row.direct {
                    qd += 1;
                }
            }
        }
        for &lz in &SIX {
            let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
                continue;
            };
            if ulp_distance(
                qw(lz, derfc0_e(lz, e0, e1, e2, last)),
                f64::from_bits(r.qbits),
            )
            .unwrap_or(99)
                == 0
            {
                hit += 1;
                zs.push(lz);
            }
        }
        (qe, qd, hit, zs)
    };
    println!("E/last-store (bar CR Q 1466 d=46 hit 1/6):");
    let cases: [(&str, f64, f64, f64, &str); 11] = [
        ("CR", E0, E1, E2, ""),
        ("E0+1", poke(E0, 1), E1, E2, ""),
        ("E0-1", poke(E0, -1), E1, E2, ""),
        ("E1+1", E0, poke(E1, 1), E2, ""),
        ("E1-1", E0, poke(E1, -1), E2, ""),
        ("E2+1", E0, E1, poke(E2, 1), ""),
        ("E2-1", E0, E1, poke(E2, -1), ""),
        ("/x up", E0, E1, E2, "up"),
        ("/x dn", E0, E1, E2, "dn"),
        ("acc+1", E0, E1, E2, "upacc"),
        ("acc-1", E0, E1, E2, "dnacc"),
    ];
    for (name, e0, e1, e2, last) in cases {
        let (qe, qd, h, zs) = sc(e0, e1, e2, last);
        println!("  {name:8} Q={qe} d={qd} hit={h}/6 {zs:?}");
    }
}
