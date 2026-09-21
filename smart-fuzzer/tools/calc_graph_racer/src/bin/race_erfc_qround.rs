//! Signed-ULP miss law and last-mul rounding of Q=w*F. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const CW_RD: u16 = 0x173F;
const CW_RU: u16 = 0x1B3F;
const CW_RZ: u16 = 0x1F3F;
const ULP_CAP: u64 = 1 << 20;
const C: [f64; 9] = [
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
const D: [f64; 8] = [
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
fn cody_cd(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(&ext_add(&xnum, &ef(C[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW);
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn signed_ulp(got: f64, or: f64) -> Option<i64> {
    let d = ulp_distance(got, or)? as i64;
    if d == 0 {
        Some(0)
    } else if got > or {
        Some(d)
    } else {
        Some(-d)
    }
}
fn band(z: f64) -> usize {
    if z < 1.0 {
        0
    } else if z < 2.0 {
        1
    } else if z < 4.0 {
        2
    } else if z < 8.0 {
        3
    } else {
        4
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mid: Vec<_> = rows.iter().filter(|r| r.z >= 0.5 && r.z < 4.0).cloned().collect();

    println!("## signed ULP histogram Cody 0x74 Q=w*F64 mid (merged / direct)");
    let mut hist = [0usize; 15]; // idx 7 = 0, 8 = +1, 6 = -1
    let mut hist_d = [0usize; 15];
    let mut band_ex = [0usize; 5];
    let mut band_n = [0usize; 5];
    let mut band_ex_d = [0usize; 5];
    let mut band_n_d = [0usize; 5];
    for r in &mid {
        let ff = cody_cd(r.z, 0x74);
        let qg = f::w_rn53(r.z) * ff;
        let qo = f64::from_bits(r.qbits);
        let Some(s) = signed_ulp(qg, qo) else {
            continue;
        };
        let b = band(r.z);
        band_n[b] += 1;
        if r.direct {
            band_n_d[b] += 1;
        }
        if s == 0 {
            band_ex[b] += 1;
            if r.direct {
                band_ex_d[b] += 1;
            }
        }
        let idx = (s.clamp(-7, 7) + 7) as usize;
        hist[idx] += 1;
        if r.direct {
            hist_d[idx] += 1;
        }
    }
    print!("ulp ");
    for k in -7i32..=7 {
        print!("{k:+3} ");
    }
    println!();
    print!("n   ");
    for i in 0..15 {
        print!("{:3} ", hist[i]);
    }
    println!();
    print!("dir ");
    for i in 0..15 {
        print!("{:3} ", hist_d[i]);
    }
    println!();
    for (i, name) in ["[0.5,1)", "[1,2)", "[2,4)", "[4,8)", "[8,inf)"].iter().enumerate() {
        if band_n[i] == 0 {
            continue;
        }
        println!(
            "band {name} exact {}/{}  direct {}/{}",
            band_ex[i], band_n[i], band_ex_d[i], band_n_d[i]
        );
    }

    println!("\n## last-mul rounding of Q=w⊗F80 (F=Cody 0x74 stored) mid");
    for (name, cw) in [("RN", CW), ("RD", CW_RD), ("RU", CW_RU), ("RZ", CW_RZ)] {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut dx = 0usize;
        let mut dn = 0usize;
        let mut max = 0u64;
        let mut hist = [0usize; 15];
        for r in &mid {
            let ff = cody_cd(r.z, 0x74);
            let w = f::w_rn53(r.z);
            let qg = ext_to_f64(&ext_mul(&ef(w), &ef(ff), cw), cw);
            let qo = f64::from_bits(r.qbits);
            let Some(s) = signed_ulp(qg, qo) else {
                continue;
            };
            n += 1;
            if r.direct {
                dn += 1;
            }
            if s == 0 {
                ex += 1;
                if r.direct {
                    dx += 1;
                }
            } else {
                max = max.max(s.unsigned_abs());
            }
            hist[(s.clamp(-7, 7) + 7) as usize] += 1;
        }
        print!("{name} exact {ex}/{n} d={dx}/{dn} max={max} hist ");
        for i in 0..15 {
            print!("{} ", hist[i]);
        }
        println!();
    }

    println!("\n## f64 mul vs x87-RN last mul vs nextafter w*F (mask0 and 0x74)");
    for (lab, mask) in [("mask0", 0u32), ("0x74", 0x74)] {
        for (name, adj) in [
            ("w*F", 0i32),
            ("next_up", 1),
            ("next_down", -1),
        ] {
            let mut ex = 0usize;
            let mut n = 0usize;
            let mut dx = 0usize;
            for r in &mid {
                let ff = cody_cd(r.z, mask);
                let mut qg = f::w_rn53(r.z) * ff;
                if adj > 0 {
                    qg = qg.next_up();
                } else if adj < 0 {
                    qg = qg.next_down();
                }
                let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
                if d > ULP_CAP {
                    continue;
                }
                n += 1;
                if d == 0 {
                    ex += 1;
                    if r.direct {
                        dx += 1;
                    }
                }
            }
            println!("{lab:6} {name:10} {ex}/{n} dmid={dx}");
        }
    }
}
