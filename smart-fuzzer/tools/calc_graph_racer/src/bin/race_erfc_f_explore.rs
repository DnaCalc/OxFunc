//! Local implied-F explorer: residual law, truncated CF, piecewise cuts,
//! t-maps, CDFLIB erfc1. Heldouts unnamed. Does not land kernels.
//!
//!   cargo run --release --bin race_erfc_f_explore -- ../../work/w109/G3-01-dist [out-dir]

use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("G3-01-dist (heldout files are not named)");
    assert!(!dir.contains("heldout"));
    let out = PathBuf::from(
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| "../../work/w109/erfc-f-explore".into()),
    );
    fs::create_dir_all(&out).unwrap();
    let rows = f::load_q_rows(&dir);
    let mid: Vec<_> = rows
        .iter()
        .copied()
        .filter(|(z, _)| *z >= 0.5 && *z < 4.0)
        .collect();
    let tail: Vec<_> = rows.iter().copied().filter(|(z, _)| *z >= 4.0).collect();
    println!(
        "rows={} mid={} tail={}  F_or uses w_rn53; heldout absent",
        rows.len(),
        mid.len(),
        tail.len()
    );

    println!("\n## named F (w_rn53 F_or)");
    for (name, eval) in [
        ("nswc_derfc0", f::nswc_derfc0 as fn(f64) -> f64),
        ("nswc_pqr", f::nswc_pqr_f),
        ("nswc_ccdd", f::nswc_ccdd_f),
        ("cody_erfcx", f::cody_erfcx_f),
        ("cephes_erfce", f::cephes_f),
        ("cdflib_erfc1", f::cdflib_erfc1_f),
        ("cf_as714_n80", f::cf_as714_f),
        ("cf_gautschi_n80", f::cf_gautschi_f),
    ] {
        let (m, t) = f::score_f(&rows, eval);
        println!("  {name:<18} mid {}  tail {}", f::fmt_acc(&m), f::fmt_acc(&t));
    }

    // Residual law vs NSWC DERFC0 on mid.
    let mut hist = [0usize; 10];
    let mut n_pos = 0usize;
    let mut n_neg = 0usize;
    let mut n_zero = 0usize;
    let mut switches = 0usize;
    let mut last_sign = 0i8;
    let res_path = out.join("residual-nswc-mid.tsv");
    let mut rf = fs::File::create(&res_path).unwrap();
    writeln!(rf, "z\tulp\tsign\tf_or\tf_nswc").unwrap();
    for &(z, qbits) in &mid {
        let Some(fo) = f::f_or(z, qbits) else {
            continue;
        };
        let fg = f::nswc_derfc0(z);
        let d = ulp_distance(fg, fo).unwrap_or(u64::MAX);
        if d > (1 << 20) {
            continue;
        }
        let sign = if d == 0 {
            0
        } else if fg > fo {
            1
        } else {
            -1
        };
        match sign {
            0 => n_zero += 1,
            1 => n_pos += 1,
            _ => n_neg += 1,
        }
        if last_sign != 0 && sign != 0 && sign != last_sign {
            switches += 1;
        }
        if sign != 0 {
            last_sign = sign;
        }
        let bucket = match d {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 4,
            5 => 5,
            6 => 6,
            7 => 7,
            8..=15 => 8,
            _ => 9,
        };
        hist[bucket] += 1;
        writeln!(
            rf,
            "{z:.16e}\t{d}\t{sign}\t{}\t{}",
            fo.to_bits(),
            fg.to_bits()
        )
        .unwrap();
    }
    println!("\n## NSWC residual law (mid)");
    println!(
        "  sign +={} -={} 0={}  sign-switches={}  dumped {}",
        n_pos,
        n_neg,
        n_zero,
        switches,
        res_path.display()
    );
    println!(
        "  ulp hist 0..7,8-15,16+: {:?}",
        hist
    );

    println!("\n## truncated CF n (w_rn53 F_or)");
    println!(
        "  {:>4} {:>22} {:>22} {:>22} {:>22}",
        "n", "as714 mid", "as714 tail", "gautschi mid", "gautschi tail"
    );
    let mut cf_lines = String::from("n\tas714_mid\tas714_tail\tgaut_mid\tgaut_tail\n");
    for n in 1u32..=40 {
        let (am, at) = f::score_f(&rows, |z| f::cf_as714_n(z, n));
        let (gm, gt) = f::score_f(&rows, |z| f::cf_gautschi_n(z, n));
        println!(
            "  {n:>4} {:>22} {:>22} {:>22} {:>22}",
            f::fmt_acc(&am),
            f::fmt_acc(&at),
            f::fmt_acc(&gm),
            f::fmt_acc(&gt)
        );
        cf_lines.push_str(&format!(
            "{n}\t{}/{}\t{}/{}\t{}/{}\t{}/{}\n",
            am.exact, am.n, at.exact, at.n, gm.exact, gm.n, gt.exact, gt.n
        ));
    }
    fs::write(out.join("cf-n-table.tsv"), cf_lines).unwrap();

    println!("\n## piecewise NSWC-below-cut / CF-as714-n80-above");
    let mut best_cut = 0.0;
    let mut best_all = 0usize;
    for k in 0..=80 {
        let cut = 0.5 + k as f64 * 0.1;
        if cut > 8.5 {
            break;
        }
        let (m, t) = f::score_f(&rows, |z| {
            if z < cut {
                f::nswc_derfc0(z)
            } else {
                f::cf_as714_f(z)
            }
        });
        let all = m.exact + t.exact;
        if all > best_all {
            best_all = all;
            best_cut = cut;
        }
        if k % 5 == 0 || all > 2389 + 1283 {
            println!(
                "  cut={cut:.1} mid {} tail {} all_exact={}",
                f::fmt_acc(&m),
                f::fmt_acc(&t),
                all
            );
        }
    }
    println!("  best_cut={best_cut:.1} all_exact={best_all}");

    println!("\n## t-map on frozen NSWC PQR (F = pqr(x) with substituted t)");
    let maps: [(&str, fn(f64) -> f64); 8] = [
        ("t=(x-3.75)/(x+3.75)", |x| (x - 3.75) / (x + 3.75)),
        ("t=(x-2)/(x+2)", |x| (x - 2.0) / (x + 2.0)),
        ("t=(x-1)/(x+1)", |x| (x - 1.0) / (x + 1.0)),
        ("t=(x-4)/(x+4)", |x| (x - 4.0) / (x + 4.0)),
        ("t=1/x", |x| 1.0 / x),
        ("t=1/x^2", |x| 1.0 / (x * x)),
        ("t=1/(2.5+x^2)", |x| 1.0 / (2.5 + x * x)),
        ("t=x", |x| x),
    ];
    for (name, tm) in maps {
        let (m, t) = f::score_f(&rows, |z| f::nswc_pqr_t(z, tm(z)));
        println!("  {name:<24} mid {} tail {}", f::fmt_acc(&m), f::fmt_acc(&t));
    }

    println!("\n## pins F ulp vs F_or(w_rn53)");
    for &z in &f::PIN_Z {
        let Some((_, qbits)) = rows.iter().find(|(zz, _)| *zz == z) else {
            continue;
        };
        let Some(fo) = f::f_or(z, *qbits) else {
            continue;
        };
        print!("  z={z}:");
        for (name, eval) in [
            ("nswc", f::nswc_derfc0 as fn(f64) -> f64),
            ("cdflib", f::cdflib_erfc1_f),
            ("cephes", f::cephes_f),
            ("cody", f::cody_erfcx_f),
            ("cf80", f::cf_as714_f),
        ] {
            print!(
                " {name}={}",
                ulp_distance(eval(z), fo).unwrap_or(u64::MAX)
            );
        }
        println!();
    }
    println!("\nartifacts in {}", out.display());
}
