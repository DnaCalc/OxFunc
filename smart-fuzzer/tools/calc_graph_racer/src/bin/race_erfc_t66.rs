//! 0x24a5 + P[4]-1 Q[0]+1 Q[1]-1 Q[3]+1 leftover. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const MASK: u32 = 0x24a5;
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
fn cephes(x: f64, p: &[f64; 9], q: &[f64; 8]) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (pp, b) = polevl(xe, p, MASK, 0);
        let (qq, b) = p1evl(xe, q, MASK, b);
        (pp, qq, b)
    } else {
        let (rr, b) = polevl(xe, &R0, MASK, 0);
        let (ss, b) = p1evl(xe, &S0, MASK, b);
        (rr, ss, b)
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
fn hit(z: f64, t: f64, p: &[f64; 9], q: &[f64; 8]) -> bool {
    ulp_distance(mul(f::w_rn53(z), cephes(z, p, q)), t).unwrap_or(99) == 0
}
fn signed_k(z: f64, t: f64, p: &[f64; 9], q: &[f64; 8]) -> Option<i32> {
    let ff = cephes(z, p, q);
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
fn apply(idx: usize, k: i32, p: &mut [f64; 9], q: &mut [f64; 8]) {
    if idx < 9 {
        p[idx] = poke(P0[idx], k);
    } else {
        q[idx - 9] = poke(Q0[idx - 9], k);
    }
}
fn name(idx: usize) -> String {
    if idx < 9 {
        format!("P[{idx}]")
    } else {
        format!("Q[{}]", idx - 9)
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let tail: Vec<&f::QRow> = rows.iter().filter(|r| r.direct && r.z >= 4.0).collect();
    let n = tail.len();
    let mut p66 = P0;
    let mut q66 = Q0;
    p66[4] = poke(P0[4], -1);
    q66[0] = poke(Q0[0], 1);
    q66[1] = poke(Q0[1], -1);
    q66[3] = poke(Q0[3], 1);
    let mut both = 0usize;
    let mut only_m = 0usize;
    let mut only_b = 0usize;
    let mut nf = 0usize;
    let mut nlo = 0usize;
    let mut hit_lo = 0usize;
    let mut nhi = 0usize;
    let mut hit_hi = 0usize;
    let mut slo = 0usize;
    let mut slo_c = 0usize;
    let mut shi = 0usize;
    let mut shi_c = 0usize;
    let mut n1 = 0usize;
    let mut u1 = 0usize;
    println!("DIRECT z>=4 n={n} 66 vs 0x24a5 leftover:");
    for r in &tail {
        let t = f64::from_bits(r.qbits);
        let em = hit(r.z, t, &p66, &q66);
        let eb = hit(r.z, t, &P0, &Q0);
        if em {
            nf += 1;
        }
        match (em, eb) {
            (true, true) => both += 1,
            (true, false) => only_m += 1,
            (false, true) => only_b += 1,
            _ => {}
        }
        let gb = mul(f::w_rn53(r.z), cephes(r.z, &P0, &Q0));
        let d0 = ulp_distance(gb, t).unwrap_or(99);
        if d0 >= 2 && gb < t {
            nlo += 1;
            if em {
                hit_lo += 1;
            }
        }
        if d0 >= 2 && gb > t {
            nhi += 1;
            if em {
                hit_hi += 1;
            }
        }
        let g = mul(f::w_rn53(r.z), cephes(r.z, &p66, &q66));
        let d = ulp_distance(g, t).unwrap_or(99);
        let k = signed_k(r.z, t, &p66, &q66);
        if d == 1 {
            n1 += 1;
            if k.is_some() {
                u1 += 1;
            }
        } else if d >= 2 && g < t {
            slo += 1;
            if matches!(k, Some(kk) if kk > 0) {
                slo_c += 1;
            }
        } else if d >= 2 && g > t {
            shi += 1;
            if matches!(k, Some(kk) if kk < 0) {
                shi_c += 1;
            }
        }
    }
    println!(
        "fused={nf} both={both} only_m={only_m} only_24a5={only_b} leftover-low {hit_lo}/{nlo} leftover-high {hit_hi}/{nhi}"
    );
    println!("self leftover-low {slo_c}/{slo} leftover-high {shi_c}/{shi} 1-ULP {u1}/{n1}");
    println!("0x24a5 leftover-low hits:");
    for r in &tail {
        let t = f64::from_bits(r.qbits);
        let gb = mul(f::w_rn53(r.z), cephes(r.z, &P0, &Q0));
        let d = ulp_distance(gb, t).unwrap_or(99);
        if d >= 2 && gb < t {
            println!(
                "  z={:.16} d={d} 24a5={} 66={}",
                r.z,
                hit(r.z, t, &P0, &Q0),
                hit(r.z, t, &p66, &q66)
            );
        }
    }
    let score = |p: &[f64; 9], q: &[f64; 8]| -> usize {
        tail.iter()
            .filter(|rr| {
                let t = f64::from_bits(rr.qbits);
                ulp_distance(mul(f::w_rn53(rr.z), cephes(rr.z, p, q)), t).unwrap_or(99) == 0
            })
            .count()
    };
    let used = [4usize, 9, 10, 12];
    println!("66 ∪ extra coeff ±1:");
    let mut best = nf;
    for i in 0..17usize {
        if used.contains(&i) {
            continue;
        }
        for &si in &[-1i32, 1] {
            let mut p = p66;
            let mut q = q66;
            apply(i, si, &mut p, &mut q);
            let s = score(&p, &q);
            let d = s as i32 - nf as i32;
            if d != 0 {
                println!("  ∪{}{si:+} fused={s} Δ={d:+}", name(i));
            }
            if s > best {
                best = s;
            }
        }
    }
    println!("best 66∪extra={best}");
}
