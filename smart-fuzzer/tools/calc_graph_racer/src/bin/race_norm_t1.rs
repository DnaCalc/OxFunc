//! 1 NORMSDIST tail leftover x≈−29.6. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const INVSQRT2: f64 = 0.7071067811865476;
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
fn rs(x: f64, mask: u32) -> f64 {
    let xe = ef(x.abs());
    let (r, b) = polevl(xe, &CEPHES_R, mask, 0);
    let (s, b) = p1evl(xe, &CEPHES_S, mask, b);
    let mut v = ext_div(&r, &s, CW);
    v = maybe(v, mask, b);
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let path = format!("{dir}/answers-b24-normref.json");
    let bank: WitnessSet = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let target = -29.5986386669432804;
    let Some(w) = bank.witnesses.iter().find(|ww| {
        let x = match &ww.args[0] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => return false,
        };
        (x - target).abs() < 1e-12
    }) else {
        println!("MISSING");
        return;
    };
    let t = parse_bits_hex(&w.expected_bits).unwrap();
    let x = target;
    let z = x * INVSQRT2;
    let a = z.abs();
    println!("x={x:.16} z={z:.16} a={a:.16} t={t:e}");
    let w53 = f::w_rn53(a);
    println!("w53={w53:e} libm_erfc={}", libm::erfc(a));
    for (name, ff) in [
        ("cephes_f", f::cephes_f(a)),
        ("rs0", rs(a, 0)),
        ("rs5005", rs(a, 0x5005)),
        ("libmF", {
            let w = f::w_rn53(a);
            if w == 0.0 {
                0.0
            } else {
                libm::erfc(a) / w
            }
        }),
    ] {
        let q = qmul(w53, ff);
        let phi = 0.5 * q; // x<0
        let d = ulp_distance(phi, t).unwrap_or(99);
        print!("  {name} 0.5Q={}{}", d, if phi < t { "L" } else { "H" });
        for k in 1..=12 {
            let qk = qmul(w53, poke(ff, k));
            let ph = 0.5 * qk;
            if ulp_distance(ph, t).unwrap_or(99) == 0 {
                print!(" F+{k}");
            }
            let qk = qmul(w53, poke(ff, -k));
            let ph = 0.5 * qk;
            if ulp_distance(ph, t).unwrap_or(99) == 0 {
                print!(" F-{k}");
            }
            if ulp_distance(poke(phi, k), t).unwrap_or(99) == 0 {
                print!(" phi+{k}");
            }
            if ulp_distance(poke(phi, -k), t).unwrap_or(99) == 0 {
                print!(" phi-{k}");
            }
        }
        println!();
    }
    let zx = ext_to_f64(&ext_mul(&ef(x), &ef(INVSQRT2), CW), CW);
    println!(
        "x87 z={:.16} 0.5*libm_erfc={}",
        zx.abs(),
        ulp_distance(0.5 * libm::erfc(zx.abs()), t).unwrap_or(99)
    );
    println!(
        "0.5*libm_erfc(a)={}",
        ulp_distance(0.5 * libm::erfc(a), t).unwrap_or(99)
    );
}
