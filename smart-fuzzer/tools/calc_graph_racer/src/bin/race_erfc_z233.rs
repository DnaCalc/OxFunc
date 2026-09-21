//! pin 2.333333 0x210-F tmd extra and leftover two-mode extras. Not an identity.
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
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn sd(g: f64, t: f64) -> String {
    let d = ulp_distance(g, t).unwrap_or(99);
    let s = if g < t {
        "L"
    } else if g > t {
        "H"
    } else {
        "="
    };
    format!("{d}{s}")
}
fn k_of(t: f64, w: f64, ff: f64) -> Option<i32> {
    for ak in 0i32..=8 {
        for &s in &[-1i32, 1] {
            let k = if ak == 0 { 0 } else { s * ak };
            if ak == 0 && s < 0 {
                continue;
            }
            if ulp_distance(mul(w, poke(ff, k)), t).unwrap_or(99) == 0 {
                return Some(k);
            }
            if ak == 0 {
                break;
            }
        }
    }
    None
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut c114 = C0;
    c114[3] = poke(C0[3], -1);
    c114[4] = poke(C0[4], 1);
    c114[5] = poke(C0[5], -1);
    c114[6] = poke(C0[6], 1);
    let want = [
        2.3333333333333335,
        2.3958333333333335,
        3.2708333333333335,
        3.46875,
        0.875,
        0.8749999999999999,
        3.875,
        0.5000000000000002,
        1.71875,
        1.3333333333333333,
        1.125,
        3.5,
        0.53125,
        2.3020833333333335,
        1.625,
        3.3020833333333335,
        3.3958333333333335,
        0.84375,
        0.8437499999999999,
        2.6458333333333335,
        1.1145833333333333,
        3.5625,
        3.75,
        2.3645833333333335,
        2.375,
        2.0520833333333335,
        2.1145833333333335,
        2.5,
        2.5208333333333335,
        2.3125,
        2.1875,
        2.4583333333333335,
        1.9895833333333333,
        2.125,
        2.1770833333333335,
        1.8333333333333333,
        1.28125,
        2.75,
        1.875,
        3.40625,
        2.3958333333333335,
        2.65625,
        2.6875,
        3.0625,
        0.8645833333333333,
        0.8958333333333333,
        2.53125,
        2.5625,
        2.4270833333333335,
        2.6770833333333335,
        1.125,
        1.625,
        3.1770833333333335,
        3.5,
        3.65625,
        3.78125,
        0.6145833333333333,
        0.75,
        1.34375,
        1.78125,
        3.28125,
        3.6770833333333335,
        3.96875,
        1.1458333333333333,
        1.15625,
        2.46875,
        2.71875,
        2.9583333333333335,
        1.7083333333333333,
        2.6145833333333335,
        2.625,
        2.9895833333333335,
        3.0,
        3.4375,
        3.6458333333333335,
        1.46875,
        1.0625,
        1.2708333333333333,
        2.25,
        3.15625,
        2.7083333333333335,
        2.8020833333333335,
        2.8333333333333335,
        0.8125,
        1.0520833333333333,
        1.2395833333333333,
        1.3125,
        0.7708333333333334,
        0.78125,
        0.8020833333333334,
        0.9270833333333334,
        0.96875,
        2.28125,
        2.59375,
        2.9375,
        3.03125,
        3.0520833333333335,
        2.2708333333333335,
        2.2395833333333335,
        0.9895833333333334,
        1.6458333333333333,
        2.7708333333333335,
        3.15625,
        1.5,
        1.46875,
        1.53125,
        2.40625,
        2.4375,
        2.46875,
        2.4583333333333335,
    ];
    println!("pin leftover two-mode extras unmask vs 0x210/114/0x74:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !want.iter().any(|&z| (r.z - z).abs() < 1e-14) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        let gu = mul(w, fu);
        let g210 = mul(w, f210);
        println!(
            "z={:.16} bits={:#x} w={:#x}",
            z,
            z.to_bits(),
            w.to_bits()
        );
        println!(
            "  Q unmask {} k={:?} 0x210 {} k={:?} 0x74 {} 114 {}",
            sd(gu, t),
            k_of(t, w, fu),
            sd(g210, t),
            k_of(t, w, f210),
            sd(mul(w, f74), t),
            sd(mul(w, f114), t)
        );
        println!(
            "  unmask tmu {} tmd {} last-mul_up {} last-mul_dn {} w-1 {} F-1 {} w+1 {} F+1 {}",
            sd(mul(w.next_up(), fu.next_up()), t),
            sd(mul(w.next_down(), fu.next_down()), t),
            sd(gu.next_up(), t),
            sd(gu.next_down(), t),
            sd(mul(w.next_down(), fu), t),
            sd(mul(w, fu.next_down()), t),
            sd(mul(w.next_up(), fu), t),
            sd(mul(w, fu.next_up()), t)
        );
        println!(
            "  0x210 tmu {} tmd {} last-mul_up {} last-mul_dn {} w-1 {} F-1 {} F-2 {} F-3 {}",
            sd(mul(w.next_up(), f210.next_up()), t),
            sd(mul(w.next_down(), f210.next_down()), t),
            sd(g210.next_up(), t),
            sd(g210.next_down(), t),
            sd(mul(w.next_down(), f210), t),
            sd(mul(w, f210.next_down()), t),
            sd(mul(w, poke(f210, -2)), t),
            sd(mul(w, poke(f210, -3)), t)
        );
        println!(
            "  114 tmu {} tmd {} 0x74 tmu {} tmd {}",
            sd(mul(w.next_up(), f114.next_up()), t),
            sd(mul(w.next_down(), f114.next_down()), t),
            sd(mul(w.next_up(), f74.next_up()), t),
            sd(mul(w.next_down(), f74.next_down()), t)
        );
        print!("  unmask F last-store");
        for k in -4i32..=4 {
            print!(" k={k}:{}", sd(mul(w, poke(fu, k)), t));
        }
        println!();
        print!("  0x210 F last-store");
        for k in -4i32..=4 {
            print!(" k={k}:{}", sd(mul(w, poke(f210, k)), t));
        }
        println!();
        print!("  unmask prod last-mul");
        for k in -4i32..=4 {
            print!(" k={k}:{}", sd(poke(gu, k), t));
        }
        println!();
        print!("  0x210 prod last-mul");
        for k in -4i32..=4 {
            print!(" k={k}:{}", sd(poke(g210, k), t));
        }
        println!();
    }
}
