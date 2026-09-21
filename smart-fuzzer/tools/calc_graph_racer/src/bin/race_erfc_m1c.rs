//! 0x210 CR vs F+1 z-cuts on DIRECT [0.5,4). Not an identity.
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let dirs: Vec<&f::QRow> = rows
        .iter()
        .filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0)
        .collect();
    let mut both = 0usize;
    let mut only_c = 0usize;
    let mut only_f = 0usize;
    for r in &dirs {
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let ff = cody(r.z);
        let ec = ulp_distance(mul(w, ff), t).unwrap_or(99) == 0;
        let ef1 = ulp_distance(mul(w, poke(ff, 1)), t).unwrap_or(99) == 0;
        match (ec, ef1) {
            (true, true) => both += 1,
            (true, false) => only_c += 1,
            (false, true) => only_f += 1,
            _ => {}
        }
    }
    println!(
        "DIRECT [0.5,4) CR vs F+1 both={both} only_CR={only_c} only_F1={only_f} union={} n={}",
        both + only_c + only_f,
        dirs.len()
    );
    let cuts = [0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0, 3.5];
    println!("z-cut CR-then-F+1 / F+1-then-CR:");
    let mut best_a = 0usize;
    let mut best_ac = 0.0;
    for &c in &cuts {
        let mut a = 0usize;
        let mut b = 0usize;
        for r in &dirs {
            let t = f64::from_bits(r.qbits);
            let w = f::w_rn53(r.z);
            let ff = cody(r.z);
            let ec = ulp_distance(mul(w, ff), t).unwrap_or(99) == 0;
            let ef1 = ulp_distance(mul(w, poke(ff, 1)), t).unwrap_or(99) == 0;
            if r.z < c {
                if ec {
                    a += 1;
                }
                if ef1 {
                    b += 1;
                }
            } else {
                if ef1 {
                    a += 1;
                }
                if ec {
                    b += 1;
                }
            }
        }
        if a > best_a {
            best_a = a;
            best_ac = c;
        }
        println!("  cut={c} CR-then-F1={a} F1-then-CR={b}");
    }
    println!("best CR-then-F1={best_a} @{best_ac} (bar dmid 105)");
}
