//! Refine the confound-free |tau|>=1 log1p test: stratify by |tau| threshold, measure
//! miss ULP magnitude, and check CR vs FYL2XP1 vs fdlibm agreement row-by-row.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN as CW, ext_add, ext_from_f64 as ef, ext_fyl2x, ext_fyl2xp1, ext_ln2, ext_one,
    ext_to_f64,
};

fn l1p_cr(r: f64) -> f64 {
    rx::excel_log1p(r)
}
fn l1p_std(r: f64) -> f64 {
    r.ln_1p()
}
fn l1p_fyl(r: f64) -> f64 {
    if r.abs() < 0.292893218813452 {
        ext_to_f64(&ext_fyl2xp1(&ext_ln2(), &ef(r), CW), CW)
    } else {
        ext_to_f64(
            &ext_fyl2x(&ext_ln2(), &ext_add(&ext_one(), &ef(r), CW), CW),
            CW,
        )
    }
}
fn onepr_exact(r: f64) -> bool {
    (1.0 + r) - 1.0 == r
}
fn pmt0(r: f64, n: f64, pv: f64, l1p: fn(f64) -> f64) -> f64 {
    let tau = -(n * l1p(r));
    let em = rx::excel_expm1_internal(tau);
    ((pv / em) / 1.0) * r
}
fn ulp_diff(a: u64, b: u64) -> i64 {
    a as i64 - b as i64
}

fn main() {
    let files = [
        "answers-pmt-em.json",
        "answers-pmt-heldout.json",
        "answers-pmt-log1p.json",
        "answers-pmt-denselog1p.json",
    ];
    let mut rows: Vec<(f64, f64, f64, u64)> = Vec::new();
    for fname in files {
        let p = format!("../../work/w109/G6-solvers/{}", fname);
        let ws: WitnessSet = match std::fs::read_to_string(&p) {
            Ok(s) => serde_json::from_str(&s).unwrap(),
            Err(_) => continue,
        };
        for w in &ws.witnesses {
            let a: Vec<f64> = w
                .args
                .iter()
                .filter_map(|x| match x {
                    WitnessArg::Scalar(s) => parse_bits_hex(s),
                    _ => None,
                })
                .collect();
            if a.len() != 5 || a[3] != 0.0 || a[4] != 0.0 || a[0] == 0.0 {
                continue;
            }
            if onepr_exact(a[0]) {
                continue;
            }
            let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
            rows.push((a[0], a[1], a[2], want));
        }
    }
    // VALIDATION: po2 (exact-1+r) |tau|>=1 - forward model must be ~100% if em/combine correct
    {
        let mut po2: Vec<(f64, f64, f64, u64)> = Vec::new();
        for fname in ["answers-pmt-po2n.json", "answers-pmt-genrate.json"] {
            let p = format!("../../work/w109/G6-solvers/{}", fname);
            let ws: WitnessSet = match std::fs::read_to_string(&p) {
                Ok(s) => serde_json::from_str(&s).unwrap(),
                Err(_) => continue,
            };
            for w in &ws.witnesses {
                let a: Vec<f64> = w
                    .args
                    .iter()
                    .filter_map(|x| match x {
                        WitnessArg::Scalar(s) => parse_bits_hex(s),
                        _ => None,
                    })
                    .collect();
                if a.len() != 5 || a[3] != 0.0 || a[4] != 0.0 || a[0] == 0.0 {
                    continue;
                }
                if !onepr_exact(a[0]) {
                    continue;
                }
                if (a[1] * rx::excel_log1p(a[0])).abs() < 1.0 {
                    continue;
                }
                po2.push((
                    a[0],
                    a[1],
                    a[2],
                    parse_bits_hex(&w.expected_bits).unwrap().to_bits(),
                ));
            }
        }
        let ok = po2
            .iter()
            .filter(|(r, n, pv, want)| pmt0(*r, *n, *pv, l1p_cr).to_bits() == *want)
            .count();
        println!(
            "VALIDATION po2 (exact-1+r) |tau|>=1: CR forward-pmt {}/{} ({:.1}%)\n",
            ok,
            po2.len(),
            100.0 * ok as f64 / po2.len().max(1) as f64
        );
    }
    rows.sort_by(|x, y| {
        (x.0.to_bits(), x.1.to_bits(), x.2.to_bits(), x.3).cmp(&(
            y.0.to_bits(),
            y.1.to_bits(),
            y.2.to_bits(),
            y.3,
        ))
    });
    rows.dedup();

    for thr in [1.0f64, 1.3, 2.0, 3.0, 5.0] {
        let sel: Vec<_> = rows
            .iter()
            .filter(|(r, n, _, _)| (n * rx::excel_log1p(*r)).abs() >= thr)
            .collect();
        if sel.is_empty() {
            continue;
        }
        let mut ok_cr = 0;
        let mut ok_fyl = 0;
        let mut ok_std = 0;
        let mut miss_ulp = std::collections::BTreeMap::<i64, u32>::new();
        let mut cr_ne_fyl = 0;
        for (r, n, pv, want) in &sel {
            let c = pmt0(*r, *n, *pv, l1p_cr);
            let fy = pmt0(*r, *n, *pv, l1p_fyl);
            let st = pmt0(*r, *n, *pv, l1p_std);
            if c.to_bits() == *want {
                ok_cr += 1;
            } else {
                *miss_ulp.entry(ulp_diff(c.to_bits(), *want)).or_default() += 1;
            }
            if fy.to_bits() == *want {
                ok_fyl += 1;
            }
            if st.to_bits() == *want {
                ok_std += 1;
            }
            if c.to_bits() != fy.to_bits() {
                cr_ne_fyl += 1;
            }
        }
        let tot = sel.len();
        println!(
            "|tau|>={:<4} n={:6}  CR:{:.1}% FYL:{:.1}% std:{:.1}%  CR!=FYL:{}",
            thr,
            tot,
            100.0 * ok_cr as f64 / tot as f64,
            100.0 * ok_fyl as f64 / tot as f64,
            100.0 * ok_std as f64 / tot as f64,
            cr_ne_fyl
        );
        if thr == 2.0 {
            print!("   CR miss ULP-signed histogram: ");
            for (u, c) in &miss_ulp {
                print!("{}:{} ", u, c);
            }
            println!();
        }
    }
}
