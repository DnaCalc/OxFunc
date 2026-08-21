//! Verify the workflow's clincher: on heldout, is std's edge over CR confined to |tau|<1
//! (expm1-wall confounded) and does CR win/tie at |tau|>=1 (clean)?
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
fn l1p_cr(r: f64) -> f64 {
    rx::excel_log1p(r)
}
fn l1p_std(r: f64) -> f64 {
    r.ln_1p()
}
fn pmt_full(r: f64, n: f64, pv: f64, fv: f64, ty: f64, l1p: fn(f64) -> f64) -> f64 {
    if r == 0.0 {
        return -(pv + fv) / n;
    }
    let tau = -(n * l1p(r));
    let em = rx::excel_expm1_internal(tau);
    let v = 1.0 + em;
    let tf = 1.0 + r * ty;
    let num = pv + fv * v;
    ((num / em) / tf) * r
}
fn main() {
    let ws: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string("../../work/w109/G6-solvers/answers-pmt-heldout.json").unwrap(),
    )
    .unwrap();
    let (mut cr_lo, mut cr_hi, mut std_lo, mut std_hi, mut n_lo, mut n_hi) = (0, 0, 0, 0, 0, 0);
    for w in &ws.witnesses {
        let a: Vec<f64> = w
            .args
            .iter()
            .filter_map(|x| match x {
                WitnessArg::Scalar(s) => parse_bits_hex(s),
                _ => None,
            })
            .collect();
        if a.len() != 5 {
            continue;
        }
        let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
        let (r, n) = (a[0], a[1]);
        let tau = -(n * rx::excel_log1p(r)).abs();
        let hi = tau.abs() >= 1.0;
        let cok = pmt_full(a[0], a[1], a[2], a[3], a[4], l1p_cr).to_bits() == want;
        let sok = pmt_full(a[0], a[1], a[2], a[3], a[4], l1p_std).to_bits() == want;
        if hi {
            n_hi += 1;
            if cok {
                cr_hi += 1;
            }
            if sok {
                std_hi += 1;
            }
        } else {
            n_lo += 1;
            if cok {
                cr_lo += 1;
            }
            if sok {
                std_lo += 1;
            }
        }
    }
    println!("heldout split by |tau|:");
    println!(
        "  |tau|<1  (n={}): CR {}  std {}  (std-CR={})",
        n_lo,
        cr_lo,
        std_lo,
        std_lo as i64 - cr_lo as i64
    );
    println!(
        "  |tau|>=1 (n={}): CR {}  std {}  (std-CR={})",
        n_hi,
        cr_hi,
        std_hi,
        std_hi as i64 - cr_hi as i64
    );
    println!(
        "=> if std-CR is +large in |tau|<1 and <=0 in |tau|>=1, log1p is CR (std's edge is expm1-wall compensation)"
    );
}
