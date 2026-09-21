//! 0x210 + C[4]-1 D[5]-1 leftover DIRECT [0.5,4). Not an identity.
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
fn hit(z: f64, t: f64, c: &[f64; 9], d: &[f64; 8]) -> bool {
    ulp_distance(mul(f::w_rn53(z), cody(z, c, d)), t).unwrap_or(99) == 0
}
fn signed_k(z: f64, t: f64, c: &[f64; 9], d: &[f64; 8]) -> Option<i32> {
    let ff = cody(z, c, d);
    for ak in 0i32..=8 {
        for &s in &[-1i32, 1] {
            let k = if ak == 0 { 0 } else { s * ak };
            if ak == 0 && s < 0 {
                continue;
            }
            if ulp_distance(mul(f::w_rn53(z), poke(ff, k)), t).unwrap_or(99) == 0 {
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
    let dirs: Vec<&f::QRow> = rows
        .iter()
        .filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0)
        .collect();
    let n = dirs.len();
    let mut cj = C0;
    let mut dj = D0;
    cj[4] = poke(C0[4], -1);
    dj[5] = poke(D0[5], -1);
    let mut both = 0usize;
    let mut only_j = 0usize;
    let mut only_m = 0usize;
    let mut nj = 0usize;
    let mut nlo = 0usize;
    let mut hit_lo = 0usize;
    let mut nhi = 0usize;
    let mut hit_hi = 0usize;
    let mut slo = 0usize;
    let mut slo_c = 0usize;
    let mut shi = 0usize;
    let mut shi_c = 0usize;
    let mut n1 = 0usize;
    let mut u1 = 0usize;
    println!("DIRECT [0.5,4) n={n} C[4]-1 D[5]-1 vs 0x210 leftover:");
    for r in &dirs {
        let t = f64::from_bits(r.qbits);
        let ej = hit(r.z, t, &cj, &dj);
        let em = hit(r.z, t, &C0, &D0);
        if ej {
            nj += 1;
        }
        match (ej, em) {
            (true, true) => both += 1,
            (true, false) => only_j += 1,
            (false, true) => only_m += 1,
            _ => {}
        }
        let gm = mul(f::w_rn53(r.z), cody(r.z, &C0, &D0));
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
        let g = mul(f::w_rn53(r.z), cody(r.z, &cj, &dj));
        let d = ulp_distance(g, t).unwrap_or(99);
        let k = signed_k(r.z, t, &cj, &dj);
        if d == 1 {
            n1 += 1;
            if k.is_some() {
                u1 += 1;
            }
        } else if d >= 2 && g < t {
            slo += 1;
            if matches!(k, Some(kk) if kk > 0) {
                slo_c += 1;
            }
        } else if d >= 2 && g > t {
            shi += 1;
            if matches!(k, Some(kk) if kk < 0) {
                shi_c += 1;
            }
        }
    }
    println!(
        "joint={nj} both={both} only_j={only_j} only_210={only_m} leftover-low {hit_lo}/{nlo} leftover-high {hit_hi}/{nhi}"
    );
    println!("self leftover-low {slo_c}/{slo} leftover-high {shi_c}/{shi} 1-ULP {u1}/{n1}");
}
