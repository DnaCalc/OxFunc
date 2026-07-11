//! W109 Phase-5: race historical GAMMALN kernels against the cached published
//! GAMMALN bits (93 rows). Candidates: Numerical Recipes gammln (Lanczos g=5,
//! 6 coefficients) in strict double, in full x87 extended, and in the
//! spill-loop staging; each with platform ln vs the x87 worksheet ln.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet, ulp_distance};
use oxfunc_core::excel_numeric::research as rx;

const COF: [f64; 6] = [
    76.18009172947146,
    -86.50532032941677,
    24.01409824083091,
    -1.231739572450155,
    0.1208650973866179e-2,
    -0.5395239384953e-5,
];

/// NR gammln, strict double, choice of ln.
fn nr_double(xx: f64, ln: fn(f64) -> f64) -> f64 {
    let x = xx;
    let mut y = x;
    let mut tmp = x + 5.5;
    tmp -= (x + 0.5) * ln(tmp);
    let mut ser = 1.000000000190015f64;
    for c in COF {
        y += 1.0;
        ser += c / y;
    }
    -tmp + ln(2.5066282746310005 * ser / x)
}

/// NR gammln entirely in x87 extended (PC=64), one final store. `FYL2X`-based
/// ln (ln2 * log2) as the CRT would produce inline.
fn nr_ext(xx: f64) -> f64 {
    let cw = rx::CW_PC64_RN;
    let ext = rx::ext_from_f64;
    let ln_ext = |v: &rx::Ext80| rx::ext_fyl2x(&rx::ext_ln2(), v, cw);
    let x = ext(xx);
    let mut y = x;
    let t0 = rx::ext_add(&x, &ext(5.5), cw);
    let xh = rx::ext_add(&x, &ext(0.5), cw);
    let tmp = rx::ext_sub(&t0, &rx::ext_mul(&xh, &ln_ext(&t0), cw), cw);
    let mut ser = ext(1.000000000190015);
    for c in COF {
        y = rx::ext_add(&y, &rx::ext_one(), cw);
        ser = rx::ext_add(&ser, &rx::ext_div(&ext(c), &y, cw), cw);
    }
    let num = rx::ext_mul(&ext(2.5066282746310005), &ser, cw);
    let q = rx::ext_div(&num, &x, cw);
    let r = rx::ext_sub(&ln_ext(&q), &tmp, cw);
    rx::ext_to_f64(&r, cw)
}

/// NR gammln with the spill-loop staging (every op RN53(RN64)), x87 ln.
fn nr_spill(xx: f64) -> f64 {
    let dr2 = |a: f64, b: f64, f: fn(&rx::Ext80, &rx::Ext80, u16) -> rx::Ext80| {
        rx::ext_to_f64(
            &f(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN),
            rx::CW_PC64_RN,
        )
    };
    let add = |a, b| dr2(a, b, rx::ext_add);
    let sub = |a, b| dr2(a, b, rx::ext_sub);
    let mul = |a, b| dr2(a, b, rx::ext_mul);
    let div = |a, b| dr2(a, b, rx::ext_div);
    let x = xx;
    let mut y = x;
    let t0 = add(x, 5.5);
    let tmp = sub(t0, mul(add(x, 0.5), rx::excel_ln(t0)));
    let mut ser = 1.000000000190015f64;
    for c in COF {
        y = add(y, 1.0);
        ser = add(ser, div(c, y));
    }
    sub(rx::excel_ln(div(mul(2.5066282746310005, ser), x)), tmp)
}

fn main() {
    let files: Vec<String> = std::env::args().skip(1).collect();
    let mut rows: Vec<(f64, f64)> = Vec::new(); // (x, excel gammaln)
    for file in &files {
        let ws: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(file).expect("read")).expect("parse");
        for w in &ws.witnesses {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            if let Some(v) = parse_bits_hex(&w.expected_bits) {
                rows.push((x, v));
            }
        }
    }
    println!("{} GAMMALN witnesses", rows.len());
    let cands: Vec<(&str, Box<dyn Fn(f64) -> f64>)> = vec![
        ("nr-double-platform-ln", Box::new(|x| nr_double(x, f64::ln))),
        ("nr-double-x87-ln", Box::new(|x| nr_double(x, rx::excel_ln))),
        ("nr-ext", Box::new(nr_ext)),
        ("nr-spill-x87-ln", Box::new(nr_spill)),
    ];
    for (name, f) in &cands {
        let (mut exact, mut max_ulp) = (0u32, 0u64);
        let mut worst = (0u64, 0.0f64);
        for &(x, want) in &rows {
            let v = f(x);
            if v.to_bits() == want.to_bits() {
                exact += 1;
            } else {
                let d = ulp_distance(v, want).unwrap_or(u64::MAX);
                if d > max_ulp {
                    max_ulp = d;
                    worst = (d, x);
                }
            }
        }
        println!("{name:24} {exact}/{} exact, max_ulp {max_ulp} (worst x={})", rows.len(), worst.1);
    }
}
