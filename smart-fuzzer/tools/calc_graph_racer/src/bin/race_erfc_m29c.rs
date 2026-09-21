//! 5 two-mode misses of 0x210 leftover 29 vs F+k / unmasked / cephes. Not an identity.
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
const MISS: [f64; 5] = [2.4270833333333335, 2.65625, 2.6875, 3.0625, 3.75];

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
fn cody_mask(y: f64, mask: u32) -> f64 {
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
fn d73(z: f64) -> f64 {
    let mut dd = D0;
    dd[7] = poke(D0[7], -3);
    let ye = ef(z.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, 0x210, 0);
    xden = maybe(xden, 0x210, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(dd[i]), CW), &ye, CW);
        xnum = maybe(xnum, 0x210, 2 + 2 * i as u32);
        xden = maybe(xden, 0x210, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(dd[7]), CW),
        CW,
    );
    q = maybe(q, 0x210, 16);
    ext_to_f64(&q, CW)
}
fn signed(g: f64, t: f64) -> String {
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    println!("5 two-mode misses:");
    for &lz in &MISS {
        let Some(r) = rows
            .iter()
            .find(|rr| rr.direct && (rr.z - lz).abs() < 1e-12)
        else {
            println!("  z={lz:.16} MISSING");
            continue;
        };
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(lz);
        let f210 = cody_mask(lz, 0x210);
        let f0 = cody_mask(lz, 0);
        let d = |ff: f64| signed(mul(w, ff), t);
        println!(
            "  z={lz:.16} F={} F+2={} F+3={} F+4={} unmask={} cephes={} D73={}",
            d(f210),
            d(poke(f210, 2)),
            d(poke(f210, 3)),
            d(poke(f210, 4)),
            d(f0),
            d(f::cephes_f(lz)),
            d(d73(lz))
        );
    }
    let mut lows: Vec<(f64, u64)> = Vec::new();
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let g = mul(f::w_rn53(r.z), cody_mask(r.z, 0x210));
        if ulp_distance(g, t).unwrap_or(99) >= 2 && g < t {
            lows.push((r.z, r.qbits));
        }
    }
    let tm = |z: f64, t: f64| {
        let w = f::w_rn53(z);
        let ff = cody_mask(z, 0x210);
        ulp_distance(ru_mul(w.next_up(), ff), t).unwrap_or(99) == 0
            || ulp_distance(mul(w.next_up(), ff.next_up()), t).unwrap_or(99) == 0
    };
    let f3 = |z: f64, t: f64| {
        let w = f::w_rn53(z);
        let ff = poke(cody_mask(z, 0x210), 3);
        ulp_distance(mul(w, ff), t).unwrap_or(99) == 0
    };
    let f2 = |z: f64, t: f64| {
        let w = f::w_rn53(z);
        let ff = poke(cody_mask(z, 0x210), 2);
        ulp_distance(mul(w, ff), t).unwrap_or(99) == 0
    };
    let mut h_tm = 0usize;
    let mut h_u3 = 0usize;
    let mut h_u2 = 0usize;
    for &(z, bits) in &lows {
        let t = f64::from_bits(bits);
        let a = tm(z, t);
        let b = f3(z, t);
        let c = f2(z, t);
        if a {
            h_tm += 1;
        }
        if a || b {
            h_u3 += 1;
        }
        if a || b || c {
            h_u2 += 1;
        }
    }
    println!(
        "leftover two-mode={h_tm}/{} ∪F+3={h_u3}/{} ∪F+2={h_u2}/{}",
        lows.len(),
        lows.len(),
        lows.len()
    );
    let mut d_tm = 0usize;
    let mut d_u3 = 0usize;
    let mut n = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        n += 1;
        let t = f64::from_bits(r.qbits);
        let a = tm(r.z, t);
        let b = f3(r.z, t);
        if a {
            d_tm += 1;
        }
        if a || b {
            d_u3 += 1;
        }
    }
    println!("dmid two-mode={d_tm}/{n} ∪F+3={d_u3} (bar 105)");
}
