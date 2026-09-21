//! Tail leftover-high grid 0x5005 vs 66 and 0x4005 vs 0x4e05. Not an identity.
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
    let mut p66 = P0;
    let mut q66 = Q0;
    p66[4] = poke(P0[4], -1);
    q66[0] = poke(Q0[0], 1);
    q66[1] = poke(Q0[1], -1);
    q66[3] = poke(Q0[3], 1);
    let mut g5: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    let mut g66: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    let mut g4: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    let mut g4e: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 4.0) {
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let g = |mask: u32, p: &[f64; 9], q: &[f64; 8]| mul(w, cephes(r.z, p, q, mask));
        let d5 = ulp_distance(g(0x5005, &P0, &Q0), t).unwrap_or(99);
        let gg5 = g(0x5005, &P0, &Q0);
        if d5 >= 2 && gg5 > t {
            *g5.entry(grid(r.z)).or_insert(0) += 1;
        }
        let gg66 = g(0x24a5, &p66, &q66);
        let d66 = ulp_distance(gg66, t).unwrap_or(99);
        if d66 >= 2 && gg66 > t {
            *g66.entry(grid(r.z)).or_insert(0) += 1;
        }
        let gg4 = g(0x4005, &P0, &Q0);
        let d4 = ulp_distance(gg4, t).unwrap_or(99);
        if d4 >= 2 && gg4 > t {
            *g4.entry(grid(r.z)).or_insert(0) += 1;
        }
        let gg4e = g(0x4e05, &P0, &Q0);
        let d4e = ulp_distance(gg4e, t).unwrap_or(99);
        if d4e >= 2 && gg4e > t {
            *g4e.entry(grid(r.z)).or_insert(0) += 1;
        }
    }
    print!("0x5005 leftover-high");
    for (k, v) in &g5 {
        print!(" {k}:{v}");
    }
    print!("\n66 leftover-high");
    for (k, v) in &g66 {
        print!(" {k}:{v}");
    }
    print!("\n0x4005 leftover-high");
    for (k, v) in &g4 {
        print!(" {k}:{v}");
    }
    print!("\n0x4e05 leftover-high");
    for (k, v) in &g4e {
        print!(" {k}:{v}");
    }
    println!();
}
