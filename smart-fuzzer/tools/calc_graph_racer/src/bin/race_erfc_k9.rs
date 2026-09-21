//! z where 0x210 fused and unmask is last-store |k|=1. Not an identity.
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
fn cody_mask(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        let bn = 2 + 2 * i;
        let bd = 3 + 2 * i;
        if mask & (1u32 << bn) != 0 {
            xnum = ef(ext_to_f64(&xnum, CW));
        }
        if mask & (1u32 << bd) != 0 {
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
fn signed_k(z: f64, t: f64, ff: f64) -> Option<i32> {
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
fn kind(z: f64) -> &'static str {
    if (z * 64.0).fract().abs() < 1e-12 {
        "dyad64"
    } else if (z * 96.0 - (z * 96.0).round()).abs() < 1e-10 {
        "96"
    } else {
        "other"
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    println!("0x210 k=0 vs unmask k:");
    let mut n_conv = 0usize;
    let mut n_lose = 0usize;
    let mut n_gain_other = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let ku = signed_k(r.z, t, cody_mask(r.z, 0));
        let km = signed_k(r.z, t, cody_mask(r.z, 0x210));
        match (ku, km) {
            (Some(u), Some(0)) if u != 0 => {
                n_conv += 1;
                println!(
                    "  CONV z={:.16} kind={} unmask_k={u}",
                    r.z,
                    kind(r.z)
                );
            }
            (Some(0), Some(m)) if m != 0 => {
                n_lose += 1;
                println!(
                    "  LOSE z={:.16} kind={} 210_k={m}",
                    r.z,
                    kind(r.z)
                );
            }
            (u, Some(0)) if u != Some(0) => {
                n_gain_other += 1;
                println!(
                    "  GAIN z={:.16} kind={} unmask_k={:?}",
                    r.z,
                    kind(r.z),
                    u
                );
            }
            _ => {}
        }
    }
    println!("CONV unmask|k|→210fused={n_conv} LOSE unmaskfused→210|k|={n_lose} GAIN other={n_gain_other}");
}
