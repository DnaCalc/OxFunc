//! 555-mantissa leftover census vs DIRECT baseline. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const MASK210: u32 = 0x210;
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
fn cody(y: f64, c: &[f64; 9], mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn is555(z: f64) -> (bool, bool) {
    let h = format!("{:x}", z.to_bits());
    let strong = h.contains("55555555") || h.contains("aaaaaaaa");
    let m = z.to_bits() & ((1u64 << 52) - 1);
    let weak = (m & 0xfff) == 0x555 || (m & 0xfff) == 0xaaa;
    (weak, strong)
}
fn bump3(c: &mut [usize; 3], z: f64) {
    c[0] += 1;
    let (weak, strong) = is555(z);
    if weak {
        c[1] += 1;
    }
    if strong {
        c[2] += 1;
    }
}
fn bump(n: &mut usize, w: &mut usize, s: &mut usize, z: f64) {
    *n += 1;
    let (weak, strong) = is555(z);
    if weak {
        *w += 1;
    }
    if strong {
        *s += 1;
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut c114 = C0;
    c114[3] = poke(C0[3], -1);
    c114[4] = poke(C0[4], 1);
    c114[5] = poke(C0[5], -1);
    c114[6] = poke(C0[6], 1);
    let mut p66 = P0;
    let mut q66 = Q0;
    p66[4] = poke(P0[4], -1);
    q66[0] = poke(Q0[0], 1);
    q66[1] = poke(Q0[1], -1);
    q66[3] = poke(Q0[3], 1);
    let mut mid_n = 0usize;
    let mut mid_w = 0usize;
    let mut mid_s = 0usize;
    let mut t_n = 0usize;
    let mut t_w = 0usize;
    let mut t_s = 0usize;
    let mut mlo = [[0usize; 3]; 4];
    let mut mhi = [[0usize; 3]; 4];
    let mut tlo = [[0usize; 3]; 5];
    let mut thi = [[0usize; 3]; 5];
    let mid_names = ["unmask", "0x210", "114"];
    let tail_names = ["unmask", "0x4005", "0x4e05", "0x5005", "66"];
    for r in rows.iter().filter(|rr| rr.direct) {
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        if r.z >= 0.5 && r.z < 4.0 {
            bump(&mut mid_n, &mut mid_w, &mut mid_s, r.z);
            let gs = [
                mul(w, cody(r.z, &C0, 0)),
                mul(w, cody(r.z, &C0, MASK210)),
                mul(w, cody(r.z, &c114, MASK210)),
            ];
            for (i, g) in gs.iter().enumerate() {
                let d = ulp_distance(*g, t).unwrap_or(99);
                if d >= 2 && *g < t {
                    bump3(&mut mlo[i], r.z);
                } else if d >= 2 && *g > t {
                    bump3(&mut mhi[i], r.z);
                }
            }
        }
        if r.z >= 4.0 {
            bump(&mut t_n, &mut t_w, &mut t_s, r.z);
            let gs = [
                mul(w, cephes(r.z, &P0, &Q0, 0)),
                mul(w, cephes(r.z, &P0, &Q0, 0x4005)),
                mul(w, cephes(r.z, &P0, &Q0, 0x4e05)),
                mul(w, cephes(r.z, &P0, &Q0, 0x5005)),
                mul(w, cephes(r.z, &p66, &q66, 0x24a5)),
            ];
            for (i, g) in gs.iter().enumerate() {
                let d = ulp_distance(*g, t).unwrap_or(99);
                if d >= 2 && *g < t {
                    bump3(&mut tlo[i], r.z);
                } else if d >= 2 && *g > t {
                    bump3(&mut thi[i], r.z);
                }
            }
        }
    }
    println!(
        "DIRECT mid n={mid_n} weak555={mid_w} strong555={mid_s}  tail n={t_n} weak555={t_w} strong555={t_s}"
    );
    for (i, name) in mid_names.iter().enumerate() {
        println!(
            "  mid {name} leftover-low n={} weak={} strong={} leftover-high n={} weak={} strong={}",
            mlo[i][0], mlo[i][1], mlo[i][2], mhi[i][0], mhi[i][1], mhi[i][2]
        );
    }
    for (i, name) in tail_names.iter().enumerate() {
        println!(
            "  tail {name} leftover-low n={} weak={} strong={} leftover-high n={} weak={} strong={}",
            tlo[i][0], tlo[i][1], tlo[i][2], thi[i][0], thi[i][1], thi[i][2]
        );
    }
}
