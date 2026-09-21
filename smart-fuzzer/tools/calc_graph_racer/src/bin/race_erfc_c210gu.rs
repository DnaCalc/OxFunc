//! Grid census of 0x210 and 114 1-ULP DIRECT [0.5,4). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const MASK: u32 = 0x210;
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
fn cody(y: f64, c: &[f64; 9]) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, MASK, 0);
    xden = maybe(xden, MASK, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, MASK, 2 + 2 * i as u32);
        xden = maybe(xden, MASK, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, MASK, 16);
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
fn grid(z: f64) -> &'static str {
    let bits = z.to_bits();
    if z.fract() == 0.0 {
        "int"
    } else if (z * 2.0).fract() == 0.0 {
        "dyad2"
    } else if (z * 16.0).fract() == 0.0 {
        "dyad16"
    } else if (z * 48.0 - (z * 48.0).round()).abs() < 1e-12 {
        "48"
    } else if (z * 96.0 - (z * 96.0).round()).abs() < 1e-12 {
        "96"
    } else if bits & 0xffff == 0x5555 || (bits & 0xfff) == 0x555 {
        "555"
    } else {
        "other"
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut c114 = C0;
    c114[3] = poke(C0[3], -1);
    c114[4] = poke(C0[4], 1);
    c114[5] = poke(C0[5], -1);
    c114[6] = poke(C0[6], 1);
    let mut g210: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    let mut g114: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    println!("1-ULP grids:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let g = mul(f::w_rn53(r.z), cody(r.z, &C0));
        let d = ulp_distance(g, t).unwrap_or(99);
        if d == 1 {
            *g210.entry(grid(r.z)).or_insert(0) += 1;
        }
        let g2 = mul(f::w_rn53(r.z), cody(r.z, &c114));
        let d2 = ulp_distance(g2, t).unwrap_or(99);
        if d2 == 1 {
            *g114.entry(grid(r.z)).or_insert(0) += 1;
        }
    }
    print!("0x210 1-ULP");
    for (k, v) in &g210 {
        print!(" {k}:{v}");
    }
    print!("\n114 1-ULP");
    for (k, v) in &g114 {
        print!(" {k}:{v}");
    }
    println!();
}
