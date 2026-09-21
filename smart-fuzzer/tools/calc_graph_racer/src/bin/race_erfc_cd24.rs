//! C/D [2,4) 0x210 DIRECT hard: signed split, Cody P/Q early-cut. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const C0: [f64; 9] = [
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
const D0: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];
const P0: [f64; 6] = [
    0.305326634961232344,
    0.360344899949804439,
    0.125781726111229246,
    0.0160837851487422766,
    6.58749161529837803e-4,
    0.0163153871373020978,
];
const Q0: [f64; 5] = [
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
fn cody_cd(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn cody_pq(y: f64) -> f64 {
    let ye = ef(y.abs());
    let ysq = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
    let mut xnum = ext_mul(&ef(P0[5]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..4 {
        xnum = ext_mul(&ext_add(&xnum, &ef(P0[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(Q0[i]), CW), &ysq, CW);
    }
    let r = ext_div(
        &ext_mul(&ysq, &ext_add(&xnum, &ef(P0[4]), CW), CW),
        &ext_add(&xden, &ef(Q0[4]), CW),
        CW,
    );
    ext_to_f64(&ext_div(&ext_sub(&ef(f::RPINV), &r, CW), &ye, CW), CW)
}
fn qwf(z: f64, ff: f64) -> f64 {
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
    let mut hard: Vec<&f::QRow> = Vec::new();
    let mut hist = [0usize; 6];
    let mut lo = 0usize;
    let mut hi = 0usize;
    println!("0x210 DIRECT hard [2,4):");
    for r in &rows {
        if !r.direct || r.z < 2.0 || r.z >= 4.0 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let g = qwf(r.z, cody_cd(r.z, 0x210));
        let d = ulp_distance(g, t).unwrap_or(99);
        if d >= 2 {
            hard.push(r);
            let b = d.min(5) as usize;
            hist[b] += 1;
            if g < t {
                lo += 1;
            } else {
                hi += 1;
            }
            if d >= 3 {
                println!(
                    "  z={:.16} ulp={d} {}",
                    r.z,
                    if g < t { "low" } else { "high" }
                );
            }
        }
    }
    println!(
        "n={} low={lo} high={hi} ulp2..5+ {} {} {} {}",
        hard.len(),
        hist[2],
        hist[3],
        hist[4],
        hist[5]
    );

    let named: [(&str, Box<dyn Fn(f64) -> f64>); 6] = [
        ("P/Q", Box::new(|z| qwf(z, cody_pq(z)))),
        ("codyF", Box::new(|z| qwf(z, f::cody_erfcx_f(z)))),
        ("derfc0", Box::new(|z| qwf(z, f::nswc_derfc0(z)))),
        ("ccdd", Box::new(|z| qwf(z, f::nswc_ccdd_f(z)))),
        ("cephes_f", Box::new(|z| qwf(z, f::cephes_f(z)))),
        ("libm", Box::new(|z| libm::erfc(z))),
    ];
    for (name, ev) in &named {
        let mut hit_lo = 0usize;
        let mut hit_hi = 0usize;
        let mut u1_lo = 0usize;
        let mut keep24 = 0usize;
        let mut n24 = 0usize;
        for r in &rows {
            if !r.direct || r.z < 2.0 || r.z >= 4.0 {
                continue;
            }
            n24 += 1;
            if ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                keep24 += 1;
            }
        }
        for r in &hard {
            let t = f64::from_bits(r.qbits);
            let g210 = qwf(r.z, cody_cd(r.z, 0x210));
            let d = ulp_distance(ev(r.z), t).unwrap_or(99);
            let is_lo = g210 < t;
            if d == 0 {
                if is_lo {
                    hit_lo += 1;
                } else {
                    hit_hi += 1;
                }
                println!("  HIT {name} z={:.16} {}", r.z, if is_lo { "LOW" } else { "HIGH" });
            } else if d == 1 && is_lo {
                u1_lo += 1;
            }
        }
        println!(
            "{name:10} [2,4) DIRECT {keep24}/{n24}  hard_lo={hit_lo} hard_hi={hit_hi} u1_lo={u1_lo}"
        );
    }

    println!("0x210-then-P/Q dmid scan (bar 105; documented cut 4.0):");
    let mut best = 0usize;
    let mut best_c = 0.0;
    let mut best_mx = 99u64;
    for k in 0..=40 {
        let c = 2.0 + k as f64 * 0.05;
        let mut dmid = 0usize;
        let mut n = 0usize;
        let mut mx = 0u64;
        let mut u4 = 0usize;
        let mut h24 = 0usize;
        for r in &rows {
            if !r.direct || r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            n += 1;
            let ff = if r.z < c {
                cody_cd(r.z, 0x210)
            } else {
                cody_pq(r.z)
            };
            let d = ulp_distance(qwf(r.z, ff), f64::from_bits(r.qbits)).unwrap_or(99);
            if d == 0 {
                dmid += 1;
                if r.z >= 2.0 {
                    h24 += 1;
                }
            } else {
                mx = mx.max(d);
                if d >= 4 {
                    u4 += 1;
                }
            }
        }
        let named_cut = (c - c.round()).abs() < 1e-12 || (c - 3.75).abs() < 1e-12 || dmid >= 105;
        if named_cut {
            println!("  cut={c:.2} dmid={dmid}/{n} maxd={mx} u4={u4} [2,4)ex={h24}");
        }
        if dmid > best || (dmid == best && mx < best_mx) {
            best = dmid;
            best_c = c;
            best_mx = mx;
        }
    }
    println!("  BEST dmid {best} maxd={best_mx} @ {best_c:.2}");
}
