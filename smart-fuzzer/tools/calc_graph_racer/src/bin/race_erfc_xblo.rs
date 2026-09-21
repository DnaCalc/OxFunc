//! XBIG leftover-low bits vs 0x5005 F / w / last-div. Not an identity.
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    println!("XBIG leftover-low of 0x5005 (dir, ulp>=2, graph<excel, z>26.5):");
    for r in &rows {
        if !r.direct || r.z <= 26.5 || r.z >= 26.544 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let f5 = cephes_mask(r.z, 0x5005);
        let g5 = qw(r.z, f5);
        let d5 = ulp_distance(g5, t).unwrap_or(99);
        let w = f::w_rn53(r.z);
        let fo = f::f_or(r.z, r.qbits);
        println!(
            "z={:.20} excel_q={:#x} w={:#x} 5005F={:#x} 5005Q={:#x} dQ={} dF={} nativeF={:#x} dQn={} w*upF={} w*dnF={} upQ={} dnQ={}",
            r.z,
            r.qbits,
            w.to_bits(),
            f5.to_bits(),
            g5.to_bits(),
            d5,
            fo.map(|fo| ulp_distance(f5, fo).unwrap_or(99)).unwrap_or(99),
            f::cephes_f(r.z).to_bits(),
            ulp_distance(qw(r.z, f::cephes_f(r.z)), t).unwrap_or(99),
            ulp_distance(qw(r.z, f5.next_up()), t).unwrap_or(99),
            ulp_distance(qw(r.z, f5.next_down()), t).unwrap_or(99),
            ulp_distance(g5.next_up(), t).unwrap_or(99),
            ulp_distance(g5.next_down(), t).unwrap_or(99),
        );
    }
    println!("keep of last-div / last-Q nudge on full tail:");
    for (name, ev) in [
        ("5005", Box::new(|z: f64| qw(z, cephes_mask(z, 0x5005))) as Box<dyn Fn(f64) -> f64>),
        ("w*up F", Box::new(|z| qw(z, cephes_mask(z, 0x5005).next_up()))),
        ("w*dn F", Box::new(|z| qw(z, cephes_mask(z, 0x5005).next_down()))),
        ("up(Q)", Box::new(|z| qw(z, cephes_mask(z, 0x5005)).next_up())),
        ("dn(Q)", Box::new(|z| qw(z, cephes_mask(z, 0x5005)).next_down())),
        ("native", Box::new(|z| qw(z, f::cephes_f(z)))),
        ("24a5", Box::new(|z| qw(z, cephes_mask(z, 0x24a5)))),
    ] {
        let mut ex = 0usize;
        let mut hit = 0usize;
        for r in &rows {
            if r.z < 4.0 {
                continue;
            }
            if ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                ex += 1;
                if r.direct && r.z > 26.52 && r.z < 26.544 {
                    hit += 1;
                }
            }
        }
        println!("  {name:8} Q {ex} xwin_hit={hit}");
    }
}
