//! PQR R[1]+1 vs 0x5005 z-cuts on Q-tail. Not an identity.
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
const P0: [f64; 8] = [
    0.16506148041280876191828601e-03,
    0.15471455377139313353998665e-03,
    0.44852548090298868465196794e-04,
    -0.49177280017226285450486205e-05,
    -0.69353602078656412367801676e-05,
    -0.20508667787746282746857743e-05,
    -0.28982842617824971177267380e-06,
    -0.17272433544836633301127174e-07,
];
const Q0: [f64; 8] = [
    1.0,
    0.16272656776533322859856317e+01,
    0.12040996037066026106794322e+01,
    0.52400246352158386907601472e+00,
    0.14497345252798672362384241e+00,
    0.25592517111042546492590736e-01,
    0.26869088293991371028123158e-02,
    0.13133767840925681614496481e-03,
];
const R0: [f64; 9] = [
    0.145589721275038539045668824025,
    -0.273421931495426482902320421863,
    0.226008066916621506788789064272,
    -0.163571895523923805648814425592,
    0.102604312032193978662297299832,
    -0.548023266949835519254211506880e-01,
    0.241432239725390106956523668160e-01,
    -0.822062115403915116036874169600e-02,
    0.180296241564687154310619200000e-02,
];
const SIX: [f64; 6] = [
    4.0520833333333330,
    4.0625000000000000,
    5.2395833333333330,
    5.2708333333333330,
    5.3333333333333330,
    6.0000000000000000,
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
fn horner(cs: &[f64], x: f64) -> f64 {
    let mut a = 0.0;
    for &c in cs.iter().rev() {
        a = a * x + c;
    }
    a
}
fn pqr_r1(x: f64) -> f64 {
    let mut r = R0;
    r[1] = poke(R0[1], 1);
    let u = horner(&P0, x);
    let v = horner(&Q0, x);
    let t = (x - 3.75) / (x + 3.75);
    let mut acc = u / v;
    for &c in r.iter().rev() {
        acc = acc * t + c;
    }
    acc
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
    println!("cuts (bar 0x5005 Q 1572 d=53 / R1 Q 459 d=38):");
    let cuts = [4.05, 4.1, 4.5, 5.0, 5.2, 5.35, 5.5, 6.0, 6.1, 8.0];
    let mut best_rc = 0usize;
    let mut best_cr = 0usize;
    let mut lab_rc = String::new();
    let mut lab_cr = String::new();
    for &c in &cuts {
        let mut rc_q = 0usize;
        let mut rc_d = 0usize;
        let mut cr_q = 0usize;
        let mut cr_d = 0usize;
        let mut hit_rc = 0usize;
        let mut hit_cr = 0usize;
        for row in rows.iter().filter(|rr| rr.z >= 4.0) {
            let t = f64::from_bits(row.qbits);
            let gc = qw(row.z, cephes(row.z));
            let gr = qw(row.z, pqr_r1(row.z));
            let g = if row.z < c { gr } else { gc };
            let h = if row.z < c { gc } else { gr };
            if ulp_distance(g, t).unwrap_or(99) == 0 {
                rc_q += 1;
                if row.direct {
                    rc_d += 1;
                }
            }
            if ulp_distance(h, t).unwrap_or(99) == 0 {
                cr_q += 1;
                if row.direct {
                    cr_d += 1;
                }
            }
        }
        for &lz in &SIX {
            let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
                continue;
            };
            let t = f64::from_bits(r.qbits);
            let g = if lz < c {
                qw(lz, pqr_r1(lz))
            } else {
                qw(lz, cephes(lz))
            };
            let h = if lz < c {
                qw(lz, cephes(lz))
            } else {
                qw(lz, pqr_r1(lz))
            };
            if ulp_distance(g, t).unwrap_or(99) == 0 {
                hit_rc += 1;
            }
            if ulp_distance(h, t).unwrap_or(99) == 0 {
                hit_cr += 1;
            }
        }
        println!(
            "  cut={c} R1-then-5005 Q={rc_q} d={rc_d} hit={hit_rc}/6  5005-then-R1 Q={cr_q} d={cr_d} hit={hit_cr}/6"
        );
        if rc_d > best_rc || (rc_d == best_rc && rc_q > best_rc) {
            best_rc = rc_d;
            lab_rc = format!("{c} Q={rc_q}");
        }
        if cr_d > best_cr {
            best_cr = cr_d;
            lab_cr = format!("{c} Q={cr_q}");
        }
    }
    println!("best R1-then-5005 d={best_rc} {lab_rc}");
    println!("best 5005-then-R1 d={best_cr} {lab_cr}");
}
