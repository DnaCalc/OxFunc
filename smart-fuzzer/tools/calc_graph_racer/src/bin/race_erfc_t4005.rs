//! 0x4005 vs 0x5005 leftover overlap DIRECT z>=4. Not an identity.
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
const SIX: [f64; 6] = [
    4.0520833333333330,
    4.0625000000000000,
    5.2395833333333330,
    5.2708333333333330,
    5.3333333333333330,
    6.0000000000000000,
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let tail: Vec<&f::QRow> = rows.iter().filter(|r| r.direct && r.z >= 4.0).collect();
    let n = tail.len();
    let masks = [
        ("unmask", 0u32),
        ("bit6", 1u32 << 6),
        ("0x41", 0x41u32),
        ("0x5005", 0x5005u32),
        ("0x4005", 0x4005u32),
    ];
    println!("DIRECT z>=4 n={n} 0x4005 leftover overlap:");
    for &(name, m) in &masks {
        let mut both = 0usize;
        let mut only_m = 0usize;
        let mut only_5 = 0usize;
        let mut nf = 0usize;
        let mut n5 = 0usize;
        let mut nlo = 0usize;
        let mut hit_lo = 0usize;
        let mut nhi = 0usize;
        let mut hit_hi = 0usize;
        for r in &tail {
            let t = f64::from_bits(r.qbits);
            let em = hit(r.z, t, m);
            let e5 = hit(r.z, t, 0x5005);
            if em {
                nf += 1;
            }
            if e5 {
                n5 += 1;
            }
            match (em, e5) {
                (true, true) => both += 1,
                (true, false) => only_m += 1,
                (false, true) => only_5 += 1,
                _ => {}
            }
            let g5 = mul(f::w_rn53(r.z), cephes_mask(r.z, 0x5005));
            let d = ulp_distance(g5, t).unwrap_or(99);
            if d >= 2 && g5 < t {
                nlo += 1;
                if em {
                    hit_lo += 1;
                }
            }
            if d >= 2 && g5 > t {
                nhi += 1;
                if em {
                    hit_hi += 1;
                }
            }
        }
        println!(
            "  vs5005 {name} fused={nf} 5005={n5} both={both} only_m={only_m} only_5005={only_5} leftover-low {hit_lo}/{nlo} leftover-high {hit_hi}/{nhi}"
        );
    }

    println!("0x4005 leftover of itself:");
    let mut nlo4 = 0usize;
    let mut nhi4 = 0usize;
    let mut n1 = 0usize;
    let mut loz: Vec<f64> = Vec::new();
    for r in &tail {
        let t = f64::from_bits(r.qbits);
        let g = mul(f::w_rn53(r.z), cephes_mask(r.z, 0x4005));
        let d = ulp_distance(g, t).unwrap_or(99);
        if d == 1 {
            n1 += 1;
        }
        if d >= 2 && g < t {
            nlo4 += 1;
            loz.push(r.z);
        }
        if d >= 2 && g > t {
            nhi4 += 1;
        }
    }
    println!("  leftover-low={nlo4} leftover-high={nhi4} 1-ULP={n1}");
    print!("  leftover-low z:");
    for z in &loz {
        print!(" {z:.16}");
    }
    println!();

    println!("0x4005 leftover-low of 0x5005 hits:");
    let mut nlo5 = 0usize;
    for r in &tail {
        let t = f64::from_bits(r.qbits);
        let g5 = mul(f::w_rn53(r.z), cephes_mask(r.z, 0x5005));
        let d = ulp_distance(g5, t).unwrap_or(99);
        if d >= 2 && g5 < t {
            nlo5 += 1;
            println!(
                "  z={:.16} d={d} unmask={} bit6={} 0x41={} 0x5005={} 0x4005={}",
                r.z,
                hit(r.z, t, 0),
                hit(r.z, t, 1 << 6),
                hit(r.z, t, 0x41),
                hit(r.z, t, 0x5005),
                hit(r.z, t, 0x4005)
            );
        }
    }
    println!("  leftover-low n={nlo5}");

    println!("6 leftover-low of 0x5005 [4,8):");
    for &lz in &SIX {
        let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
            println!("  z={lz:.16} MISSING");
            continue;
        };
        let t = f64::from_bits(r.qbits);
        println!(
            "  z={lz:.16} unmask={} bit6={} 0x41={} 0x5005={} 0x4005={}",
            hit(lz, t, 0),
            hit(lz, t, 1 << 6),
            hit(lz, t, 0x41),
            hit(lz, t, 0x5005),
            hit(lz, t, 0x4005)
        );
    }

    println!("0x4005 vs unmask CONV/LOSE:");
    let mut n_conv = 0usize;
    let mut n_lose = 0usize;
    let mut hu = [0usize; 9];
    let mut h4 = [0usize; 9];
    let mut h5 = [0usize; 9];
    for r in &tail {
        let t = f64::from_bits(r.qbits);
        let ku = signed_k(r.z, t, 0);
        let k4 = signed_k(r.z, t, 0x4005);
        let k5 = signed_k(r.z, t, 0x5005);
        if let Some(u) = ku {
            if (u.unsigned_abs() as usize) < 9 {
                hu[u.unsigned_abs() as usize] += 1;
            }
        }
        if let Some(v) = k4 {
            if (v.unsigned_abs() as usize) < 9 {
                h4[v.unsigned_abs() as usize] += 1;
            }
        }
        if let Some(v) = k5 {
            if (v.unsigned_abs() as usize) < 9 {
                h5[v.unsigned_abs() as usize] += 1;
            }
        }
        match (ku, k4) {
            (Some(u), Some(0)) if u != 0 => {
                n_conv += 1;
                println!("  CONV z={:.16} unmask_k={u}", r.z);
            }
            (Some(0), Some(m)) if m != 0 => {
                n_lose += 1;
                println!("  LOSE z={:.16} 4005_k={m}", r.z);
            }
            _ => {}
        }
    }
    println!(
        "CONV={n_conv} LOSE={n_lose} net={:+}",
        n_conv as i32 - n_lose as i32
    );
    print!("unmask |k|");
    for (i, c) in hu.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    print!("  0x4005 |k|");
    for (i, c) in h4.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    print!("  0x5005 |k|");
    for (i, c) in h5.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();

    println!("0x4005 vs 0x5005 fused CONV/LOSE:");
    let mut c45 = 0usize;
    let mut l45 = 0usize;
    for r in &tail {
        let t = f64::from_bits(r.qbits);
        let e4 = hit(r.z, t, 0x4005);
        let e5 = hit(r.z, t, 0x5005);
        if e4 && !e5 {
            c45 += 1;
            let k5 = signed_k(r.z, t, 0x5005);
            println!("  4005-only z={:.16} 5005_k={k5:?}", r.z);
        }
        if e5 && !e4 {
            l45 += 1;
            let k4 = signed_k(r.z, t, 0x4005);
            println!("  5005-only z={:.16} 4005_k={k4:?}", r.z);
        }
    }
    println!(
        "4005-only={c45} 5005-only={l45} net={:+}",
        c45 as i32 - l45 as i32
    );
}
