//! W109 G6-01 (Fable audit#6): is Excel's PMT BODY an x87 spill-loop (each
//! combine op = fl53(fl64(.)), like the proven XNPV body)? If so, the "em wall"
//! may be partly a COMBINE-precision artifact, not an expm1 defect. Test the
//! combine discipline {SSE2, x87-spill} x em {SSE2-Kahan(163), spill-Kahan(165)}
//! against the real PMT oracles.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC53_RN, CW_PC64_RN, ext_div, ext_from_f64, ext_mul, ext_to_f64};

const RN53: u16 = CW_PC53_RN;
fn s_mul(a: f64, b: f64) -> f64 {
    ext_to_f64(
        &ext_mul(&ext_from_f64(a), &ext_from_f64(b), CW_PC64_RN),
        RN53,
    )
}
fn s_div(a: f64, b: f64) -> f64 {
    ext_to_f64(
        &ext_div(&ext_from_f64(a), &ext_from_f64(b), CW_PC64_RN),
        RN53,
    )
}
fn s_add(a: f64, b: f64) -> f64 {
    ext_to_f64(
        &rx::ext_add(&ext_from_f64(a), &ext_from_f64(b), CW_PC64_RN),
        RN53,
    )
}

// em variants
fn em_sse2(nl: f64) -> f64 {
    rx::excel_expm1_internal(nl)
}
fn em_spill(nl: f64) -> f64 {
    // spill-loop Kahan: u=exp(nl); if u==1 nl; if |nl|<1 fl53(fl64(fl53(fl64((u-1)*nl))/ln u)); else u-1
    let u = rx::excel_exp(nl);
    if u == 1.0 {
        return nl;
    }
    if nl.abs() < 1.0 {
        let y = u - 1.0;
        let num = s_mul(y, nl); // fl53(fl64(y*nl))
        s_div(num, rx::excel_ln(u)) // fl53(fl64(num/lnu))
    } else {
        u - 1.0
    }
}

// combine SSE2: ((pv+fv*v)/em/tf)*r
fn comb_sse2(r: f64, pv: f64, fv: f64, ty: f64, em: f64) -> f64 {
    let v = 1.0 + em;
    let tf = 1.0 + r * ty;
    let num = pv + fv * v;
    ((num / em) / tf) * r
}
// combine x87-spill: each op double-rounded
fn comb_spill(r: f64, pv: f64, fv: f64, ty: f64, em: f64) -> f64 {
    let v = s_add(1.0, em);
    let tf = s_add(1.0, s_mul(r, ty));
    let num = s_add(pv, s_mul(fv, v));
    s_mul(s_div(s_div(num, em), tf), r)
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
    // (label, em_fn, combine_fn)
    type E = fn(f64) -> f64;
    type C = fn(f64, f64, f64, f64, f64) -> f64;
    let variants: [(&str, E, C); 4] = [
        ("emSSE2/combSSE2 [landed]", em_sse2, comb_sse2),
        ("emSSE2/combSPILL", em_sse2, comb_spill),
        ("emSPILL/combSSE2", em_spill, comb_sse2),
        ("emSPILL/combSPILL", em_spill, comb_spill),
    ];
    print!("{:11}{:>7}", "corpus", "N");
    for (nm, _, _) in &variants {
        print!(" {:>22}", nm);
    }
    println!();
    for cn in corpora {
        let p = format!("../../work/w109/G6-solvers/answers-pmt-{}.json", cn);
        if !std::path::Path::new(&p).exists() {
            continue;
        }
        let ws: WitnessSet = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let mut sc = [0u32; 4];
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
            let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
            tot += 1;
            let nl = -(n * rx::excel_log1p(r));
            for (i, (_, ef, cf)) in variants.iter().enumerate() {
                let em = ef(nl);
                if em == 0.0 {
                    continue;
                }
                if cf(r, pv, fv, ty, em).to_bits() == want {
                    sc[i] += 1;
                }
            }
        }
        print!("{:11}{:>7}", cn, tot);
        for i in 0..4 {
            print!(" {:>21.1}%", 100.0 * sc[i] as f64 / tot as f64);
        }
        println!();
    }
}
