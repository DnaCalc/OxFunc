//! W109 step-1: test the UNIFIED atanh form 0.5*log1p(2x/(1-x)) via x87 fyl2xp1
//! against the transition band [9.5e-5, 1.02e-4] where neither the pair nor the
//! plain ratio matches. Hypothesis: Excel uses one log1p(2x/(1-x)) form for
//! |x| below the fyl2x switch, and the pair/ratio are only close approximations
//! that diverge from it exactly in the band. All scoring is offline.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

const CW: u16 = 0x133F;
fn e(x: f64) -> rx::Ext80 { rx::ext_from_f64(x) }
fn tof(v: &rx::Ext80) -> f64 { rx::ext_to_f64(v, CW) }

// --- reference forms (from check_atanh_full) ---
fn atanh_pair_x87(x: f64) -> f64 {
    let ln2 = rx::ext_ln2();
    let t1 = rx::ext_fyl2xp1(&ln2, &e(x), CW);
    let t2 = rx::ext_fyl2xp1(&ln2, &e(-x), CW);
    let d = rx::ext_sub(&t1, &t2, CW);
    tof(&rx::ext_mul(&d, &e(0.5), CW))
}
fn atanh_ratio_x87(x: f64) -> f64 {
    let r = (1.0 + x) / (1.0 - x);
    let l = rx::ext_fyl2x(&rx::ext_ln2(), &e(r), CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}
// Region-B pair via the PRODUCTION SSE2 double-double log1p (rx::excel_log1p),
// to see whether region B can be ported without any new x87 exposure.
fn atanh_pair_sse(x: f64) -> f64 {
    0.5 * (rx::excel_log1p(x) - rx::excel_log1p(-x))
}

// --- unified log1p(2x/(1-x)) forms ---
// arg formed in binary64, various groupings; log1p via x87 fyl2xp1; half staged 2 ways.
fn uni_a(x: f64) -> f64 { // 2*x/(1-x), half in extended
    let r = 2.0 * x / (1.0 - x);
    let l = rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}
fn uni_b(x: f64) -> f64 { // 2*(x/(1-x)), half in extended
    let r = 2.0 * (x / (1.0 - x));
    let l = rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}
fn uni_c(x: f64) -> f64 { // (2*x)/(1-x), half in extended
    let r = (2.0 * x) / (1.0 - x);
    let l = rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}
fn uni_a_store(x: f64) -> f64 { // 2*x/(1-x), log1p stored to f64 then *0.5
    let r = 2.0 * x / (1.0 - x);
    let l = tof(&rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW));
    0.5 * l
}
fn uni_ext(x: f64) -> f64 { // arg 2x/(1-x) fully in x87 extended, half extended
    let ex = e(x);
    let one = rx::ext_one();
    let num = rx::ext_mul(&e(2.0), &ex, CW);
    let den = rx::ext_sub(&one, &ex, CW);
    let r = rx::ext_div(&num, &den, CW);
    let l = rx::ext_fyl2xp1(&rx::ext_ln2(), &r, CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}
fn uni_fdlibm(x: f64) -> f64 { // fdlibm small-x argument: 2x + 2x*x/(1-x), binary64
    let r = 2.0 * x + 2.0 * x * x / (1.0 - x);
    let l = rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}

// fdlibm-argument staging variants (all log1p via x87 fyl2xp1, half in extended).
fn fd_lit(x: f64) -> f64 { // fdlibm literal: t=x+x; t + t*x/(1-x)
    let t = x + x;
    let r = t + t * x / (1.0 - x);
    let l = rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}
fn fd_lit_grp(x: f64) -> f64 { // t=x+x; t + t*(x/(1-x))
    let t = x + x;
    let r = t + t * (x / (1.0 - x));
    let l = rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}
fn fd_grp2(x: f64) -> f64 { // 2x + (2x*x)/(1-x)
    let r = 2.0 * x + (2.0 * x * x) / (1.0 - x);
    let l = rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}
fn fd_corr_ext(x: f64) -> f64 { // 2x binary64 + correction 2x^2/(1-x) formed in x87, arg stored
    let two_x = e(2.0 * x);
    let xx = rx::ext_mul(&e(x), &e(x), CW);
    let corr = rx::ext_div(&rx::ext_mul(&e(2.0), &xx, CW), &rx::ext_sub(&rx::ext_one(), &e(x), CW), CW);
    let arg = rx::ext_add(&two_x, &corr, CW);
    let l = rx::ext_fyl2xp1(&rx::ext_ln2(), &arg, CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}
fn fd_full_ext(x: f64) -> f64 { // entire fdlibm arg in x87: t=2x; t + t*x/(1-x)
    let ex = e(x);
    let t = rx::ext_mul(&e(2.0), &ex, CW);
    let corr = rx::ext_div(&rx::ext_mul(&t, &ex, CW), &rx::ext_sub(&rx::ext_one(), &ex, CW), CW);
    let arg = rx::ext_add(&t, &corr, CW);
    let l = rx::ext_fyl2xp1(&rx::ext_ln2(), &arg, CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}

fn load(paths: &[&str]) -> Vec<(f64, u64)> {
    let mut m: std::collections::BTreeMap<u64, u64> = Default::default();
    for p in paths {
        let ws: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(p).expect("read")).expect("parse");
        for w in &ws.witnesses {
            if !w.expected_bits.starts_with("0x") { continue; }
            let x = match &w.args[0] { WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(), _ => continue };
            if let Some(v) = parse_bits_hex(&w.expected_bits) {
                m.insert(x.to_bits(), v.to_bits());
            }
        }
    }
    m.into_iter().map(|(k, v)| (f64::from_bits(k), v)).collect()
}

fn score_region(name: &str, rows: &[(f64, u64)], lo: f64, hi: f64, f: &dyn Fn(f64) -> f64) {
    let mut ok = 0u32;
    let mut n = 0u32;
    for (x, want) in rows {
        let a = x.abs();
        if a < lo || a >= hi { continue; }
        n += 1;
        if f(*x).to_bits() == *want { ok += 1; }
    }
    println!("  {name:28} [{lo:.1e},{hi:.1e}): {ok}/{n}");
}

fn main() {
    let rows = load(&[
        "../../work/w109/G4-hyp-answers-atanh.json",
        "../../work/w109/G4-02-answers-atanh-band.json",
        "../../work/w109/G4-02-answers-atanh-gap.json",
        "../../work/w109/G4-02-answers-atanh-switch.json",
    ]);
    println!("{} distinct ATANH rows\n", rows.len());

    // The three band |x| values (both signs) that miss under pair|ratio piecewise.
    let band: Vec<(f64, u64)> = rows
        .iter()
        .cloned()
        .filter(|(x, _)| { let a = x.abs(); a >= 9.5e-5 && a <= 1.02e-4 })
        .collect();
    println!("Band rows (|x| in [9.5e-5, 1.02e-4]): {}", band.len());

    let forms: [(&str, fn(f64) -> f64); 11] = [
        ("uni_a 2x/(1-x)", uni_a),
        ("uni_b 2(x/(1-x))", uni_b),
        ("uni_c (2x)/(1-x)", uni_c),
        ("uni_a_store", uni_a_store),
        ("uni_ext (all x87)", uni_ext),
        ("uni_fdlibm 2x+2x^2/(1-x)", uni_fdlibm),
        ("fd_lit t=x+x;t+t*x/(1-x)", fd_lit),
        ("fd_lit_grp t+t*(x/(1-x))", fd_lit_grp),
        ("fd_grp2 2x+(2x*x)/(1-x)", fd_grp2),
        ("fd_corr_ext", fd_corr_ext),
        ("fd_full_ext", fd_full_ext),
    ];

    // Per-form: region-B (must stay 175/175) + band hits, with got-want ulp per band row.
    for (name, f) in forms {
        let mut rb = 0u32;
        let mut rbn = 0u32;
        for (x, want) in &rows {
            if x.abs() < 9.5e-5 { rbn += 1; if f(*x).to_bits() == *want { rb += 1; } }
        }
        let mut ok = 0;
        for (x, want) in &band { if f(*x).to_bits() == *want { ok += 1; } }
        println!("\n=== {name}: regionB {rb}/{rbn}  band {ok}/{} ===", band.len());
        for (x, want) in &band {
            let d = f(*x).to_bits() as i64 - *want as i64;
            println!("   x={x:+.6e}  got-want {d:+} ulp");
        }
    }

    // Region breakdown for the two most promising unified forms + references.
    println!("\n--- region breakdown (|x| bands) ---");
    for (name, f) in [
        ("pair_x87", atanh_pair_x87 as fn(f64) -> f64),
        ("pair_sse", atanh_pair_sse),
        ("ratio_x87", atanh_ratio_x87),
        ("uni_a", uni_a),
        ("uni_ext", uni_ext),
    ] {
        println!(" {name}:");
        score_region(name, &rows, 0.0, 9.5e-5, &f);       // region B
        score_region(name, &rows, 9.5e-5, 1.02e-4, &f);   // transition band
        score_region(name, &rows, 1.02e-4, 1.0, &f);      // region C (fyl2xp1 out of domain for large x)
    }

    // Per band row under ratio / pair_x87 / pair_sse (decide the switch side).
    println!("\n--- band rows: ratio vs pair got-want ---");
    for (x, want) in &band {
        let r = atanh_ratio_x87(*x).to_bits() as i64 - *want as i64;
        let px = atanh_pair_x87(*x).to_bits() as i64 - *want as i64;
        let ps = atanh_pair_sse(*x).to_bits() as i64 - *want as i64;
        println!("   x={x:+.7e}  ratio {r:+}  pair_x87 {px:+}  pair_sse {ps:+}");
    }

    // Boundary from existing data: largest |x| where pair_x87 is exact and
    // smallest |x| where ratio_x87 is exact — brackets the safe production floor.
    let mut sorted: Vec<(f64, u64)> = rows.iter().cloned().filter(|(x, _)| x.abs() >= 5e-5 && x.abs() <= 2e-4).collect();
    sorted.sort_by(|a, b| a.0.abs().partial_cmp(&b.0.abs()).unwrap());
    println!("\n--- boundary bracket [5e-5, 2e-4], |x| ascending ---");
    for (x, want) in &sorted {
        let pe = atanh_pair_x87(*x).to_bits() == *want;
        let re = atanh_ratio_x87(*x).to_bits() == *want;
        println!("   |x|={:.8e}  pair_x87 {}  ratio_x87 {}", x.abs(), if pe {"OK "} else {"..."}, if re {"OK "} else {"..."});
    }

    // Best unified-below / ratio-above piecewise: sweep switch T.
    println!("\n--- piecewise uni_a(|x|<T) | ratio_x87 else ---");
    let mut best = (0u32, 0f64);
    for &t in &[8e-5f64, 9e-5, 9.5e-5, 1e-4, 1.02e-4, 1.05e-4, 1.1e-4, 1.2e-4, 1.25e-4, 1.5e-4, 2e-4] {
        let mut ok = 0u32;
        for (x, want) in &rows {
            let v = if x.abs() < t { uni_a(*x) } else { atanh_ratio_x87(*x) };
            if v.to_bits() == *want { ok += 1; }
        }
        if ok > best.0 { best = (ok, t); }
        println!("  T={t:.2e}: {ok}/{}", rows.len());
    }
    println!("BEST uni|ratio piecewise: {}/{} at T={:.2e}", best.0, rows.len(), best.1);
}
