//! 0x4e05 / 0x24a5 vs 0x4605 leftover DIRECT z>=4. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const CEPHES_P: [f64; 9] = [
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
const CEPHES_Q: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
const CEPHES_R: [f64; 6] = [
    5.64189583547755073984e-1,
    1.27536670759978104416e0,
    5.01905042251180477414e0,
    6.16021097993053585195e0,
    7.40974269950448939160e0,
    2.97886665372100240670e0,
];
const CEPHES_S: [f64; 6] = [
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
fn cephes_mask(x: f64, mask: u32) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl(xe, &CEPHES_P, mask, 0);
        let (q, b) = p1evl(xe, &CEPHES_Q, mask, b);
        (p, q, b)
    } else {
        let (r, b) = polevl(xe, &CEPHES_R, mask, 0);
        let (s, b) = p1evl(xe, &CEPHES_S, mask, b);
        (r, s, b)
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
fn hit(z: f64, t: f64, mask: u32) -> bool {
    ulp_distance(mul(f::w_rn53(z), cephes_mask(z, mask)), t).unwrap_or(99) == 0
}
fn signed_k(z: f64, t: f64, mask: u32) -> Option<i32> {
    let ff = cephes_mask(z, mask);
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

fn leftover_self(tail: &[&f::QRow], mask: u32, name: &str) {
    let mut nlo = 0usize;
    let mut nhi = 0usize;
    let mut n1 = 0usize;
    let mut lo_cov = 0usize;
    let mut hi_cov = 0usize;
    let mut u1_cov = 0usize;
    let mut fused = 0usize;
    for r in tail {
        let t = f64::from_bits(r.qbits);
        let ff = cephes_mask(r.z, mask);
        let g = mul(f::w_rn53(r.z), ff);
        let d = ulp_distance(g, t).unwrap_or(99);
        let k = signed_k(r.z, t, mask);
        if d == 0 {
            fused += 1;
        } else if d == 1 {
            n1 += 1;
            if k.is_some() {
                u1_cov += 1;
            }
        } else if g < t {
            nlo += 1;
            if matches!(k, Some(kk) if kk > 0) {
                lo_cov += 1;
            } else {
                println!("  {name} leftover-low MISS z={:.16} k={k:?} d={d}", r.z);
            }
        } else {
            nhi += 1;
            if matches!(k, Some(kk) if kk < 0) {
                hi_cov += 1;
            } else {
                println!("  {name} leftover-high MISS z={:.16} k={k:?} d={d}", r.z);
            }
        }
    }
    println!(
        "  {name} fused={fused} leftover-low {lo_cov}/{nlo} leftover-high {hi_cov}/{nhi} 1-ULP {u1_cov}/{n1}"
    );
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let tail: Vec<&f::QRow> = rows.iter().filter(|r| r.direct && r.z >= 4.0).collect();
    let n = tail.len();
    println!("DIRECT z>=4 n={n} 60 vs 0x4605 leftover:");
    for &(name, m) in &[
        ("unmask", 0u32),
        ("0x4605", 0x4605u32),
        ("0x4e05", 0x4e05u32),
        ("0x24a5", 0x24a5u32),
        ("0x4005", 0x4005u32),
    ] {
        let mut both = 0usize;
        let mut only_m = 0usize;
        let mut only_b = 0usize;
        let mut nf = 0usize;
        let mut nlo = 0usize;
        let mut hit_lo = 0usize;
        let mut nhi = 0usize;
        let mut hit_hi = 0usize;
        for r in &tail {
            let t = f64::from_bits(r.qbits);
            let em = hit(r.z, t, m);
            let eb = hit(r.z, t, 0x4605);
            if em {
                nf += 1;
            }
            match (em, eb) {
                (true, true) => both += 1,
                (true, false) => only_m += 1,
                (false, true) => only_b += 1,
                _ => {}
            }
            let gb = mul(f::w_rn53(r.z), cephes_mask(r.z, 0x4605));
            let d = ulp_distance(gb, t).unwrap_or(99);
            if d >= 2 && gb < t {
                nlo += 1;
                if em {
                    hit_lo += 1;
                }
            }
            if d >= 2 && gb > t {
                nhi += 1;
                if em {
                    hit_hi += 1;
                }
            }
        }
        println!(
            "  vs4605 {name} fused={nf} both={both} only_m={only_m} only_4605={only_b} leftover-low {hit_lo}/{nlo} leftover-high {hit_hi}/{nhi}"
        );
    }
    println!("0x4605 leftover-low hits:");
    for r in &tail {
        let t = f64::from_bits(r.qbits);
        let gb = mul(f::w_rn53(r.z), cephes_mask(r.z, 0x4605));
        let d = ulp_distance(gb, t).unwrap_or(99);
        if d >= 2 && gb < t {
            println!(
                "  z={:.16} d={d} 0x4605={} 0x4e05={} 0x24a5={}",
                r.z,
                hit(r.z, t, 0x4605),
                hit(r.z, t, 0x4e05),
                hit(r.z, t, 0x24a5)
            );
        }
    }
    println!("0x4e05 vs 0x24a5:");
    let mut both = 0usize;
    let mut only_a = 0usize;
    let mut only_b = 0usize;
    for r in &tail {
        let t = f64::from_bits(r.qbits);
        let ea = hit(r.z, t, 0x4e05);
        let eb = hit(r.z, t, 0x24a5);
        match (ea, eb) {
            (true, true) => both += 1,
            (true, false) => {
                only_a += 1;
                println!(
                    "  0x4e05-only z={:.16} 24a5_k={:?}",
                    r.z,
                    signed_k(r.z, t, 0x24a5)
                );
            }
            (false, true) => {
                only_b += 1;
                println!(
                    "  0x24a5-only z={:.16} 4e05_k={:?}",
                    r.z,
                    signed_k(r.z, t, 0x4e05)
                );
            }
            _ => {}
        }
    }
    println!("both={both} only_4e05={only_a} only_24a5={only_b}");
    println!("CONV/LOSE vs unmask:");
    for &(name, m) in &[("0x4e05", 0x4e05u32), ("0x24a5", 0x24a5u32)] {
        let mut n_conv = 0usize;
        let mut n_lose = 0usize;
        let mut hm = [0usize; 9];
        for r in &tail {
            let t = f64::from_bits(r.qbits);
            let ku = signed_k(r.z, t, 0);
            let km = signed_k(r.z, t, m);
            if let Some(v) = km {
                if (v.unsigned_abs() as usize) < 9 {
                    hm[v.unsigned_abs() as usize] += 1;
                }
            }
            match (ku, km) {
                (Some(u), Some(0)) if u != 0 => n_conv += 1,
                (Some(0), Some(k)) if k != 0 => n_lose += 1,
                _ => {}
            }
        }
        print!(
            "  {name} CONV={n_conv} LOSE={n_lose} net={:+} |k|",
            n_conv as i32 - n_lose as i32
        );
        for (i, c) in hm.iter().enumerate() {
            if *c > 0 {
                print!(" {i}:{c}");
            }
        }
        println!();
    }
    println!("leftover last-store of itself:");
    leftover_self(&tail, 0x4e05, "0x4e05");
    leftover_self(&tail, 0x24a5, "0x24a5");
}
