//! Cephes printed-decimal parse vs IEEE consts as 0x5005 Q-tail. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const P_LIT: [&str; 9] = [
    "2.46196981473530512524e-10",
    "5.64189564831068821977e-1",
    "7.46321056442269912687e0",
    "4.86371970985681366614e1",
    "1.96520832956077098242e2",
    "5.26445194995477358631e2",
    "9.34528527171957607540e2",
    "1.02755188689515710272e3",
    "5.57535335369399327526e2",
];
const Q_LIT: [&str; 8] = [
    "1.32281951154744992508e1",
    "8.67072140885989742329e1",
    "3.54937778887819891062e2",
    "9.75708501743205489753e2",
    "1.82390916687909736289e3",
    "2.24633760818710981792e3",
    "1.65666309194161350182e3",
    "5.57535340817727675546e2",
];
const R_LIT: [&str; 6] = [
    "5.64189583547755073984e-1",
    "1.27536670759978104416e0",
    "5.01905042251180477414e0",
    "6.16021097993053585195e0",
    "7.40974269950448939160e0",
    "2.97886665372100240670e0",
];
const S_LIT: [&str; 6] = [
    "2.26052863220117276590e0",
    "9.39603524938001434673e0",
    "1.20489539808096656605e1",
    "1.70814450747565897222e1",
    "9.60896809063285878198e0",
    "3.36907645100081516050e0",
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

fn parse_arr<const N: usize>(lits: &[&str; N]) -> [f64; N] {
    let mut a = [0.0; N];
    for i in 0..N {
        a[i] = lits[i].parse::<f64>().unwrap();
    }
    a
}
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
fn cephes(x: f64, mask: u32, p: &[f64], q: &[f64], r: &[f64], s: &[f64]) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (pp, b) = polevl_mask(xe, p, mask, 0);
        let (qq, b) = p1evl_mask(xe, q, mask, b);
        (pp, qq, b)
    } else {
        let (rr, b) = polevl_mask(xe, r, mask, 0);
        let (ss, b) = p1evl_mask(xe, s, mask, b);
        (rr, ss, b)
    };
    let mut v = ext_div(&num, &den, CW);
    v = maybe(v, mask, bit0);
    ext_to_f64(&v, CW)
}
fn qwf(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let p = parse_arr(&P_LIT);
    let q = parse_arr(&Q_LIT);
    let r = parse_arr(&R_LIT);
    let s = parse_arr(&S_LIT);
    let mut ndiff = 0usize;
    for i in 0..9 {
        if p[i].to_bits() != CEPHES_P[i].to_bits() {
            ndiff += 1;
            println!("P[{i}] lit={:#x} ieee={:#x}", p[i].to_bits(), CEPHES_P[i].to_bits());
        }
    }
    for i in 0..8 {
        if q[i].to_bits() != CEPHES_Q[i].to_bits() {
            ndiff += 1;
            println!("Q[{i}] lit={:#x} ieee={:#x}", q[i].to_bits(), CEPHES_Q[i].to_bits());
        }
    }
    for i in 0..6 {
        if r[i].to_bits() != CEPHES_R[i].to_bits() {
            ndiff += 1;
            println!("R[{i}] lit={:#x} ieee={:#x}", r[i].to_bits(), CEPHES_R[i].to_bits());
        }
        if s[i].to_bits() != CEPHES_S[i].to_bits() {
            ndiff += 1;
            println!("S[{i}] lit={:#x} ieee={:#x}", s[i].to_bits(), CEPHES_S[i].to_bits());
        }
    }
    println!("decimal vs IEEE coeff diffs n={ndiff}/29");
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut e_ieee = 0usize;
    let mut e_lit = 0usize;
    let mut n = 0usize;
    let mut dmid_i = 0usize;
    let mut dmid_l = 0usize;
    for row in &rows {
        if row.z < 4.0 {
            continue;
        }
        n += 1;
        let t = f64::from_bits(row.qbits);
        let di = ulp_distance(
            qwf(row.z, cephes(row.z, 0x5005, &CEPHES_P, &CEPHES_Q, &CEPHES_R, &CEPHES_S)),
            t,
        )
        .unwrap_or(99);
        let dl = ulp_distance(qwf(row.z, cephes(row.z, 0x5005, &p, &q, &r, &s)), t).unwrap_or(99);
        if di == 0 {
            e_ieee += 1;
            if row.direct {
                dmid_i += 1;
            }
        }
        if dl == 0 {
            e_lit += 1;
            if row.direct {
                dmid_l += 1;
            }
        }
    }
    println!("0x5005 IEEE Q {e_ieee}/{n} d={dmid_i}  lit Q {e_lit}/{n} d={dmid_l}");
}
