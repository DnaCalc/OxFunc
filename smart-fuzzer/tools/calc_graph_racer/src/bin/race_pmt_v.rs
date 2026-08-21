//! W109 G6-01: does PMT's discount factor use v=1+em or v=exp(tau)? For large |fv|
//! (num=pv+fv*v), a 1-ulp v error is amplified ~fv-fold -> the broad fvty misses.
//! Test both v deliveries across all corpora. Combine = quotient-first H-DF.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

fn pmt(r: f64, n: f64, pv: f64, fv: f64, ty: f64, v_is_exp: bool) -> f64 {
    if r == 0.0 {
        return -(pv + fv) / n;
    }
    let tau = -(n * rx::excel_log1p(r));
    let em = rx::excel_expm1_internal(tau);
    let v = if v_is_exp {
        rx::excel_exp(tau)
    } else {
        1.0 + em
    };
    let tf = 1.0 + r * ty;
    let num = pv + fv * v;
    let q1 = num / em;
    let q2 = q1 / tf;
    q2 * r
}
fn main() {
    let corpora = [
        "heldout",
        "combsweep",
        "po2",
        "r25",
        "pvladder",
        "fvty",
        "fvsweep",
        "fv1sweep",
    ];
    for cn in corpora {
        let p = format!("../../work/w109/G6-solvers/answers-pmt-{}.json", cn);
        if !std::path::Path::new(&p).exists() {
            continue;
        }
        let ws: WitnessSet = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let mut ok1 = 0u32;
        let mut okE = 0u32;
        let mut tot = 0u32;
        // fv!=0 subset
        let mut ok1_fv = 0u32;
        let mut okE_fv = 0u32;
        let mut tot_fv = 0u32;
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
            let h1 = pmt(a[0], a[1], a[2], a[3], a[4], false).to_bits() == want;
            let hE = pmt(a[0], a[1], a[2], a[3], a[4], true).to_bits() == want;
            tot += 1;
            if h1 {
                ok1 += 1;
            }
            if hE {
                okE += 1;
            }
            if a[3] != 0.0 {
                tot_fv += 1;
                if h1 {
                    ok1_fv += 1;
                }
                if hE {
                    okE_fv += 1;
                }
            }
        }
        println!(
            "{:10} N={:6}  v=1+em {:>5}/{} ({:.0}%)   v=exp {:>5}/{} ({:.0}%)   [fv!=0: 1+em {}/{}  exp {}/{}]",
            cn,
            tot,
            ok1,
            tot,
            100.0 * ok1 as f64 / tot as f64,
            okE,
            tot,
            100.0 * okE as f64 / tot as f64,
            ok1_fv,
            tot_fv,
            okE_fv,
            tot_fv
        );
    }
}
