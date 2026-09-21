//! 0x210 leftover ± vs unmask Cody F last-store. Not an identity.
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
fn hit(z: f64, t: f64, c: &[f64; 9], d: &[f64; 8]) -> bool {
    ulp_distance(mul(f::w_rn53(z), cody(z, c, d, MASK)), t).unwrap_or(99) == 0
}
fn signed_k(z: f64, t: f64, c: &[f64; 9], d: &[f64; 8]) -> Option<i32> {
    let ff = cody(z, c, d, 0);
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
fn apply(idx: usize, k: i32, c: &mut [f64; 9], d: &mut [f64; 8]) {
    if idx < 9 {
        c[idx] = poke(C0[idx], k);
    } else {
        d[idx - 9] = poke(D0[idx - 9], k);
    }
}
fn name(idx: usize) -> String {
    if idx < 9 {
        format!("C[{idx}]")
    } else {
        format!("D[{}]", idx - 9)
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let dirs: Vec<&f::QRow> = rows
        .iter()
        .filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0)
        .collect();
    let mut nlo = 0usize;
    let mut hit = [0usize; 9];
    let mut cov = 0usize;
    println!("0x210 leftover-low vs unmask Cody F last-store DIRECT [0.5,4):");
    for r in &dirs {
        let t = f64::from_bits(r.qbits);
        let g = mul(f::w_rn53(r.z), cody(r.z, &C0, &D0, MASK));
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 2 && g < t) {
            continue;
        }
        nlo += 1;
        let k = signed_k(r.z, t, &C0, &D0);
        match k {
            Some(kk) => {
                let ak = kk.unsigned_abs() as usize;
                if ak < 9 {
                    hit[ak] += 1;
                }
                cov += 1;
                println!("  z={:.16} d={d} unmask_k={kk}", r.z);
            }
            None => println!("  z={:.16} d={d} unmask MISS", r.z),
        }
    }
    print!("leftover-low unmask F± {cov}/{nlo} |k|");
    for (i, c) in hit.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();
    let mut nhi = 0usize;
    let mut hith = [0usize; 9];
    let mut covh = 0usize;
    for r in &dirs {
        let t = f64::from_bits(r.qbits);
        let g = mul(f::w_rn53(r.z), cody(r.z, &C0, &D0, MASK));
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 2 && g > t) {
            continue;
        }
        nhi += 1;
        match signed_k(r.z, t, &C0, &D0) {
            Some(kk) => {
                let ak = kk.unsigned_abs() as usize;
                if ak < 9 {
                    hith[ak] += 1;
                }
                covh += 1;
            }
            None => println!("  leftover-high MISS z={:.16} d={d}", r.z),
        }
    }
    print!("leftover-high unmask F± {covh}/{nhi} |k|");
    for (i, c) in hith.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();
}
