//! Q-tail 0x5005 leftover hard vs derfc0 compose and named F. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
fn cephes_5005(x: f64) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let mask = 0x5005u32;
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let q5005 = |z: f64| qw(z, cephes_5005(z));
    let mut hard: Vec<&f::QRow> = Vec::new();
    let mut ex0 = 0usize;
    let mut n = 0usize;
    let mut d0 = 0usize;
    for r in &rows {
        if r.z < 4.0 {
            continue;
        }
        let d = ulp_distance(q5005(r.z), f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        n += 1;
        if d == 0 {
            ex0 += 1;
            if r.direct {
                d0 += 1;
            }
        } else if r.direct && d >= 2 {
            hard.push(r);
        }
    }
    println!("0x5005 Q-tail {ex0}/{n} d={d0} hard_direct_ulp>=2 n={}", hard.len());
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 7] = [
        ("derfc0", Box::new(|z| qw(z, f::nswc_derfc0(z)))),
        ("ccdd", Box::new(|z| qw(z, f::nswc_ccdd_f(z)))),
        ("pqr", Box::new(|z| qw(z, f::nswc_pqr_f(z)))),
        ("cephes_f", Box::new(|z| qw(z, f::cephes_f(z)))),
        ("libm", Box::new(|z| libm::erfc(z))),
        (
            "derfc0-then-5005@4.9",
            Box::new(|z| {
                if z < 4.9 {
                    qw(z, f::nswc_derfc0(z))
                } else {
                    q5005(z)
                }
            }),
        ),
        (
            "derfc0-then-5005@5.0",
            Box::new(|z| {
                if z < 5.0 {
                    qw(z, f::nswc_derfc0(z))
                } else {
                    q5005(z)
                }
            }),
        ),
    ];
    for (name, ev) in &graphs {
        let mut hit = 0usize;
        let mut keep = 0usize;
        let mut kn = 0usize;
        let mut mx = 0u64;
        let mut shown = 0usize;
        for r in &rows {
            if r.z < 4.0 {
                continue;
            }
            let qg = ev(r.z);
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            kn += 1;
            if d == 0 {
                keep += 1;
            }
        }
        for r in &hard {
            let d = ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99);
            if d == 0 {
                hit += 1;
                if shown < 8 {
                    println!("  HIT {name} z={:.16}", r.z);
                    shown += 1;
                }
            } else {
                mx = mx.max(d);
            }
        }
        println!("{name:24} hard_hit={hit}/{} keep={keep}/{kn} miss_max={mx}", hard.len());
    }
}
