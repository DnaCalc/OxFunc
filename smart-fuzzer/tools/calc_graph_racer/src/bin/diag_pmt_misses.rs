//! W109 G6-01: classify PMT misses by CAUSE to point effort at real ROI.
//! landed model = quotient-first expm1 form. For each corpus, bucket misses by
//! |tau| band, fv==0?, ty==0?, integer n?, and ULP error magnitude.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

fn v_landed(r: f64, n: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let nl = -(n * rx::excel_log1p(r));
    let em = rx::excel_expm1_internal(nl);
    if em == 0.0 {
        return f64::NAN;
    }
    let v = 1.0 + em;
    let tf = 1.0 + r * ty;
    let num = pv + fv * v;
    ((num / em) / tf) * r
}

fn main() {
    let corpora = [
        "po2",
        "po2n",
        "genrate",
        "heldout",
        "combsweep",
        "fvty",
        "fv1sweep",
    ];
    for cn in corpora {
        let p = format!("../../work/w109/G6-solvers/answers-pmt-{}.json", cn);
        if !std::path::Path::new(&p).exists() {
            continue;
        }
        let ws: WitnessSet = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let mut tot = 0u32;
        let mut hit = 0u32;
        // miss buckets
        let (mut m_smalltau, mut m_bigtau) = (0u32, 0u32);
        let (mut m_fv0ty0, mut m_fvnz, mut m_tynz) = (0u32, 0u32, 0u32);
        let mut ulp_small = 0u32; // |err|<=2 ULP
        let mut ulp_big = 0u32;
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
            let want = parse_bits_hex(&w.expected_bits).unwrap();
            tot += 1;
            let got = v_landed(r, n, pv, fv, ty);
            if got.to_bits() == want.to_bits() {
                hit += 1;
                continue;
            }
            let tau = (n * rx::excel_log1p(r)).abs();
            if tau < 1.0 {
                m_smalltau += 1;
            } else {
                m_bigtau += 1;
            }
            if fv == 0.0 && ty == 0.0 {
                m_fv0ty0 += 1;
            }
            if fv != 0.0 {
                m_fvnz += 1;
            }
            if ty != 0.0 {
                m_tynz += 1;
            }
            let d = (got.to_bits() as i64 - want.to_bits() as i64).abs();
            if d <= 2 {
                ulp_small += 1;
            } else {
                ulp_big += 1;
            }
        }
        let miss = tot - hit;
        println!(
            "{:10} N={:6} hit={:5} ({:4.1}%) miss={:5} | tau<1:{:4} tau>=1:{:4} | fv0ty0:{:4} fv!=0:{:4} ty!=0:{:4} | <=2ulp:{:4} >2ulp:{:4}",
            cn,
            tot,
            hit,
            100.0 * hit as f64 / tot as f64,
            miss,
            m_smalltau,
            m_bigtau,
            m_fv0ty0,
            m_fvnz,
            m_tynz,
            ulp_small,
            ulp_big
        );
    }
}
