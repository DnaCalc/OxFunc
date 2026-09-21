//! NSWC PQR on [0.5,2) DIRECT + compose with 0x210 [2,4). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
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
    let band = |lo: f64, hi: f64, ff: &dyn Fn(f64) -> f64| {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut mx = 0u64;
        for r in &rows {
            if !r.direct || r.z < lo || r.z >= hi {
                continue;
            }
            n += 1;
            let d = ulp_distance(qwf(r.z, ff(r.z)), f64::from_bits(r.qbits)).unwrap_or(99);
            if d == 0 {
                ex += 1;
            } else {
                mx = mx.max(d);
            }
        }
        (ex, n, mx)
    };
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 4] = [
        ("0x210", Box::new(|z| cody_cd(z, 0x210))),
        ("pqr", Box::new(|z| f::nswc_pqr_f(z))),
        ("derfc0", Box::new(|z| f::nswc_derfc0(z))),
        ("pqr t-pub", Box::new(|z| f::nswc_t_published(z))),
    ];
    println!("DIRECT by band:");
    for (name, ev) in &graphs {
        let a = band(0.5, 1.0, ev);
        let b = band(1.0, 2.0, ev);
        let c = band(2.0, 4.0, ev);
        let d = band(0.5, 4.0, ev);
        println!(
            "  {name:10} [0.5,1) {}/{} [1,2) {}/{} [2,4) {}/{} all {}/{} maxd={}",
            a.0, a.1, b.0, b.1, c.0, c.1, d.0, d.1, d.2
        );
    }
    println!("compose PQR below cut else 0x210 (documented 2.0):");
    let mut best = 0usize;
    let mut best_c = 0.0;
    for k in 0..=30 {
        let c = 0.5 + k as f64 * 0.05;
        let mut dmid = 0usize;
        let mut n = 0usize;
        let mut mx = 0u64;
        for r in &rows {
            if !r.direct || r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            n += 1;
            let ff = if r.z < c {
                f::nswc_pqr_f(r.z)
            } else {
                cody_cd(r.z, 0x210)
            };
            let d = ulp_distance(qwf(r.z, ff), f64::from_bits(r.qbits)).unwrap_or(99);
            if d == 0 {
                dmid += 1;
            } else {
                mx = mx.max(d);
            }
        }
        if (c - 1.0).abs() < 1e-12 || (c - 1.5).abs() < 1e-12 || (c - 2.0).abs() < 1e-12 || dmid >= 105
        {
            println!("  cut={c:.2} dmid={dmid}/{n} maxd={mx}");
        }
        if dmid > best {
            best = dmid;
            best_c = c;
        }
    }
    println!("  BEST dmid {best} @ {best_c:.2} (bar 105)");
}
