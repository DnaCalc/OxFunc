//! Localize the std/UCRT heldout "+30" advantage: split the 875 heldout rows by
//! |tau|<1 (expm1-wall confounded) vs |tau|>=1 (clean). Expect: std wins only in the
//! confounded region; CR>=std in the clean region -> the +30 is an expm1-wall artifact.
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
    let (mut lc, mut ls) = (0u32, 0u32);
    let mut ln = 0u32; // |tau|<1
    let (mut bc, mut bs) = (0u32, 0u32);
    let mut bn = 0u32; // |tau|>=1
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
        let (r, n, pv, fv, ty) = (a[0], a[1], a[2], a[3], a[4]);
        let small = r != 0.0 && (n * rx::excel_log1p(r)).abs() < 1.0;
        let cok = pmt_full(r, n, pv, fv, ty, l1p_cr).to_bits() == want;
        let sok = pmt_full(r, n, pv, fv, ty, l1p_std).to_bits() == want;
        if small {
            ln += 1;
            if cok {
                lc += 1;
            }
            if sok {
                ls += 1;
            }
        } else {
            bn += 1;
            if cok {
                bc += 1;
            }
            if sok {
                bs += 1;
            }
        }
    }
    println!(
        "heldout |tau|<1  ({} rows, CONFOUNDED): CR {}/{}  std {}/{}   std-CR = {:+}",
        ln,
        lc,
        ln,
        ls,
        ln,
        ls as i32 - lc as i32
    );
    println!(
        "heldout |tau|>=1 ({} rows, CLEAN):      CR {}/{}  std {}/{}   std-CR = {:+}",
        bn,
        bc,
        bn,
        bs,
        bn,
        bs as i32 - bc as i32
    );
    println!(
        "TOTAL: CR {}/{}  std {}/{}",
        lc + bc,
        ln + bn,
        ls + bs,
        ln + bn
    );
}
