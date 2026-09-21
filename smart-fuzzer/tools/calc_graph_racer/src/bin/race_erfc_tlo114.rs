//! Tail leftover-low of 0x5005 as 66 and 0x4005 as 0x4e05. Not an identity.
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let tail: Vec<&f::QRow> = rows.iter().filter(|r| r.direct && r.z >= 4.0).collect();
    let mut p66 = P0;
    let mut q66 = Q0;
    p66[4] = poke(P0[4], -1);
    q66[0] = poke(Q0[0], 1);
    q66[1] = poke(Q0[1], -1);
    q66[3] = poke(Q0[3], 1);
    println!("DIRECT z>=4 n={} 0x5005 leftover-low as 66:", tail.len());
    let mut nlo = 0usize;
    let mut b66 = [0usize; 4];
    let idx = |s: &str| match s {
        "fused" => 0,
        "1-ULP" => 1,
        "leftover-low" => 2,
        _ => 3,
    };
    let names = ["fused", "1-ULP", "leftover-low", "leftover-high"];
    for r in &tail {
        let t = f64::from_bits(r.qbits);
        let g5 = mul(f::w_rn53(r.z), cephes(r.z, &P0, &Q0, 0x5005));
        let d5 = ulp_distance(g5, t).unwrap_or(99);
        if !(d5 >= 2 && g5 < t) {
            continue;
        }
        nlo += 1;
        let g = mul(f::w_rn53(r.z), cephes(r.z, &p66, &q66, 0x24a5));
        let b = bucket(g, t);
        b66[idx(b)] += 1;
        println!("  z={:.16} d5005={d5} 66={b}", r.z);
    }
    print!("0x5005 leftover-low n={nlo} as 66");
    for (i, c) in b66.iter().enumerate() {
        if *c > 0 {
            print!(" {}:{}", names[i], c);
        }
    }
    println!();
    println!("0x4005 leftover-low as 0x4e05:");
    let mut n4 = 0usize;
    let mut b4 = [0usize; 4];
    for r in &tail {
        let t = f64::from_bits(r.qbits);
        let g4 = mul(f::w_rn53(r.z), cephes(r.z, &P0, &Q0, 0x4005));
        let d4 = ulp_distance(g4, t).unwrap_or(99);
        if !(d4 >= 2 && g4 < t) {
            continue;
        }
        n4 += 1;
        let g = mul(f::w_rn53(r.z), cephes(r.z, &P0, &Q0, 0x4e05));
        let b = bucket(g, t);
        b4[idx(b)] += 1;
        println!("  z={:.16} d4005={d4} 0x4e05={b}", r.z);
    }
    print!("0x4005 leftover-low n={n4} as 0x4e05");
    for (i, c) in b4.iter().enumerate() {
        if *c > 0 {
            print!(" {}:{}", names[i], c);
        }
    }
    println!();
    println!("0x4e05 leftover-low as 0x4005:");
    let mut n4e = 0usize;
    let mut b4e = [0usize; 4];
    for r in &tail {
        let t = f64::from_bits(r.qbits);
        let g = mul(f::w_rn53(r.z), cephes(r.z, &P0, &Q0, 0x4e05));
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 2 && g < t) {
            continue;
        }
        n4e += 1;
        let g4 = mul(f::w_rn53(r.z), cephes(r.z, &P0, &Q0, 0x4005));
        let b = bucket(g4, t);
        b4e[idx(b)] += 1;
        println!("  z={:.16} d4e05={d} 0x4005={b}", r.z);
    }
    print!("0x4e05 leftover-low n={n4e} as 0x4005");
    for (i, c) in b4e.iter().enumerate() {
        if *c > 0 {
            print!(" {}:{}", names[i], c);
        }
    }
    println!();
}
