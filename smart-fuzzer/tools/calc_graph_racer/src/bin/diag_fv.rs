//! W109 G6-01: diagnose the large-fv PMT assembly. fvsweep (r=.05,n=12,fv=1000)
//! fails 0/1024 under num=pv+fv*v. Print exact bits + test assembly variants.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

fn sordi(u: u64) -> i64 {
    if u < 1 << 63 {
        u as i64
    } else {
        -((u ^ (1u64 << 63)) as i64)
    }
}

fn main() {
    for cn in ["fvsweep", "fvty", "fv1sweep", "heldout"] {
        let p = format!("../../work/w109/G6-solvers/answers-pmt-{}.json", cn);
        if !std::path::Path::new(&p).exists() {
            continue;
        }
        let ws: WitnessSet = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        // assembly variants (all use tau=-n*log1p, em, v=1+em, P=exp(-tau)=(1+r)^n)
        let variants: [&str; 8] = [
            "disc num=pv+fv*v /em",               // 0 current
            "disc num=fv*v+pv /em",               // 1
            "disc (pv+fv*v)*r/(tf*em)",           // 2 product-order denom
            "fwd -(pv*P+fv)*r/(tf*(P-1))",        // 3 forward via P
            "fwd -(pv*P+fv)/(tf*q) q=(P-1)/r",    // 4 forward annuity q
            "disc num=pv+fv*v; q=num/em; *r/tf",  //5 (r before tf)
            "disc v=exp(tau) num=pv+fv*v",        //6
            "fwd P=exp(-tau) num=pv*P+fv em=P-1", //7
        ];
        let mut sc = [0u32; 8];
        let mut tot = 0u32;
        let mut off: std::collections::BTreeMap<i64, u32> = std::collections::BTreeMap::new();
        let mut sample = String::new();
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
            if r == 0.0 {
                continue;
            }
            let tau = -(n * rx::excel_log1p(r));
            let em = rx::excel_expm1_internal(tau);
            let v = 1.0 + em;
            let vexp = rx::excel_exp(tau);
            let bigp = rx::excel_exp(-tau); // (1+r)^n
            let tf = 1.0 + r * ty;
            let g = [
                {
                    let num = pv + fv * v;
                    (num / em) / tf * r
                },
                {
                    let num = fv * v + pv;
                    (num / em) / tf * r
                },
                {
                    let num = pv + fv * v;
                    (num * r) / (tf * em)
                },
                { -(pv * bigp + fv) * r / (tf * (bigp - 1.0)) },
                {
                    let q = (bigp - 1.0) / r;
                    -(pv * bigp + fv) / (tf * q)
                },
                {
                    let num = pv + fv * v;
                    ((num / em) * r) / tf
                },
                {
                    let num = pv + fv * vexp;
                    (num / em) / tf * r
                },
                {
                    let num = pv * bigp + fv;
                    (num / (bigp - 1.0)) / tf * r * -1.0
                },
            ];
            tot += 1;
            for i in 0..8 {
                if g[i].to_bits() == want {
                    sc[i] += 1;
                }
            }
            let d = (sordi(g[0].to_bits()) - sordi(want)).clamp(-12, 12);
            *off.entry(d).or_default() += 1;
            if sample.is_empty() {
                sample = format!(
                    "  sample r={} n={} pv={} fv={} ty={}: want={:016x} v0={:016x} v3(fwd)={:016x}",
                    r,
                    n,
                    pv,
                    fv,
                    ty,
                    want,
                    g[0].to_bits(),
                    g[3].to_bits()
                );
            }
        }
        println!("=== {} (N={}) ===", cn, tot);
        for i in 0..8 {
            println!("  [{}] {:<34} {}/{}", i, variants[i], sc[i], tot);
        }
        print!("  off(v0):");
        for (d, c) in &off {
            print!(" {:+}:{}", d, c);
        }
        println!();
        println!("{}", sample);
    }
}
