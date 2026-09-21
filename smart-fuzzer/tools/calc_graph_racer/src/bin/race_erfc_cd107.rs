//! C[1]−1 C[3]−1 D[5]−1 vs 0x210 leftover and pins. Not an identity.
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
const PINS: [f64; 5] = [0.5, 0.75, 1.28125, 1.875, 2.125];

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
fn cody(y: f64, c: &[f64; 9], d: &[f64; 8]) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ef(c[7]), CW),
            &ext_add(&xden, &ef(d[7]), CW),
            CW,
        ),
        CW,
    )
}
fn cody210(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        let bn = 2 + 2 * i;
        let bd = 3 + 2 * i;
        if 0x210u32 & (1u32 << bn) != 0 {
            xnum = ef(ext_to_f64(&xnum, CW));
        }
        if 0x210u32 & (1u32 << bd) != 0 {
            xden = ef(ext_to_f64(&xden, CW));
        }
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ef(C0[7]), CW),
            &ext_add(&xden, &ef(D0[7]), CW),
            CW,
        ),
        CW,
    )
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
    let mut c = C0;
    let mut d = D0;
    c[1] = poke(C0[1], -1);
    c[3] = poke(C0[3], -1);
    d[5] = poke(D0[5], -1);
    let mut both = 0usize;
    let mut only_j = 0usize;
    let mut only_m = 0usize;
    let mut nj = 0usize;
    let mut nm = 0usize;
    let mut nlo = 0usize;
    let mut hit_lo = 0usize;
    let mut nhi = 0usize;
    let mut hit_hi = 0usize;
    println!("C[1]-1 C[3]-1 D[5]-1 vs 0x210:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let gj = mul(w, cody(r.z, &c, &d));
        let gm = mul(w, cody210(r.z));
        let ej = ulp_distance(gj, t).unwrap_or(99) == 0;
        let em = ulp_distance(gm, t).unwrap_or(99) == 0;
        if ej {
            nj += 1;
        }
        if em {
            nm += 1;
        }
        match (ej, em) {
            (true, true) => both += 1,
            (true, false) => only_j += 1,
            (false, true) => only_m += 1,
            _ => {}
        }
        let dm = ulp_distance(gm, t).unwrap_or(99);
        if dm >= 2 && gm < t {
            nlo += 1;
            if ej {
                hit_lo += 1;
            }
        }
        if dm >= 2 && gm > t {
            nhi += 1;
            if ej {
                hit_hi += 1;
            }
        }
    }
    println!(
        "dmid joint={nj} 0x210={nm} both={both} only_j={only_j} only_210={only_m} leftover-low {hit_lo}/{nlo} leftover-high {hit_hi}/{nhi}"
    );
    println!("pins:");
    for &z in &PINS {
        let Some(r) = rows
            .iter()
            .find(|rr| rr.direct && (rr.z - z).abs() < 1e-12)
        else {
            println!("  z={z} MISSING");
            continue;
        };
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let dj = ulp_distance(mul(w, cody(z, &c, &d)), t).unwrap_or(99);
        let dm = ulp_distance(mul(w, cody210(z)), t).unwrap_or(99);
        println!("  z={z} joint={dj} 0x210={dm}");
    }
}
