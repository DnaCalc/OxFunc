//! 0x5005 CR vs F+1 z-cuts DIRECT z>=4. Not an identity.
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
fn cephes(x: f64) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl(xe, &CEPHES_P, MASK, 0);
        let (q, b) = p1evl(xe, &CEPHES_Q, MASK, b);
        (p, q, b)
    } else {
        let (r, b) = polevl(xe, &CEPHES_R, MASK, 0);
        let (s, b) = p1evl(xe, &CEPHES_S, MASK, b);
        (r, s, b)
    };
    let mut v = ext_div(&num, &den, CW);
    v = maybe(v, MASK, bit0);
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let tail: Vec<&f::QRow> = rows.iter().filter(|r| r.direct && r.z >= 4.0).collect();
    let mut both = 0usize;
    let mut only_c = 0usize;
    let mut only_f = 0usize;
    for r in &tail {
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let ff = cephes(r.z);
        let ec = ulp_distance(mul(w, ff), t).unwrap_or(99) == 0;
        let ef1 = ulp_distance(mul(w, poke(ff, 1)), t).unwrap_or(99) == 0;
        match (ec, ef1) {
            (true, true) => both += 1,
            (true, false) => only_c += 1,
            (false, true) => only_f += 1,
            _ => {}
        }
    }
    println!(
        "DIRECT z>=4 5005 vs F+1 both={both} only_5005={only_c} only_F1={only_f} union={} n={}",
        both + only_c + only_f,
        tail.len()
    );
    let cuts = [4.5, 5.0, 5.5, 6.0, 8.0, 16.0, 26.0];
    let mut best = 0usize;
    let mut bestc = 0.0;
    println!("z-cut 5005-then-F+1 / F+1-then-5005:");
    for &c in &cuts {
        let mut a = 0usize;
        let mut b = 0usize;
        for r in &tail {
            let t = f64::from_bits(r.qbits);
            let w = f::w_rn53(r.z);
            let ff = cephes(r.z);
            let ec = ulp_distance(mul(w, ff), t).unwrap_or(99) == 0;
            let ef1 = ulp_distance(mul(w, poke(ff, 1)), t).unwrap_or(99) == 0;
            if r.z < c {
                if ec {
                    a += 1;
                }
                if ef1 {
                    b += 1;
                }
            } else {
                if ef1 {
                    a += 1;
                }
                if ec {
                    b += 1;
                }
            }
        }
        if a > best {
            best = a;
            bestc = c;
        }
        println!("  cut={c} 5005-then-F1={a} F1-then-5005={b}");
    }
    println!("best 5005-then-F1={best} @{bestc} (bar d=53)");
}
