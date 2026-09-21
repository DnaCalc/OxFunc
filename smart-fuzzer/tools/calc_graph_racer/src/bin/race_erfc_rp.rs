//! Cephes R[0]/P[1] vs 1/√π; drop P[0]. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const RPINV: f64 = 0.56418958354775628695;
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
fn cephes(x: f64, mask: u32, p: &[f64; 9], r: &[f64; 6]) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (pp, b) = polevl(xe, p, mask, 0);
        let (q, b) = p1evl(xe, &CEPHES_Q, mask, b);
        (pp, q, b)
    } else {
        let (rr, b) = polevl(xe, r, mask, 0);
        let (s, b) = p1evl(xe, &CEPHES_S, mask, b);
        (rr, s, b)
    };
    let mut q = ext_div(&num, &den, CW);
    q = maybe(q, mask, bit0);
    ext_to_f64(&q, CW)
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
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let rp80 = ext_to_f64(&ext_div(&ef(1.0), &ext_sqrt(&ef(std::f64::consts::PI), CW), CW), CW);
    println!(
        "R0={:#x} P1={:#x} RPINV={:#x} x87={:#x}",
        CEPHES_R[0].to_bits(),
        CEPHES_P[1].to_bits(),
        RPINV.to_bits(),
        rp80.to_bits()
    );
    let mut r_rp = CEPHES_R;
    r_rp[0] = RPINV;
    let mut r_80 = CEPHES_R;
    r_80[0] = rp80;
    let mut p_rp = CEPHES_P;
    p_rp[1] = RPINV;
    let mut p_z0 = CEPHES_P;
    p_z0[0] = 0.0;
    let mut both = CEPHES_R;
    both[0] = RPINV;
    let mut p_both = CEPHES_P;
    p_both[1] = RPINV;
    let score = |p: &[f64; 9], r: &[f64; 6]| {
        let mut ex = 0usize;
        let mut dd = 0usize;
        let mut n = 0usize;
        for row in &rows {
            if row.z < 4.0 {
                continue;
            }
            n += 1;
            if ulp_distance(
                qwf(row.z, cephes(row.z, 0x5005, p, r)),
                f64::from_bits(row.qbits),
            )
            .unwrap_or(99)
                == 0
            {
                ex += 1;
                if row.direct {
                    dd += 1;
                }
            }
        }
        (ex, dd, n)
    };
    for (name, p, r) in [
        ("CR 0x5005", &CEPHES_P, &CEPHES_R),
        ("R0=RPINV", &CEPHES_P, &r_rp),
        ("R0=x87 1/√π", &CEPHES_P, &r_80),
        ("P1=RPINV", &p_rp, &CEPHES_R),
        ("P0=0", &p_z0, &CEPHES_R),
        ("R0+P1=RPINV", &p_both, &both),
    ] {
        let (ex, dd, n) = score(p, r);
        println!("{name:14} Q {ex}/{n} d={dd}");
    }
}
