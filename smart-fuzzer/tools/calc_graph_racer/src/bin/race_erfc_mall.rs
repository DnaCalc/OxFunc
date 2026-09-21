//! All Q z in [0.5,4) and z>=4: published F fused ∪ F± cover. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const C0: [f64; 9] = [
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
const D0: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];
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
fn cody_mask(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
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
fn hit_ls(z: f64, t: f64, ff: f64, lo: i32, hi: i32) -> bool {
    (lo..=hi)
        .filter(|&k| k != 0)
        .any(|k| ulp_distance(mul(f::w_rn53(z), poke(ff, k)), t).unwrap_or(99) == 0)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut qn = 0usize;
    let mut qh = 0usize;
    let mut dn = 0usize;
    let mut dh = 0usize;
    let mut miss_q = 0usize;
    for r in rows.iter().filter(|rr| rr.z >= 0.5 && rr.z < 4.0) {
        qn += 1;
        if r.direct {
            dn += 1;
        }
        let t = f64::from_bits(r.qbits);
        let ff = cody_mask(r.z, 0);
        let hit = ulp_distance(mul(f::w_rn53(r.z), ff), t).unwrap_or(99) == 0
            || hit_ls(r.z, t, ff, -4, 4);
        if hit {
            qh += 1;
            if r.direct {
                dh += 1;
            }
        } else {
            miss_q += 1;
            if miss_q <= 8 {
                println!(
                    "  MID-MISS z={:.16} direct={} d={}",
                    r.z,
                    r.direct,
                    ulp_distance(mul(f::w_rn53(r.z), ff), t).unwrap_or(99)
                );
            }
        }
    }
    println!("Q [0.5,4) unmask∪F±={qh}/{qn} DIRECT={dh}/{dn} missQ={miss_q}");
    let mut tqn = 0usize;
    let mut tqh = 0usize;
    let mut tdn = 0usize;
    let mut tdh = 0usize;
    let mut tmiss = 0usize;
    for r in rows.iter().filter(|rr| rr.z >= 4.0) {
        tqn += 1;
        if r.direct {
            tdn += 1;
        }
        let t = f64::from_bits(r.qbits);
        let ff = cephes_mask(r.z, 0);
        let cf = f::cephes_f(r.z);
        let hit = ulp_distance(mul(f::w_rn53(r.z), ff), t).unwrap_or(99) == 0
            || hit_ls(r.z, t, ff, -5, 5)
            || ulp_distance(mul(f::w_rn53(r.z), cf), t).unwrap_or(99) == 0
            || hit_ls(r.z, t, cf, -5, 5);
        if hit {
            tqh += 1;
            if r.direct {
                tdh += 1;
            }
        } else {
            tmiss += 1;
            if tmiss <= 8 {
                println!(
                    "  TAIL-MISS z={:.16} direct={} d={}",
                    r.z,
                    r.direct,
                    ulp_distance(mul(f::w_rn53(r.z), ff), t).unwrap_or(99)
                );
            }
        }
    }
    println!("Q z>=4 mask0∪cephes_f∪F±={tqh}/{tqn} DIRECT={tdh}/{tdn} missQ={tmiss}");
}
