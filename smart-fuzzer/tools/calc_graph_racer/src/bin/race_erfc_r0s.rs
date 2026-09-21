//! 0x5005 R[0]/S[0] steps vs Q-tail leftover-low / XBIG. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const MASK: u32 = 0x5005;
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
const XBIG: [f64; 3] = [26.53, 26.542, 26.543];

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
fn polevl_mask(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ef(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn p1evl_mask(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
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
fn cephes(x: f64, r: &[f64; 6], s: &[f64; 6]) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl_mask(xe, &CEPHES_P, MASK, 0);
        let (q, b) = p1evl_mask(xe, &CEPHES_Q, MASK, b);
        (p, q, b)
    } else {
        let (rr, b) = polevl_mask(xe, r, MASK, 0);
        let (ss, b) = p1evl_mask(xe, s, MASK, b);
        (rr, ss, b)
    };
    let mut q = ext_div(&num, &den, CW);
    q = maybe(q, MASK, bit0);
    ext_to_f64(&q, CW)
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let tail: Vec<&f::QRow> = rows.iter().filter(|r| r.direct && r.z >= 4.0).collect();
    let mut lows: Vec<f64> = Vec::new();
    for r in &tail {
        let g = qw(r.z, cephes(r.z, &CEPHES_R, &CEPHES_S));
        let t = f64::from_bits(r.qbits);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d >= 2 && g < t {
            lows.push(r.z);
        }
    }
    println!(
        "0x5005 R[0]/S[0] steps (bar d=53 leftover-low {} / n={}):",
        lows.len(),
        tail.len()
    );
    let run = |lab: &str, r: &[f64; 6], s: &[f64; 6]| {
        let mut qd = 0usize;
        let mut qe = 0usize;
        let mut hit = 0usize;
        let mut xhit = 0usize;
        for row in rows.iter().filter(|rr| rr.z >= 4.0) {
            let g = qw(row.z, cephes(row.z, r, s));
            let d = ulp_distance(g, f64::from_bits(row.qbits)).unwrap_or(99);
            if d == 0 {
                qe += 1;
                if row.direct {
                    qd += 1;
                }
            }
        }
        for &lz in &lows {
            let Some(row) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
                continue;
            };
            if ulp_distance(qw(lz, cephes(lz, r, s)), f64::from_bits(row.qbits)).unwrap_or(99) == 0 {
                hit += 1;
            }
        }
        for &xz in &XBIG {
            if let Some(row) = tail.iter().find(|rr| (rr.z - xz).abs() < 0.0006) {
                if ulp_distance(qw(row.z, cephes(row.z, r, s)), f64::from_bits(row.qbits)).unwrap_or(99)
                    == 0
                {
                    xhit += 1;
                }
            }
        }
        println!(
            "  {lab} Q={qe} d={qd} leftover-low {hit}/{} xhit={xhit}/3",
            lows.len()
        );
    };
    println!("R[0] steps:");
    for k in 0i32..=8 {
        let mut r = CEPHES_R;
        r[0] = poke(CEPHES_R[0], k);
        run(&format!("R[0] {k:+}"), &r, &CEPHES_S);
    }
    println!("S[0] steps:");
    for k in -8i32..=2 {
        let mut s = CEPHES_S;
        s[0] = poke(CEPHES_S[0], k);
        run(&format!("S[0] {k:+}"), &CEPHES_R, &s);
    }
}
