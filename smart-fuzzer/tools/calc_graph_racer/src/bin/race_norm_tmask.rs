//! NORMSDIST |z|>=4: mask0 x87 cephes last-store of F and ½. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const INVSQRT2: f64 = 0.7071067811865476;
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
fn qmul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn phi_from(z: f64, q: f64) -> f64 {
    if z >= 0.0 {
        1.0 - 0.5 * q
    } else {
        0.5 * q
    }
}
fn hit_ff(z: f64, a: f64, t: f64, ff: f64) -> bool {
    let w53 = f::w_rn53(a);
    let q0 = qmul(w53, ff);
    let phi0 = phi_from(z, q0);
    if ulp_distance(phi0, t).unwrap_or(99) == 0 {
        return true;
    }
    for k in -5i32..=5 {
        if k == 0 {
            continue;
        }
        let q = qmul(w53, poke(ff, k));
        let phi = phi_from(z, q);
        if ulp_distance(phi, t).unwrap_or(99) == 0
            || ulp_distance(poke(phi0, k), t).unwrap_or(99) == 0
            || ulp_distance(poke(phi, k), t).unwrap_or(99) == 0
        {
            return true;
        }
    }
    false
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let path = format!("{dir}/answers-b24-normref.json");
    let bank: WitnessSet = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let mut n = 0usize;
    let mut h_cf = 0usize;
    let mut h_m0 = 0usize;
    let mut h_u = 0usize;
    let mut h_x87 = 0usize;
    let mut miss = 0usize;
    println!("NORMSDIST |z|>=4 mask0 vs cephes_f last-store:");
    for w in &bank.witnesses {
        let x = match &w.args[0] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => continue,
        };
        let cum = match &w.args[1] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => continue,
        };
        if cum != 1.0 {
            continue;
        }
        let Some(e) = parse_bits_hex(&w.expected_bits) else {
            continue;
        };
        let t = e;
        let z = x * INVSQRT2;
        let a = z.abs();
        if a < 4.0 {
            continue;
        }
        n += 1;
        let cf = hit_ff(z, a, t, f::cephes_f(a));
        let m0 = hit_ff(z, a, t, cephes_mask(a, 0));
        let zx = ext_to_f64(&ext_mul(&ef(x), &ef(INVSQRT2), CW), CW);
        let ax = zx.abs();
        let xz = hit_ff(zx, ax, t, f::cephes_f(ax)) || hit_ff(zx, ax, t, cephes_mask(ax, 0));
        if cf {
            h_cf += 1;
        }
        if m0 {
            h_m0 += 1;
        }
        if xz {
            h_x87 += 1;
        }
        if cf || m0 || xz {
            h_u += 1;
        } else {
            miss += 1;
            if miss <= 8 {
                println!("  MISS x={x:.16} a={a:.16}");
            }
        }
    }
    println!(
        "n={n} cephes_f-glue={h_cf} mask0-glue={h_m0} x87z={h_x87} union={h_u}/{n} miss={miss}"
    );
}
