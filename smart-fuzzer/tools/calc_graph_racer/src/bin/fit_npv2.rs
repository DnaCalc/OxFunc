//! W109 Phase-4b: broaden the NPV kernel race with (a) the transcendental
//! discount path — pow(1+r, i) via the confirmed x87 exp/ln (excel_pow_positive)
//! and the system powf — and (b) forward/reverse accumulation with the x87
//! double-rounded add/div ops. Baseline to beat: reverse-Horner division = 117.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

// x87 double-rounded scalar ops (RN53(RN64)) — the proven legacy financial body op.
fn xadd(a: f64, b: f64) -> f64 { xop(a,b,'+') }
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
fn xdiv(a: f64, b: f64) -> f64 { xop(a,b,'/') }
fn xmul(a: f64, b: f64) -> f64 { xop(a,b,'*') }

fn binexp(base: f64, mut n: u64) -> f64 {
    let mut acc = 1.0f64; let mut b = base;
    while n > 0 { if n & 1 == 1 { acc *= b; } n >>= 1; if n > 0 { b *= b; } }
    acc
}

// pow via x87 exp/ln (excel_pow_positive) for base>0
fn xpow(base: f64, i: u32) -> f64 { rx::excel_pow_positive(base, i as f64) }

// ---- candidate NPV kernels ----
fn rev_horner_sse(rate: f64, cf: &[f64]) -> f64 {
    let w = 1.0 + rate; let mut a = 0.0;
    for &c in cf.iter().rev() { a = (a + c) / w; }
    a
}
fn rev_horner_x87(rate: f64, cf: &[f64]) -> f64 {
    let w = xadd(1.0, rate); let mut a = 0.0;
    for &c in cf.iter().rev() { a = xdiv(xadd(a, c), w); }
    a
}
// forward, pow=x87 exp/ln, x87 div+add (XNPV analog)
fn fwd_xpow_x87(rate: f64, cf: &[f64]) -> f64 {
    let base = xadd(1.0, rate); let mut t = 0.0;
    for (i, &c) in cf.iter().enumerate() {
        let p = xpow(base, (i + 1) as u32);
        t = xadd(t, xdiv(c, p));
    }
    t
}
fn fwd_xpow_sse(rate: f64, cf: &[f64]) -> f64 {
    let base = 1.0 + rate; let mut t = 0.0;
    for (i, &c) in cf.iter().enumerate() { t += c / xpow(base, (i + 1) as u32); }
    t
}
fn rev_xpow_sse(rate: f64, cf: &[f64]) -> f64 {
    let base = 1.0 + rate; let mut terms: Vec<f64> = Vec::new();
    for (i, &c) in cf.iter().enumerate() { terms.push(c / xpow(base, (i + 1) as u32)); }
    let mut t = 0.0; for &x in terms.iter().rev() { t += x; } t
}
// forward, pow=system powf, sse
fn fwd_powf_sse(rate: f64, cf: &[f64]) -> f64 {
    let base = 1.0 + rate; let mut t = 0.0;
    for (i, &c) in cf.iter().enumerate() { t += c / base.powf((i + 1) as f64); }
    t
}
fn rev_powf_sse(rate: f64, cf: &[f64]) -> f64 {
    let base = 1.0 + rate; let mut terms: Vec<f64> = Vec::new();
    for (i, &c) in cf.iter().enumerate() { terms.push(c / base.powf((i + 1) as f64)); }
    let mut t = 0.0; for &x in terms.iter().rev() { t += x; } t
}
// forward, pow=binexp, sse
fn fwd_binexp_sse(rate: f64, cf: &[f64]) -> f64 {
    let base = 1.0 + rate; let mut t = 0.0;
    for (i, &c) in cf.iter().enumerate() { t += c / binexp(base, (i + 1) as u64); }
    t
}
// forward running product x87
fn fwd_runprod_x87(rate: f64, cf: &[f64]) -> f64 {
    let w = xadd(1.0, rate); let mut f = 1.0; let mut s = 0.0;
    for &c in cf { f = xmul(f, w); s = xadd(s, xdiv(c, f)); }
    s
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
        ("rev_horner_sse", rev_horner_sse),
        ("rev_horner_x87", rev_horner_x87),
        ("fwd_xpow_x87", fwd_xpow_x87),
        ("fwd_xpow_sse", fwd_xpow_sse),
        ("rev_xpow_sse", rev_xpow_sse),
        ("fwd_powf_sse", fwd_powf_sse),
        ("rev_powf_sse", rev_powf_sse),
        ("fwd_binexp_sse", fwd_binexp_sse),
        ("fwd_runprod_x87", fwd_runprod_x87),
    ];
    for (name, f) in &kernels {
        let mut hit = 0; let mut cl = 0; let mut cltot = 0;
        for (r, cf, want, id) in &obs {
            let is_cl = id.starts_with("npvA");
            if is_cl { cltot += 1; }
            if f(*r, cf).to_bits() == *want { hit += 1; if is_cl { cl += 1; } }
        }
        println!("{name:18} {hit:3}/{}  cluster {cl}/{cltot}", obs.len());
    }
    // dump cluster misses for the best pow form vs rev_horner
    println!("\n-- cluster misses rev_horner_sse vs fwd_xpow_x87 --");
    for (r, cf, want, id) in &obs {
        if !id.starts_with("npvA") { continue; }
        let a = rev_horner_sse(*r, cf).to_bits();
        let b = fwd_xpow_x87(*r, cf).to_bits();
        if a != *want || b != *want {
            println!("  {id:10} rate={r:.10} rhΔ={:+} xpowΔ={:+}", a as i64 - *want as i64, b as i64 - *want as i64);
        }
    }
}
