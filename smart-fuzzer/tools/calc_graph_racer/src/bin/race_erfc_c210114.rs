//! 0x210 leftover vs 114 envelope buckets DIRECT [0.5,4). Not an identity.
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
fn cody(y: f64, c: &[f64; 9], d: &[f64; 8]) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, MASK, 0);
    xden = maybe(xden, MASK, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
        xnum = maybe(xnum, MASK, 2 + 2 * i as u32);
        xden = maybe(xden, MASK, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(d[7]), CW),
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
fn bucket(g: f64, t: f64) -> &'static str {
    let d = ulp_distance(g, t).unwrap_or(99);
    if d == 0 {
        "fused"
    } else if d == 1 {
        "1-ULP"
    } else if g < t {
        "leftover-low"
    } else {
        "leftover-high"
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let dirs: Vec<&f::QRow> = rows
        .iter()
        .filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0)
        .collect();
    let mut c114 = C0;
    c114[3] = poke(C0[3], -1);
    c114[4] = poke(C0[4], 1);
    c114[5] = poke(C0[5], -1);
    c114[6] = poke(C0[6], 1);
    let n = dirs.len();
    println!("DIRECT [0.5,4) n={n} 0x210 leftover vs 114 buckets:");
    let mut lo = 0usize;
    let mut hi = 0usize;
    let mut u1 = 0usize;
    let mut lo_b = [0usize; 4];
    let mut hi_b = [0usize; 4];
    let mut u1_b = [0usize; 4];
    let idx = |s: &str| match s {
        "fused" => 0,
        "1-ULP" => 1,
        "leftover-low" => 2,
        _ => 3,
    };
    println!("0x210 leftover-low as 114:");
    for r in &dirs {
        let t = f64::from_bits(r.qbits);
        let g210 = mul(f::w_rn53(r.z), cody(r.z, &C0, &D0));
        let d210 = ulp_distance(g210, t).unwrap_or(99);
        let g114 = mul(f::w_rn53(r.z), cody(r.z, &c114, &D0));
        let b = bucket(g114, t);
        if d210 >= 2 && g210 < t {
            lo += 1;
            lo_b[idx(b)] += 1;
            println!("  LOW z={:.16} d210={d210} 114={b}", r.z);
        } else if d210 >= 2 && g210 > t {
            hi += 1;
            hi_b[idx(b)] += 1;
        } else if d210 == 1 {
            u1 += 1;
            u1_b[idx(b)] += 1;
        }
    }
    let names = ["fused", "1-ULP", "leftover-low", "leftover-high"];
    print!("0x210 leftover-low n={lo} as 114");
    for (i, c) in lo_b.iter().enumerate() {
        if *c > 0 {
            print!(" {}:{}", names[i], c);
        }
    }
    print!("\n0x210 leftover-high n={hi} as 114");
    for (i, c) in hi_b.iter().enumerate() {
        if *c > 0 {
            print!(" {}:{}", names[i], c);
        }
    }
    print!("\n0x210 1-ULP n={u1} as 114");
    for (i, c) in u1_b.iter().enumerate() {
        if *c > 0 {
            print!(" {}:{}", names[i], c);
        }
    }
    println!();
    let mut lo114 = 0usize;
    let mut lo114_b = [0usize; 4];
    println!("114 leftover-low as 0x210:");
    for r in &dirs {
        let t = f64::from_bits(r.qbits);
        let g114 = mul(f::w_rn53(r.z), cody(r.z, &c114, &D0));
        let d114 = ulp_distance(g114, t).unwrap_or(99);
        if !(d114 >= 2 && g114 < t) {
            continue;
        }
        lo114 += 1;
        let g210 = mul(f::w_rn53(r.z), cody(r.z, &C0, &D0));
        let b = bucket(g210, t);
        lo114_b[idx(b)] += 1;
        println!("  LOW z={:.16} d114={d114} 0x210={b}", r.z);
    }
    print!("114 leftover-low n={lo114} as 0x210");
    for (i, c) in lo114_b.iter().enumerate() {
        if *c > 0 {
            print!(" {}:{}", names[i], c);
        }
    }
    println!();
}
