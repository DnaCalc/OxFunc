//! leftover-low last-store k of unmask 0x210 vs 114. Not an identity.
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
fn cody(y: f64, c: &[f64; 9], mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
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
fn k_of(z: f64, t: f64, ff: f64) -> Option<i32> {
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
    let mut c114 = C0;
    c114[3] = poke(C0[3], -1);
    c114[4] = poke(C0[4], 1);
    c114[5] = poke(C0[5], -1);
    c114[6] = poke(C0[6], 1);
    let mut n210 = 0usize;
    let mut n114 = 0usize;
    let mut h210 = [0usize; 9];
    let mut h114 = [0usize; 9];
    let mut shared_k = [0usize; 9];
    let mut nsh = 0usize;
    println!("leftover-low last-store k of unmask:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let g210 = mul(w, cody(r.z, &C0, MASK));
        let d210 = ulp_distance(g210, t).unwrap_or(99);
        let g114 = mul(w, cody(r.z, &c114, MASK));
        let d114 = ulp_distance(g114, t).unwrap_or(99);
        let lo210 = d210 >= 2 && g210 < t;
        let lo114 = d114 >= 2 && g114 < t;
        let ff = cody(r.z, &C0, 0);
        let k = k_of(r.z, t, ff);
        let g = mul(w, ff);
        let up = ulp_distance(g.next_up(), t).unwrap_or(99) == 0;
        if lo210 {
            n210 += 1;
            if let Some(kk) = k {
                let ak = kk.unsigned_abs() as usize;
                if ak < 9 {
                    h210[ak] += 1;
                }
            }
            if d114 == 1 {
                println!(
                    "  0x210-LOW to 114 1-ULP z={:.16} unmask_k={k:?} last-mul next_up={up}",
                    r.z
                );
            }
        }
        if lo114 {
            n114 += 1;
            if let Some(kk) = k {
                let ak = kk.unsigned_abs() as usize;
                if ak < 9 {
                    h114[ak] += 1;
                }
            }
            if d210 == 1 {
                println!(
                    "  114-LOW from 0x210 1-ULP z={:.16} unmask_k={k:?} last-mul next_up={up}",
                    r.z
                );
            }
        }
        if lo210 && lo114 {
            nsh += 1;
            if let Some(kk) = k {
                let ak = kk.unsigned_abs() as usize;
                if ak < 9 {
                    shared_k[ak] += 1;
                }
            }
        }
    }
    print!("0x210 leftover-low n={n210} unmask |k|");
    for (i, c) in h210.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    print!("\n114 leftover-low n={n114} unmask |k|");
    for (i, c) in h114.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    print!("\nshared leftover-low n={nsh} unmask |k|");
    for (i, c) in shared_k.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();
}
