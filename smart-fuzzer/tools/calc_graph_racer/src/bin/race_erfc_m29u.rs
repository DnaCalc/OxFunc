//! 0x210 leftover-low 29: RU last-mul ∪ up(w)×up(F). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const CW_RU: u16 = CW_PC64_RN | 0x0800;
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
fn ru_mul(w: f64, ff: f64) -> f64 {
    let v = ext_to_f64(&ext_mul(&ef(w), &ef(ff), CW_RU), CW_RU);
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut lows: Vec<(f64, u64)> = Vec::new();
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let g = mul(f::w_rn53(r.z), cody(r.z));
        let d = ulp_distance(g, t).unwrap_or(99);
        if d >= 2 && g < t {
            lows.push((r.z, r.qbits));
        }
    }
    println!("0x210 leftover-low n={}", lows.len());
    let mut hit_ru = 0usize;
    let mut hit_tf = 0usize;
    let mut hit_wup = 0usize;
    let mut hit_fup = 0usize;
    let mut hit_u = 0usize;
    for &(z, bits) in &lows {
        let t = f64::from_bits(bits);
        let w = f::w_rn53(z);
        let ff = cody(z);
        let eru = ulp_distance(ru_mul(w.next_up(), ff), t).unwrap_or(99) == 0;
        let etf = ulp_distance(mul(w.next_up(), ff.next_up()), t).unwrap_or(99) == 0;
        let ew = ulp_distance(mul(w.next_up(), ff), t).unwrap_or(99) == 0;
        let efup = ulp_distance(mul(w, ff.next_up()), t).unwrap_or(99) == 0;
        if eru {
            hit_ru += 1;
        }
        if etf {
            hit_tf += 1;
        }
        if ew {
            hit_wup += 1;
        }
        if efup {
            hit_fup += 1;
        }
        if eru || etf {
            hit_u += 1;
        }
        println!(
            "  z={z:.16} RU={eru} TF={etf} upw={ew} upF={efup} any={}",
            eru || etf
        );
    }
    println!(
        "leftover RU={hit_ru}/{} TF={hit_tf}/{} upw={hit_wup}/{} upF={hit_fup}/{} union={hit_u}/{}",
        lows.len(),
        lows.len(),
        lows.len(),
        lows.len(),
        lows.len()
    );
    let mut d_cr = 0usize;
    let mut d_ru = 0usize;
    let mut d_tf = 0usize;
    let mut d_u = 0usize;
    let mut n = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        n += 1;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let ff = cody(r.z);
        let cr = ulp_distance(mul(w, ff), t).unwrap_or(99) == 0;
        let eru = ulp_distance(ru_mul(w.next_up(), ff), t).unwrap_or(99) == 0;
        let etf = ulp_distance(mul(w.next_up(), ff.next_up()), t).unwrap_or(99) == 0;
        if cr {
            d_cr += 1;
        }
        if eru {
            d_ru += 1;
        }
        if etf {
            d_tf += 1;
        }
        if eru || etf {
            d_u += 1;
        }
    }
    println!("dmid CR={d_cr}/{n} RU={d_ru} TF={d_tf} union={d_u} (bar 105)");
}
