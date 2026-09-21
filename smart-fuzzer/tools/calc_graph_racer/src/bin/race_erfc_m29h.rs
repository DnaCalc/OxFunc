//! 0x210 leftover-high: F−k last-store. Not an identity.
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
fn cody(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, 0x210, 0);
    xden = maybe(xden, 0x210, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, 0x210, 2 + 2 * i as u32);
        xden = maybe(xden, 0x210, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, 0x210, 16);
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
fn hitk(z: f64, t: f64, k: i32) -> bool {
    ulp_distance(mul(f::w_rn53(z), poke(cody(z), k)), t).unwrap_or(99) == 0
}
fn kind(z: f64) -> &'static str {
    let m = z.to_bits() & ((1u64 << 52) - 1);
    let hex = format!("{m:013x}");
    if m.trailing_zeros() >= 20 {
        "dyad"
    } else if hex.contains("555") || hex.contains("aaa") {
        "555"
    } else {
        "other"
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut highs: Vec<(f64, u64, u64)> = Vec::new();
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let g = mul(f::w_rn53(r.z), cody(r.z));
        let d = ulp_distance(g, t).unwrap_or(99);
        if d >= 2 && g > t {
            highs.push((r.z, r.qbits, d));
        }
    }
    println!("0x210 leftover-high n={}", highs.len());
    let mut h = [0usize; 5];
    let mut hu = 0usize;
    let mut n_d = 0usize;
    let mut n_5 = 0usize;
    for &(z, bits, d) in &highs {
        let t = f64::from_bits(bits);
        let knd = kind(z);
        match knd {
            "dyad" => n_d += 1,
            "555" => n_5 += 1,
            _ => {}
        }
        let mut any = false;
        print!("  z={z:.16} kind={knd} ulp={d}");
        for k in 1..=4 {
            let hit = hitk(z, t, -k);
            if hit {
                h[k as usize] += 1;
                any = true;
            }
            print!(" F-{k}={hit}");
        }
        if any {
            hu += 1;
        }
        println!(" any={any}");
    }
    println!(
        "kind dyad={n_d} 555={n_5} leftover F-1={}/{} F-2={}/{} F-3={}/{} F-4={}/{} union={}/{}",
        h[1],
        highs.len(),
        h[2],
        highs.len(),
        h[3],
        highs.len(),
        h[4],
        highs.len(),
        hu,
        highs.len()
    );
    let mut d0 = 0usize;
    let mut du = 0usize;
    let mut n = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        n += 1;
        let t = f64::from_bits(r.qbits);
        if hitk(r.z, t, 0) {
            d0 += 1;
        }
        if (1..=4).any(|k| hitk(r.z, t, -k)) {
            du += 1;
        }
    }
    println!("dmid CR={d0}/{n} F-1..4={du} (bar 105)");
}
