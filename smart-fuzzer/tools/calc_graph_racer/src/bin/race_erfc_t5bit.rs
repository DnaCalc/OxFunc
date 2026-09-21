//! Exhaustive 5-bit Cephes Horner stores DIRECT z>=4. Not an identity.
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let tail: Vec<&f::QRow> = rows.iter().filter(|r| r.direct && r.z >= 4.0).collect();
    let n = tail.len();
    let score = |mask: u32| -> usize {
        tail.iter()
            .filter(|r| {
                let t = f64::from_bits(r.qbits);
                ulp_distance(mul(f::w_rn53(r.z), cephes_mask(r.z, mask)), t).unwrap_or(99) == 0
            })
            .count()
    };
    println!(
        "DIRECT z>=4 n={n} unmask={} 0x4d={} 0x4805={} 0x4005={}",
        score(0),
        score(0x4d),
        score(0x4805),
        score(0x4005)
    );
    println!("0x4d ∪ extra bit / 0x4805 ∪ extra bit:");
    for &(base, bname) in &[(0x4du32, "0x4d"), (0x4805u32, "0x4805")] {
        let sbase = score(base);
        for b in 0..17u32 {
            if base & (1u32 << b) != 0 {
                continue;
            }
            let m = base | (1u32 << b);
            let s = score(m);
            let d = s as i32 - sbase as i32;
            if d != 0 {
                println!("  {bname}∪bit{b} mask={m:#x} fused={s} Δ={d:+}");
            }
        }
    }
    let mut ranked: Vec<(usize, u32)> = Vec::new();
    for a in 0..17u32 {
        for b in (a + 1)..17 {
            for c in (b + 1)..17 {
                for d in (c + 1)..17 {
                    for e in (d + 1)..17 {
                        let mask =
                            (1u32 << a) | (1u32 << b) | (1u32 << c) | (1u32 << d) | (1u32 << e);
                        ranked.push((score(mask), mask));
                    }
                }
            }
        }
    }
    ranked.sort_by(|x, y| y.0.cmp(&x.0).then(x.1.cmp(&y.1)));
    println!("top 12 5-bit:");
    for (s, mask) in ranked.iter().take(12) {
        println!("  fused={s} mask={mask:#x}");
    }
    let best = ranked[0].0;
    let nbest = ranked.iter().filter(|t| t.0 == best).count();
    let nge58 = ranked.iter().filter(|t| t.0 >= 58).count();
    let ngt58 = ranked.iter().filter(|t| t.0 > 58).count();
    println!(
        "best 5-bit={best} mask={:#x} unique={} nbest={nbest} ge58={nge58} gt58={ngt58} n5={}",
        ranked[0].1,
        nbest == 1,
        ranked.len()
    );
    println!("all fused>58:");
    for (s, mask) in ranked.iter().filter(|t| t.0 > 58) {
        println!("  fused={s} mask={mask:#x}");
    }
    println!("all fused==58 5-bit:");
    for (s, mask) in ranked.iter().filter(|t| t.0 == 58) {
        println!("  fused={s} mask={mask:#x}");
    }
}
