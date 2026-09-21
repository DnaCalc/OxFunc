//! 3-piece Cephes-mask Q-tail around 6.1/8; CCDD prefix. Not an identity.
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
fn cephes_mask(x: f64, mask: u32) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl_mask(xe, &CEPHES_P, mask, 0);
        let (q, b) = p1evl_mask(xe, &CEPHES_Q, mask, b);
        (p, q, b)
    } else {
        let (r, b) = polevl_mask(xe, &CEPHES_R, mask, 0);
        let (s, b) = p1evl_mask(xe, &CEPHES_S, mask, b);
        (r, s, b)
    };
    let mut q = ext_div(&num, &den, CW);
    q = maybe(q, mask, bit0);
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
fn score(rows: &[f::QRow], ff: impl Fn(f64) -> f64) -> (usize, usize, usize, usize) {
    let mut qe = 0usize;
    let mut qd = 0usize;
    let mut fe = 0usize;
    let mut fd = 0usize;
    for r in rows {
        if r.z < 4.0 {
            continue;
        }
        let fval = ff(r.z);
        if ulp_distance(qw(r.z, fval), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
            qe += 1;
            if r.direct {
                qd += 1;
            }
        }
        if let Some(fo) = f::f_or(r.z, r.qbits) {
            if ulp_distance(fval, fo).unwrap_or(99) == 0 {
                fe += 1;
                if r.direct {
                    fd += 1;
                }
            }
        }
    }
    (qe, qd, fe, fd)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    println!("bars Q 1572/53 F_or 1355/34; 24a5-then-5005@6.1 Q 1599/59 F_or 1379/39; mixed F_or tail 1370");

    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 10] = [
        ("5005", Box::new(|z| cephes_mask(z, 0x5005))),
        ("24a5", Box::new(|z| cephes_mask(z, 0x24a5))),
        ("24a5<6.1 else 5005", Box::new(|z| cephes_mask(z, if z < 6.1 { 0x24a5 } else { 0x5005 }))),
        ("24a5<6|z>=8 else 5005", Box::new(|z| {
            let m = if z < 6.1 || z >= 8.0 { 0x24a5 } else { 0x5005 };
            cephes_mask(z, m)
        })),
        ("24a5<6.1 / 5005 / mask0>=8", Box::new(|z| {
            let m = if z < 6.1 {
                0x24a5
            } else if z < 8.0 {
                0x5005
            } else {
                0
            };
            cephes_mask(z, m)
        })),
        ("24a5<6 / 5005 / 24a5>=8", Box::new(|z| {
            let m = if z < 6.0 || z >= 8.0 { 0x24a5 } else { 0x5005 };
            cephes_mask(z, m)
        })),
        ("CCDD<4.9 else 5005", Box::new(|z| {
            if z < 4.9 { f::nswc_ccdd_f(z) } else { cephes_mask(z, 0x5005) }
        })),
        ("CCDD<5.7 else 5005", Box::new(|z| {
            if z < 5.7 { f::nswc_ccdd_f(z) } else { cephes_mask(z, 0x5005) }
        })),
        ("CCDD<6.1 else 24a5-5005", Box::new(|z| {
            if z < 6.1 {
                f::nswc_ccdd_f(z)
            } else {
                cephes_mask(z, 0x5005)
            }
        })),
        ("CCDD<6.1 / 24a5< wait 5005", Box::new(|z| {
            if z < 6.1 {
                f::nswc_ccdd_f(z)
            } else {
                cephes_mask(z, if z < 6.1 { 0x24a5 } else { 0x5005 })
            }
        })),
    ];
    for (name, ev) in &graphs {
        let (qe, qd, fe, fd) = score(&rows, ev);
        println!("{name:34} Q {qe} d={qd}  F_or {fe} d={fd}");
    }

    println!("CCDD-then-24a5-then-5005 c1 in [4.5,6.5] c2=6.1 (bar 1599/1379):");
    let mut best = (0usize, 0usize, 0usize, 0usize, 0.0);
    for k in 0..=40 {
        let c1 = 4.5 + k as f64 * 0.05;
        if c1 >= 6.1 {
            break;
        }
        let (qe, qd, fe, fd) = score(&rows, |z| {
            if z < c1 {
                f::nswc_ccdd_f(z)
            } else if z < 6.1 {
                cephes_mask(z, 0x24a5)
            } else {
                cephes_mask(z, 0x5005)
            }
        });
        if fe > best.2 || (fe == best.2 && qe > best.0) {
            best = (qe, qd, fe, fd, c1);
        }
        if (c1 * 2.0 - (c1 * 2.0).round()).abs() < 1e-12 {
            println!("  c1={c1:.2} Q {qe} d={qd} F_or {fe} d={fd}");
        }
    }
    println!(
        "  BEST F_or {} d={} Q {} d={} @ c1={:.2}",
        best.2, best.3, best.0, best.1, best.4
    );
}
