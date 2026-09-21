//! NSWC ccdd CC/DD/E ±1 vs 6 leftover-low. Not an identity.
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
const CC0: [f64; 9] = [
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
const DD0: [f64; 10] = [
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
const E0: f64 = 0.540464821348814822409610122136;
const E1: f64 = -0.261515522487415653487049835220e-01;
const E2: f64 = -0.288573438386338758794591212600e-02;
const E3: f64 = -0.529353396945788057720258856000e-03;

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
fn ccdd(
    x: f64,
    cc: &[f64; 9],
    dd: &[f64; 10],
    e0: f64,
    e1: f64,
    e2: f64,
    e3: f64,
) -> f64 {
    let z = 1.0 / (2.5 + x * x);
    let t = 13.0 * z - 1.0;
    let acc = (((horner(cc, z) / horner(dd, z) * t + e3) * t + e2) * t + e1) * t + e0;
    acc / x
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
    let sc = |cc: &[f64; 9], dd: &[f64; 10], e0: f64, e1: f64, e2: f64, e3: f64| {
        let mut qd = 0usize;
        let mut qe = 0usize;
        let mut hit = 0usize;
        let mut zs = Vec::new();
        for row in rows.iter().filter(|rr| rr.z >= 4.0) {
            let g = qw(row.z, ccdd(row.z, cc, dd, e0, e1, e2, e3));
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
                qw(lz, ccdd(lz, cc, dd, e0, e1, e2, e3)),
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
    let (qe0, qd0, h0, _) = sc(&CC0, &DD0, E0, E1, E2, E3);
    println!("ccdd CR Q={qe0} d={qd0} hit={h0}/6 (bar 0x5005 1572/53)");
    println!("±1 (print hit>h0 or d>=qd0 or Q>=qe0):");
    let mut best_h = h0;
    let mut best_d = qd0;
    let mut lab = "CR".to_string();
    for i in 0..9 {
        for k in [-1i32, 1] {
            let mut c = CC0;
            c[i] = poke(CC0[i], k);
            let (qe, qd, h, zs) = sc(&c, &DD0, E0, E1, E2, E3);
            if h > h0 || qd >= qd0 || qe >= qe0 {
                println!("  C[{i}] {k:+} Q={qe} d={qd} hit={h}/6 {zs:?}");
            }
            if h > best_h || (h == best_h && qd > best_d) {
                best_h = h;
                best_d = qd;
                lab = format!("C[{i}] {k:+}");
            }
        }
    }
    for i in 1..10 {
        for k in [-1i32, 1] {
            let mut d = DD0;
            d[i] = poke(DD0[i], k);
            let (qe, qd, h, zs) = sc(&CC0, &d, E0, E1, E2, E3);
            if h > h0 || qd >= qd0 || qe >= qe0 {
                println!("  D[{i}] {k:+} Q={qe} d={qd} hit={h}/6 {zs:?}");
            }
            if h > best_h || (h == best_h && qd > best_d) {
                best_h = h;
                best_d = qd;
                lab = format!("D[{i}] {k:+}");
            }
        }
    }
    let es: [(&str, f64, f64, f64, f64); 8] = [
        ("E0+1", poke(E0, 1), E1, E2, E3),
        ("E0-1", poke(E0, -1), E1, E2, E3),
        ("E1+1", E0, poke(E1, 1), E2, E3),
        ("E1-1", E0, poke(E1, -1), E2, E3),
        ("E2+1", E0, E1, poke(E2, 1), E3),
        ("E2-1", E0, E1, poke(E2, -1), E3),
        ("E3+1", E0, E1, E2, poke(E3, 1)),
        ("E3-1", E0, E1, E2, poke(E3, -1)),
    ];
    for (name, e0, e1, e2, e3) in es {
        let (qe, qd, h, zs) = sc(&CC0, &DD0, e0, e1, e2, e3);
        if h > h0 || qd >= qd0 || qe >= qe0 {
            println!("  {name} Q={qe} d={qd} hit={h}/6 {zs:?}");
        }
        if h > best_h || (h == best_h && qd > best_d) {
            best_h = h;
            best_d = qd;
            lab = name.to_string();
        }
    }
    println!("best-by-hit {lab} d={best_d} hit={best_h}");
}
