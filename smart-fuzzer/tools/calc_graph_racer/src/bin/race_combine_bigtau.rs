//! W109 G6-01: the |tau|>=1 fv/ty COMBINE wall. There em=exp(tau)-1 is EXACT
//! (no expm1 imprecision), so any PMT miss is a pure combine op-graph error.
//! Isolate & close it. Excel exp via x87 (Rust). Filter heldout/fvty to |tau|>=1.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC53_RN, CW_PC64_RN, ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64};

const RN53: u16 = CW_PC53_RN;
fn sm(a: f64, b: f64) -> f64 {
    ext_to_f64(
        &ext_mul(&ext_from_f64(a), &ext_from_f64(b), CW_PC64_RN),
        RN53,
    )
}
fn sd(a: f64, b: f64) -> f64 {
    ext_to_f64(
        &ext_div(&ext_from_f64(a), &ext_from_f64(b), CW_PC64_RN),
        RN53,
    )
}
fn sa(a: f64, b: f64) -> f64 {
    ext_to_f64(
        &ext_add(&ext_from_f64(a), &ext_from_f64(b), CW_PC64_RN),
        RN53,
    )
}

fn main() {
    let corpora = ["heldout", "fvty", "fv1sweep", "po2n"];
    for cn in corpora {
        let p = format!("../../work/w109/G6-solvers/answers-pmt-{}.json", cn);
        if !std::path::Path::new(&p).exists() {
            continue;
        }
        let ws: WitnessSet = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        // arrangements scored on the |tau|>=1 subset only
        let names = [
            "A1 landed",
            "A2 v=u",
            "A3 r-in-num",
            "A4 tf-recip",
            "A5 posP",
            "A6 spill-comb",
            "A7 num=fv*v+pv",
            "A8 tf-last-div",
        ];
        let mut sc = [0u32; 8];
        let mut tot = 0u32;
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
            let (r, n, pv, fv, ty) = (a[0], a[1], a[2], a[3], a[4]);
            let nl = -(n * rx::excel_log1p(r));
            if nl.abs() < 1.0 {
                continue;
            } // only |tau|>=1 where em is exact
            let u = rx::excel_exp(nl);
            let em = u - 1.0;
            if em == 0.0 {
                continue;
            }
            let v = 1.0 + em;
            let tf = 1.0 + r * ty;
            let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
            tot += 1;
            let cands = [
                ((pv + fv * v) / em / tf) * r,         // A1 landed
                ((pv + fv * u) / em / tf) * r,         // A2 v=u
                ((pv + fv * v) * r) / (em * tf),       // A3 r in num
                ((pv + fv * v) / em) * (1.0 / tf) * r, // A4 tf reciprocal
                {
                    let pp = 1.0 / v;
                    -(pv * pp + fv) * r / ((pp - 1.0) * tf)
                }, // A5 positive-power
                {
                    let vv = sa(1.0, em);
                    let tff = sa(1.0, sm(r, ty));
                    let num = sa(pv, sm(fv, vv));
                    sm(sd(sd(num, em), tff), r)
                }, // A6 spill combine
                ((fv * v + pv) / em / tf) * r,         // A7 order swap
                ((pv + fv * v) / em / tf * r),         // A8 (= A1 assoc)
            ];
            for i in 0..8 {
                if cands[i].to_bits() == want {
                    sc[i] += 1;
                }
            }
        }
        print!("{:9} |tau>=1| N={:5} :", cn, tot);
        for i in 0..8 {
            print!(
                "  {}={:.0}%",
                names[i],
                100.0 * sc[i] as f64 / tot.max(1) as f64
            );
        }
        println!();
    }
}
