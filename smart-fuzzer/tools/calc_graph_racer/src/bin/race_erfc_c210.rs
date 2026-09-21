//! 0x210 x87 then native codyF / cephes_f cut by dmid. Not an identity.
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
fn cody(y: f64, mask: u32) -> f64 {
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
    let dmid_of = |ff: &dyn Fn(f64) -> f64| {
        let mut dmid = 0usize;
        let mut n = 0usize;
        let mut mx = 0u64;
        let mut u4 = 0usize;
        for r in &rows {
            if !r.direct || r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            n += 1;
            let d = ulp_distance(qwf(r.z, ff(r.z)), f64::from_bits(r.qbits)).unwrap_or(99);
            if d == 0 {
                dmid += 1;
            } else {
                mx = mx.max(d);
                if d >= 4 {
                    u4 += 1;
                }
            }
        }
        (dmid, n, mx, u4)
    };
    println!("bars 0x210 dmid 105 maxd 4 u4=3");
    for (name, hi) in [
        ("codyF", Box::new(|z| f::cody_erfcx_f(z)) as Box<dyn Fn(f64) -> f64>),
        ("cephes_f", Box::new(|z| f::cephes_f(z))),
        ("derfc0", Box::new(|z| f::nswc_derfc0(z))),
    ] {
        let mut best = 0usize;
        let mut best_c = 0.0;
        let mut best_mx = 99u64;
        let mut best_u4 = 99usize;
        println!("0x210-then-{name}:");
        for k in 0..=30 {
            let c = 0.5 + k as f64 * 0.1;
            let (dmid, n, mx, u4) = dmid_of(&|z| {
                if z < c {
                    cody(z, 0x210)
                } else {
                    hi(z)
                }
            });
            if (c - c.round()).abs() < 1e-12 || dmid >= 105 {
                println!("  cut={c:.1} dmid={dmid}/{n} maxd={mx} u4={u4}");
            }
            if dmid > best || (dmid == best && mx < best_mx) {
                best = dmid;
                best_c = c;
                best_mx = mx;
                best_u4 = u4;
            }
        }
        println!("  BEST dmid {best} maxd={best_mx} u4={best_u4} @ {best_c:.1}");
    }
}
