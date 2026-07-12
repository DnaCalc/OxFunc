//! W109 bounded offline probe: additional NPV staging variants beyond
//! fit_npv2.rs's reverse-Horner leader (117/142, cluster 36/47). Scores
//! reverse-Horner recip forms, forward running-product forms, integer-binexp
//! pow forms, and two association probes on reverse-Horner, against the same
//! npv-r0/r1 corpus used by fit_npv2. OFFLINE MEASUREMENT ONLY — no
//! production code touched.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

// x87 double-rounded scalar ops (RN53(RN64)) — the proven legacy financial body op.
fn xadd(a: f64, b: f64) -> f64 { xop(a, b, '+') }
fn xop(a: f64, b: f64, o: char) -> f64 {
    let cw = 0x133Fu16;
    let ea = rx::ext_from_f64(a);
    let eb = rx::ext_from_f64(b);
    let r = match o {
        '+' => rx::ext_add(&ea, &eb, cw),
        '-' => rx::ext_sub(&ea, &eb, cw),
        '*' => rx::ext_mul(&ea, &eb, cw),
        _ => rx::ext_div(&ea, &eb, cw),
    };
    rx::ext_to_f64(&r, cw)
}
fn xdiv(a: f64, b: f64) -> f64 { xop(a, b, '/') }
fn xmul(a: f64, b: f64) -> f64 { xop(a, b, '*') }
fn xrecip(x: f64) -> f64 { rx::x87_recip(x) }

fn binexp_x87(base: f64, mut n: u64) -> f64 {
    let mut acc = 1.0f64;
    let mut b = base;
    while n > 0 {
        if n & 1 == 1 { acc = xmul(acc, b); }
        n >>= 1;
        if n > 0 { b = xmul(b, b); }
    }
    acc
}

// ---- baselines (copied from fit_npv2.rs for side-by-side comparison) ----
fn rev_horner_sse(rate: f64, cf: &[f64]) -> f64 {
    let w = 1.0 + rate;
    let mut a = 0.0;
    for &c in cf.iter().rev() { a = (a + c) / w; }
    a
}
fn fwd_powf_sse(rate: f64, cf: &[f64]) -> f64 {
    let base = 1.0 + rate;
    let mut t = 0.0;
    for (i, &c) in cf.iter().enumerate() { t += c / base.powf((i + 1) as f64); }
    t
}

// ---- new staging variants ----

// 1. reverse-Horner, reciprocal computed once, sse
fn rev_horner_recip_sse(rate: f64, cf: &[f64]) -> f64 {
    let w = 1.0 + rate;
    let ir = 1.0 / w;
    let mut a = 0.0;
    for &c in cf.iter().rev() { a = (a + c) * ir; }
    a
}

// 2. reverse-Horner, reciprocal computed once via x87 recip, x87 add/mul
fn rev_horner_recip_x87(rate: f64, cf: &[f64]) -> f64 {
    let w = xadd(1.0, rate);
    let ir = xrecip(w);
    let mut a = 0.0;
    for &c in cf.iter().rev() { a = xmul(xadd(a, c), ir); }
    a
}

// 3. forward running-product, x87 recip per-term
fn fwd_runprod_recip_x87(rate: f64, cf: &[f64]) -> f64 {
    let w = xadd(1.0, rate);
    let mut factor = 1.0;
    let mut s = 0.0;
    for &c in cf {
        factor = xmul(factor, w);
        let term = xmul(c, xrecip(factor));
        s = xadd(s, term);
    }
    s
}

// 4. forward running-product, pure sse div
fn fwd_runprod_div_sse(rate: f64, cf: &[f64]) -> f64 {
    let w = 1.0 + rate;
    let mut factor = 1.0;
    let mut s = 0.0;
    for &c in cf {
        factor *= w;
        s += c / factor;
    }
    s
}

// 5. forward running-product terms (x87 div, x87 factor), summed in REVERSE order (x87 add)
fn fwd_runprod_reversed_sum_x87(rate: f64, cf: &[f64]) -> f64 {
    let w = xadd(1.0, rate);
    let mut factor = 1.0;
    let mut terms: Vec<f64> = Vec::with_capacity(cf.len());
    for &c in cf {
        factor = xmul(factor, w);
        terms.push(xdiv(c, factor));
    }
    let mut s = 0.0;
    for &t in terms.iter().rev() { s = xadd(s, t); }
    s
}

// 6. forward running-product terms (sse running product + div), summed FORWARD (sse add)
fn fwd_runprod_fwd_sum_sse_revterms(rate: f64, cf: &[f64]) -> f64 {
    let w = 1.0 + rate;
    let mut factor = 1.0;
    let mut terms: Vec<f64> = Vec::with_capacity(cf.len());
    for &c in cf {
        factor *= w;
        terms.push(c / factor);
    }
    let mut s = 0.0;
    for &t in terms.iter() { s += t; }
    s
}

// 7. integer-binexp pow, x87 throughout, term = c / pow (x87 div)
fn fwd_binexp_x87(rate: f64, cf: &[f64]) -> f64 {
    let w = xadd(1.0, rate);
    let mut s = 0.0;
    for (i, &c) in cf.iter().enumerate() {
        let pow = binexp_x87(w, (i + 1) as u64);
        let term = xdiv(c, pow);
        s = xadd(s, term);
    }
    s
}

// 8. integer-binexp pow, x87 throughout, term = c * recip(pow)
fn fwd_binexp_recip_x87(rate: f64, cf: &[f64]) -> f64 {
    let w = xadd(1.0, rate);
    let mut s = 0.0;
    for (i, &c) in cf.iter().enumerate() {
        let pow = binexp_x87(w, (i + 1) as u64);
        let term = xmul(c, xrecip(pow));
        s = xadd(s, term);
    }
    s
}

// 9. reverse-Horner, association probe: split a/w + c/w (two sse divisions) reversed
fn rev_horner_split_sse(rate: f64, cf: &[f64]) -> f64 {
    let w = 1.0 + rate;
    let mut a = 0.0;
    for &c in cf.iter().rev() { a = a / w + c / w; }
    a
}

// 10. reverse-Horner x87, but recompute w = xadd(1,rate) INSIDE each iteration (no CSE)
fn rev_horner_x87_wfused(rate: f64, cf: &[f64]) -> f64 {
    let mut a = 0.0;
    for &c in cf.iter().rev() {
        let w = xadd(1.0, rate);
        a = xdiv(xadd(a, c), w);
    }
    a
}

fn main() {
    let mut obs: Vec<(f64, Vec<f64>, u64, String)> = Vec::new();
    for file in ["../../work/w109/G6-solvers/answers-npv-r0.json",
                 "../../work/w109/G6-solvers/answers-npv-r1.json"] {
        let ws: WitnessSet = serde_json::from_str(&std::fs::read_to_string(file).expect("read")).expect("parse");
        for w in &ws.witnesses {
            let rate = match &w.args[0] { WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(), _ => continue };
            let cf: Vec<f64> = match &w.args[1] { WitnessArg::Array(items) => items.iter().map(|s| parse_bits_hex(s).unwrap()).collect(), _ => continue };
            if let Some(want) = parse_bits_hex(&w.expected_bits) { obs.push((rate, cf, want.to_bits(), w.id.clone().unwrap_or_default())); }
        }
    }
    println!("{} NPV rows", obs.len());

    type K = (&'static str, fn(f64, &[f64]) -> f64);
    let kernels: Vec<K> = vec![
        // baselines for comparison
        ("rev_horner_sse", rev_horner_sse),
        ("fwd_powf_sse", fwd_powf_sse),
        // new staging variants
        ("rev_horner_recip_sse", rev_horner_recip_sse),
        ("rev_horner_recip_x87", rev_horner_recip_x87),
        ("fwd_runprod_recip_x87", fwd_runprod_recip_x87),
        ("fwd_runprod_div_sse", fwd_runprod_div_sse),
        ("fwd_runprod_reversed_sum_x87", fwd_runprod_reversed_sum_x87),
        ("fwd_runprod_fwd_sum_sse_revterms", fwd_runprod_fwd_sum_sse_revterms),
        ("fwd_binexp_x87", fwd_binexp_x87),
        ("fwd_binexp_recip_x87", fwd_binexp_recip_x87),
        ("rev_horner_split_sse", rev_horner_split_sse),
        ("rev_horner_x87_wfused", rev_horner_x87_wfused),
    ];

    let mut results: Vec<(&str, usize, usize, usize)> = Vec::new(); // name, hit, cl, cltot
    for (name, f) in &kernels {
        let mut hit = 0;
        let mut cl = 0;
        let mut cltot = 0;
        for (r, cf, want, id) in &obs {
            let is_cl = id.starts_with("npvA");
            if is_cl { cltot += 1; }
            if f(*r, cf).to_bits() == *want {
                hit += 1;
                if is_cl { cl += 1; }
            }
        }
        println!("{name:34} {hit:3}/{}  cluster {cl}/{cltot}", obs.len());
        results.push((name, hit, cl, cltot));
    }

    // find best 2 by total hits (break ties by cluster hits)
    let mut ranked: Vec<&(&str, usize, usize, usize)> = results.iter().collect();
    ranked.sort_by(|a, b| (b.1, b.2).cmp(&(a.1, a.2)));
    println!("\n-- top 2 by total hits --");
    for r in ranked.iter().take(2) {
        println!("  {} : {}/{}  cluster {}/{}", r.0, r.1, obs.len(), r.2, r.3);
    }

    // dump per-cluster-row delta for the best 2 (by name lookup back into kernels)
    for rk in ranked.iter().take(2) {
        let name = rk.0;
        let f = kernels.iter().find(|(n, _)| *n == name).unwrap().1;
        println!("\n-- cluster rows for {name} (npvA*) --");
        let mut max_abs_delta: i64 = 0;
        for (r, cf, want, id) in &obs {
            if !id.starts_with("npvA") { continue; }
            let got = f(*r, cf).to_bits();
            let delta = got as i64 - *want as i64;
            if delta.abs() > max_abs_delta { max_abs_delta = delta.abs(); }
            let marker = if delta == 0 { "" } else { " <-- MISS" };
            println!("  {id:10} rate={r:.10} \u{394}={delta:+}{marker}");
        }
        println!("  max|\u{394}| on cluster = {max_abs_delta}");
    }

    // also report max|delta| on cluster for every variant, to find the smallest-max one
    println!("\n-- max|\u{394}| on npvA cluster, all variants --");
    let mut maxdeltas: Vec<(&str, i64, usize, usize)> = Vec::new();
    for (name, f) in &kernels {
        let mut max_abs_delta: i64 = 0;
        let mut cl = 0;
        let mut cltot = 0;
        for (r, cf, want, id) in &obs {
            if !id.starts_with("npvA") { continue; }
            cltot += 1;
            let got = f(*r, cf).to_bits();
            let delta = got as i64 - *want as i64;
            if delta == 0 { cl += 1; }
            if delta.abs() > max_abs_delta { max_abs_delta = delta.abs(); }
        }
        maxdeltas.push((name, max_abs_delta, cl, cltot));
    }
    maxdeltas.sort_by_key(|x| x.1);
    for (name, maxd, cl, cltot) in &maxdeltas {
        println!("  {name:34} max|\u{394}|={maxd:4}  cluster {cl}/{cltot}");
    }
}
