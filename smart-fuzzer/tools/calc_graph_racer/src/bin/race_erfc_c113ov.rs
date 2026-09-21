//! 0x210+C[4]-1 D[5]-1 vs unmask C[1]-1 C[3]-1 D[5]-1. Not an identity.
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
fn hit(z: f64, t: f64, c: &[f64; 9], d: &[f64; 8], mask: u32) -> bool {
    ulp_distance(mul(f::w_rn53(z), cody(z, c, d, mask)), t).unwrap_or(99) == 0
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let dirs: Vec<&f::QRow> = rows
        .iter()
        .filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0)
        .collect();
    let n = dirs.len();
    let mut c113 = C0;
    let mut d113 = D0;
    c113[4] = poke(C0[4], -1);
    d113[5] = poke(D0[5], -1);
    let mut c107 = C0;
    let mut d107 = D0;
    c107[1] = poke(C0[1], -1);
    c107[3] = poke(C0[3], -1);
    d107[5] = poke(D0[5], -1);
    let mut both = 0usize;
    let mut only_a = 0usize;
    let mut only_b = 0usize;
    let mut na = 0usize;
    let mut nb = 0usize;
    let mut n210 = 0usize;
    println!("DIRECT [0.5,4) n={n} 113 vs unmask 107:");
    for r in &dirs {
        let t = f64::from_bits(r.qbits);
        let ea = hit(r.z, t, &c113, &d113, 0x210);
        let eb = hit(r.z, t, &c107, &d107, 0);
        let e210 = hit(r.z, t, &C0, &D0, 0x210);
        if ea {
            na += 1;
        }
        if eb {
            nb += 1;
        }
        if e210 {
            n210 += 1;
        }
        match (ea, eb) {
            (true, true) => both += 1,
            (true, false) => only_a += 1,
            (false, true) => only_b += 1,
            _ => {}
        }
    }
    println!(
        "113={na} 107={nb} 0x210={n210} both={both} only_113={only_a} only_107={only_b} union={}",
        both + only_a + only_b
    );
}
