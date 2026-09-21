//! Q-mid j3432 leftover signed-ULP and next_down of w*F. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody(y: f64, c: &[f64; 9], d: &[f64; 8], mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(d[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
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
    let mut jc = C0;
    jc[0] = poke(C0[0], 4);
    jc[1] = poke(C0[1], 1);
    jc[4] = poke(C0[4], -1);
    let mut jd = D0;
    jd[0] = poke(D0[0], -1);
    jd[4] = poke(D0[4], -1);
    println!("Q-mid leftover signed and last-store nudge (bars j3432 3432 / 0x74 3398):");
    for (name, ev) in [
        ("j3432", Box::new(|z: f64| qw(z, cody(z, &jc, &jd, 0))) as Box<dyn Fn(f64) -> f64>),
        ("j+0x20", Box::new(|z| qw(z, cody(z, &jc, &jd, 0x20)))),
        ("0x74", Box::new(|z| qw(z, cody(z, &C0, &D0, 0x74)))),
        ("CR", Box::new(|z| qw(z, cody(z, &C0, &D0, 0)))),
        (
            "next_down(j)",
            Box::new(|z| qw(z, cody(z, &jc, &jd, 0)).next_down()),
        ),
        (
            "next_up(j)",
            Box::new(|z| qw(z, cody(z, &jc, &jd, 0)).next_up()),
        ),
        (
            "next_down(0x74)",
            Box::new(|z| qw(z, cody(z, &C0, &D0, 0x74)).next_down()),
        ),
        (
            "w*next_up(Fj)",
            Box::new(|z| qw(z, cody(z, &jc, &jd, 0).next_up())),
        ),
        (
            "w*next_down(Fj)",
            Box::new(|z| qw(z, cody(z, &jc, &jd, 0).next_down())),
        ),
        (
            "w*next_up(F74)",
            Box::new(|z| qw(z, cody(z, &C0, &D0, 0x74).next_up())),
        ),
    ] {
        let mut ex = 0usize;
        let mut dd = 0usize;
        let mut mx = 0u64;
        let mut hi = 0usize;
        let mut lo = 0usize;
        let mut h1 = 0usize;
        let mut l1 = 0usize;
        let mut n = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let t = f64::from_bits(r.qbits);
            let g = ev(r.z);
            let d = ulp_distance(g, t).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
                if r.direct {
                    dd += 1;
                }
            } else {
                mx = mx.max(d);
                if g > t {
                    hi += 1;
                    if d == 1 {
                        h1 += 1;
                    }
                } else {
                    lo += 1;
                    if d == 1 {
                        l1 += 1;
                    }
                }
            }
        }
        println!(
            "  {name:16} Q {ex}/{n} d={dd} max={mx}  left>={hi} (ulp1={h1}) left<={lo} (ulp1={l1})"
        );
    }
}
