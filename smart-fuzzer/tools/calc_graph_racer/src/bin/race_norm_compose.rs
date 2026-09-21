//! NORM.S.DIST(x,TRUE) vs erf/erfc compositions. Last-bit. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const SQRT2: f64 = 1.4142135623730951;
const INVSQRT2: f64 = 0.7071067811865476;
const C: [f64; 9] = [
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
const D: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
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
fn cody_f(y: f64, mask: u32) -> f64 {
    let y = y.abs();
    let ye = ef(y);
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(&ext_add(&xnum, &ef(C[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW);
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn erfc_cody(z: f64, mask: u32) -> f64 {
    let a = z.abs();
    let q = f::w_rn53(a) * cody_f(a, mask);
    if z >= 0.0 {
        q
    } else {
        2.0 - q
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let path = format!("{dir}/answers-b24-normref.json");
    let bank: WitnessSet = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let mut rows = Vec::new();
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
        rows.push((x, e.to_bits()));
    }
    println!("NORM.S.DIST cumulative rows={}", rows.len());

    let graphs: [(&str, fn(f64) -> f64); 10] = [
        ("0.5*erfc(-x/sqrt2) libm", |x| 0.5 * libm::erfc(-x / SQRT2)),
        ("0.5*erfc(-x*invsqrt2) libm", |x| 0.5 * libm::erfc(-x * INVSQRT2)),
        ("0.5+0.5*erf(x/sqrt2) libm", |x| 0.5 + 0.5 * libm::erf(x / SQRT2)),
        ("0.5+0.5*erf(x*invsqrt2) libm", |x| 0.5 + 0.5 * libm::erf(x * INVSQRT2)),
        ("0.5*erfc_cody74(-x*invsqrt2)", |x| 0.5 * erfc_cody(-x * INVSQRT2, 0x74)),
        ("0.5*erfc_cody0(-x*invsqrt2)", |x| 0.5 * erfc_cody(-x * INVSQRT2, 0)),
        ("0.5*nswc_wF(-x*invsqrt2)", |x| {
            let z = -x * INVSQRT2;
            let a = z.abs();
            let q = f::w_rn53(a) * f::nswc_derfc0(a);
            0.5 * if z >= 0.0 { q } else { 2.0 - q }
        }),
        ("0.5*cephes_wF(-x*invsqrt2)", |x| {
            let z = -x * INVSQRT2;
            let a = z.abs();
            let q = f::w_rn53(a) * f::cephes_f(a);
            0.5 * if z >= 0.0 { q } else { 2.0 - q }
        }),
        ("x87 0.5*erfc(-x*invsqrt2) libm z", |x| {
            let z = ext_to_f64(&ext_mul(&ef(x), &ef(-INVSQRT2), CW), CW);
            0.5 * libm::erfc(z)
        }),
        ("0.5*erfc(-x/sqrt2) RN53 z libm", |x| {
            let z = -x / SQRT2;
            0.5 * libm::erfc(z)
        }),
    ];
    for (name, ev) in graphs {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut max = 0u64;
        let mut sum = 0u128;
        for &(x, bits) in &rows {
            let g = ev(x);
            if !g.is_finite() {
                continue;
            }
            let d = ulp_distance(g, f64::from_bits(bits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
            } else {
                max = max.max(d);
                sum += d as u128;
            }
        }
        println!("{name:42} {ex}/{n} max={max} sum={sum}");
    }
}
