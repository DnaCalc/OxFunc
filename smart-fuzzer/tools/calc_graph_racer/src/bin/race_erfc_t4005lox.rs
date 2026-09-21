//! Exclusive leftover-low z of 0x4005 vs 0x4e05 and 1-ULP |k|. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const P0: [f64; 9] = [
    2.46196981473530512524e-10,
    5.64189564831068821977e-1,
    7.46321056442269912687e0,
    4.86371970985681366614e1,
    1.96520832956077098242e2,
    5.26445194995477358631e2,
    9.34528527171957607540e2,
    1.02755188689515710272e3,
    5.57535335369399327526e2,
];
const Q0: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
const R0: [f64; 6] = [
    5.64189583547755073984e-1,
    1.27536670759978104416e0,
    5.01905042251180477414e0,
    6.16021097993053585195e0,
    7.40974269950448939160e0,
    2.97886665372100240670e0,
];
const S0: [f64; 6] = [
    2.26052863220117276590e0,
    9.39603524938001434673e0,
    1.20489539808096656605e1,
    1.70814450747565897222e1,
    9.60896809063285878198e0,
    3.36907645100081516050e0,
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
fn polevl(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ef(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn p1evl(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ext_add(&x, &ef(coef[0]), CW);
    ans = maybe(ans, mask, bit);
    bit += 1;
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn cephes(x: f64, p: &[f64; 9], q: &[f64; 8], mask: u32) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (pp, b) = polevl(xe, p, mask, 0);
        let (qq, b) = p1evl(xe, q, mask, b);
        (pp, qq, b)
    } else {
        let (rr, b) = polevl(xe, &R0, mask, 0);
        let (ss, b) = p1evl(xe, &S0, mask, b);
        (rr, ss, b)
    };
    let mut v = ext_div(&num, &den, CW);
    v = maybe(v, mask, bit0);
    ext_to_f64(&v, CW)
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
fn bucket(g: f64, t: f64) -> &'static str {
    let d = ulp_distance(g, t).unwrap_or(99);
    if d == 0 {
        "fused"
    } else if d == 1 {
        "1-ULP"
    } else if g < t {
        "leftover-low"
    } else {
        "leftover-high"
    }
}
fn grid(z: f64) -> &'static str {
    if z.fract() == 0.0 {
        "int"
    } else if (z * 2.0).fract() == 0.0 {
        "dyad2"
    } else if (z * 16.0).fract() == 0.0 {
        "dyad16"
    } else if (z * 48.0 - (z * 48.0).round()).abs() < 1e-12 {
        "48"
    } else if (z * 96.0 - (z * 96.0).round()).abs() < 1e-12 {
        "96"
    } else {
        "other"
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut h5 = [0usize; 9];
    let mut h66 = [0usize; 9];
    let mut n5 = 0usize;
    let mut n66 = 0usize;
    let mut nsh = 0usize;
    println!("exclusive leftover-low 0x4005 vs 0x4e05:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 4.0) {
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let g5 = mul(w, cephes(r.z, &P0, &Q0, 0x4005));
        let d5 = ulp_distance(g5, t).unwrap_or(99);
        let g66 = mul(w, cephes(r.z, &P0, &Q0, 0x4e05));
        let d66 = ulp_distance(g66, t).unwrap_or(99);
        let lo5 = d5 >= 2 && g5 < t;
        let lo66 = d66 >= 2 && g66 < t;
        let ff = cephes(r.z, &P0, &Q0, 0);
        let k = k_of(r.z, t, ff);
        let g = mul(w, ff);
        let up = ulp_distance(g.next_up(), t).unwrap_or(99) == 0;
        if lo5 {
            n5 += 1;
            if let Some(kk) = k {
                let ak = kk.unsigned_abs() as usize;
                if ak < 9 {
                    h5[ak] += 1;
                }
            }
        }
        if lo66 {
            n66 += 1;
            if let Some(kk) = k {
                let ak = kk.unsigned_abs() as usize;
                if ak < 9 {
                    h66[ak] += 1;
                }
            }
        }
        if lo5 && lo66 {
            nsh += 1;
        } else if lo5 {
            println!(
                "  only4005 z={:.16} grid={} k={k:?} last-mul_up={up} 4e05={}",
                r.z,
                grid(r.z),
                bucket(g66, t)
            );
        } else if lo66 {
            println!(
                "  only4e05 z={:.16} grid={} k={k:?} last-mul_up={up} 4005={}",
                r.z,
                grid(r.z),
                bucket(g5, t)
            );
        }
    }
    println!("leftover-low n4005={n5} n4e05={n66} shared={nsh}");
    let mut u5 = [0usize; 9];
    let mut u66 = [0usize; 9];
    let mut nu5 = 0usize;
    let mut nu66 = 0usize;
    let mut nush = 0usize;
    let mut nconv = 0usize;
    println!("1-ULP |k| 0x4005 vs 0x4e05:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 4.0) {
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let d5 = ulp_distance(mul(w, cephes(r.z, &P0, &Q0, 0x4005)), t).unwrap_or(99);
        let d66 = ulp_distance(mul(w, cephes(r.z, &P0, &Q0, 0x4e05)), t).unwrap_or(99);
        let k = k_of(r.z, t, cephes(r.z, &P0, &Q0, 0));
        if d5 == 1 {
            nu5 += 1;
            if let Some(kk) = k {
                let ak = kk.unsigned_abs() as usize;
                if ak < 9 {
                    u5[ak] += 1;
                }
            }
            if d66 == 0 {
                nconv += 1;
                let g = mul(w, cephes(r.z, &P0, &Q0, 0));
                println!(
                    "  CONV z={:.16} k={k:?} last-mul_up={}",
                    r.z,
                    ulp_distance(g.next_up(), t).unwrap_or(99) == 0
                );
            }
        }
        if d66 == 1 {
            nu66 += 1;
            if let Some(kk) = k {
                let ak = kk.unsigned_abs() as usize;
                if ak < 9 {
                    u66[ak] += 1;
                }
            }
        }
        if d5 == 1 && d66 == 1 {
            nush += 1;
        }
    }
    print!("0x4005 1-ULP n={nu5} unmask |k|");
    for (i, c) in u5.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    print!("\n0x4e05 1-ULP n={nu66} unmask |k|");
    for (i, c) in u66.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!("\nshared 1-ULP={nush} CONV={nconv}");
}
