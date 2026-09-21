//! DIRECT rows that need C[0]'s 18th digit vs 15-digit parse. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const C0TAB: [f64; 9] = [
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
fn cody(y: f64, c0: f64) -> f64 {
    let ye = ef(y.abs());
    let mut c = C0TAB;
    c[0] = c0;
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, 0x210, 0);
    xden = maybe(xden, 0x210, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, 0x210, 2 + 2 * i as u32);
        xden = maybe(xden, 0x210, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, 0x210, 16);
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
    let c18: f64 = 0.564188496988670089;
    let c15: f64 = "0.564188496988670".parse().unwrap();
    println!(
        "C0_18={:#x} C0_15={:#x} ulp={}",
        c18.to_bits(),
        c15.to_bits(),
        ulp_distance(c15, c18).unwrap_or(99)
    );
    println!("DIRECT z in [0.5,4) where 18-digit exact and 15-digit not:");
    let mut g18 = 0usize;
    let mut g15 = 0usize;
    let mut n = 0usize;
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        n += 1;
        let t = f64::from_bits(r.qbits);
        let d18 = ulp_distance(qwf(r.z, cody(r.z, c18)), t).unwrap_or(99);
        let d15 = ulp_distance(qwf(r.z, cody(r.z, c15)), t).unwrap_or(99);
        if d18 == 0 {
            g18 += 1;
        }
        if d15 == 0 {
            g15 += 1;
        }
        if d18 == 0 && d15 != 0 {
            println!(
                "  GAIN z={:.16} 15ulp={d15} band={}",
                r.z,
                if r.z < 1.0 {
                    "[0.5,1)"
                } else if r.z < 2.0 {
                    "[1,2)"
                } else {
                    "[2,4)"
                }
            );
        }
        if d15 == 0 && d18 != 0 {
            println!("  LOSE z={:.16} 18ulp={d18}", r.z);
        }
    }
    println!("dmid 18={g18}/{n} 15={g15}/{n}");
}
