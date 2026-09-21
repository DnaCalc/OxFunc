//! fdlibm RB/SB ±1 vs 6 leftover-low. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research::excel_exp;
use std::env;

const SIX: [f64; 6] = [
    4.0520833333333330,
    4.0625000000000000,
    5.2395833333333330,
    5.2708333333333330,
    5.3333333333333330,
    6.0000000000000000,
];
const RB0: [f64; 7] = [
    -9.86494292470009928597e-03,
    -7.99283237680523006574e-01,
    -1.77579549177547519889e+01,
    -1.60636384855821916062e+02,
    -6.37566443368389627722e+02,
    -1.02509513161107724954e+03,
    -4.83519191608651397019e+02,
];
const SB0: [f64; 7] = [
    3.03380607434824582924e+01,
    3.25792512996573918826e+02,
    1.53672958608443695994e+03,
    3.19985821950859553908e+03,
    2.55305040643316442583e+03,
    4.74528541206955367215e+02,
    -2.24409524465858183362e+01,
];

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
fn fd_rb(ax: f64, rb: &[f64; 7], sb: &[f64; 7]) -> f64 {
    let s = 1.0 / (ax * ax);
    let rr = horner(rb, s);
    let ss = 1.0 + s * horner(sb, s);
    let z = f64::from_bits(ax.to_bits() & 0xffff_ffff_0000_0000);
    let q = excel_exp(-z * z - 0.5625) * excel_exp((z - ax) * (z + ax) + rr / ss) / ax;
    let w = excel_exp(-(ax * ax));
    if w == 0.0 {
        f64::NAN
    } else {
        q / w
    }
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
    let sc = |rb: &[f64; 7], sb: &[f64; 7]| {
        let mut qd = 0usize;
        let mut qe = 0usize;
        let mut hit = 0usize;
        let mut zs = Vec::new();
        for row in rows.iter().filter(|rr| rr.z >= 4.0) {
            let g = qw(row.z, fd_rb(row.z.abs(), rb, sb));
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
            if ulp_distance(qw(lz, fd_rb(lz.abs(), rb, sb)), f64::from_bits(r.qbits)).unwrap_or(99)
                == 0
            {
                hit += 1;
                zs.push(lz);
            }
        }
        (qe, qd, hit, zs)
    };
    let (qe0, qd0, h0, _) = sc(&RB0, &SB0);
    println!("fdlibm RB/SB CR Q={qe0} d={qd0} hit={h0}/6 (bar 0x5005 1572/53)");
    println!("±1 (print hit>h0 or d>=qd0 or Q>=qe0):");
    let mut best_h = h0;
    let mut best_d = qd0;
    let mut lab = "CR".to_string();
    for i in 0..7 {
        for k in [-1i32, 1] {
            let mut r = RB0;
            r[i] = poke(RB0[i], k);
            let (qe, qd, h, zs) = sc(&r, &SB0);
            if h > h0 || qd >= qd0 || qe >= qe0 {
                println!("  R[{i}] {k:+} Q={qe} d={qd} hit={h}/6 {zs:?}");
            }
            if h > best_h || (h == best_h && qd > best_d) {
                best_h = h;
                best_d = qd;
                lab = format!("R[{i}] {k:+}");
            }
        }
        for k in [-1i32, 1] {
            let mut s = SB0;
            s[i] = poke(SB0[i], k);
            let (qe, qd, h, zs) = sc(&RB0, &s);
            if h > h0 || qd >= qd0 || qe >= qe0 {
                println!("  S[{i}] {k:+} Q={qe} d={qd} hit={h}/6 {zs:?}");
            }
            if h > best_h || (h == best_h && qd > best_d) {
                best_h = h;
                best_d = qd;
                lab = format!("S[{i}] {k:+}");
            }
        }
    }
    println!("best-by-hit {lab} d={best_d} hit={best_h}");
}
