//! W109 sweep: pin the full piecewise ATANH kernel over ALL live corpora.
//! Region C (ratio-log) is bit-exact for |x| >= ~1e-4; region B (tiny x) needs
//! Excel's x87 fyl2xp1 log1p pair. Score candidate single-forms and piecewise
//! splits to find the exact switch and confirm bit-exactness.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

const CW: u16 = 0x133F;
fn e(x: f64) -> rx::Ext80 { rx::ext_from_f64(x) }
fn tof(v: &rx::Ext80) -> f64 { rx::ext_to_f64(v, CW) }

/// 0.5*(ln1p(x) - ln1p(-x)) via x87 fyl2xp1, extended, single final store.
fn atanh_pair_x87(x: f64) -> f64 {
    let ln2 = rx::ext_ln2();
    let t1 = rx::ext_fyl2xp1(&ln2, &e(x), CW);
    let t2 = rx::ext_fyl2xp1(&ln2, &e(-x), CW);
    let d = rx::ext_sub(&t1, &t2, CW);
    tof(&rx::ext_mul(&d, &e(0.5), CW))
}

/// 0.5*log((1+x)/(1-x)) with the ratio formed in binary64, log via x87 fyl2x.
fn atanh_ratio_x87(x: f64) -> f64 {
    let r = (1.0 + x) / (1.0 - x); // binary64 ratio (double-rounding is load-bearing)
    let l = rx::ext_fyl2x(&rx::ext_ln2(), &e(r), CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}

/// 0.5*log((1+x)/(1-x)) fully in the portable/SSE2 excel_log.
fn atanh_ratio_sse(x: f64) -> f64 {
    let r = (1.0 + x) / (1.0 - x);
    0.5 * rx::excel_ln(r)
}

/// pair with each ln1p STORED to double before subtraction (CRT-call semantics).
fn atanh_pair_stored(x: f64) -> f64 {
    let ln2 = rx::ext_ln2();
    let a = tof(&rx::ext_fyl2xp1(&ln2, &e(x), CW));
    let b = tof(&rx::ext_fyl2xp1(&ln2, &e(-x), CW));
    0.5 * (a - b)
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

fn score(name: &str, rows: &[(f64, u64)], f: &dyn Fn(f64) -> f64) -> u32 {
    let mut ok = 0u32;
    for (x, want) in rows { if f(*x).to_bits() == *want { ok += 1; } }
    println!("{name:42} {ok}/{}", rows.len());
    ok
}

fn main() {
    let rows = load(&[
        "../../work/w109/G4-hyp-answers-atanh.json",
        "../../work/w109/G4-02-answers-atanh-band.json",
        "../../work/w109/G4-02-answers-atanh-gap.json",
        "../../work/w109/G4-02-answers-atanh-switch.json",
    ]);
    println!("{} distinct ATANH rows", rows.len());

    // Single forms (no passthrough guard)
    score("x87 pair (ln1p diff)", &rows, &|x| atanh_pair_x87(x));
    score("x87 ratio-log", &rows, &|x| atanh_ratio_x87(x));
    score("sse ratio-log", &rows, &|x| atanh_ratio_sse(x));
    score("x87 pair STORED halves", &rows, &|x| atanh_pair_stored(x));

    // stored-pair piecewise
    for &t in &[9e-5f64, 1e-4, 1.1e-4, 1.25e-4] {
        let f = |x: f64| if x.abs() < t { atanh_pair_stored(x) } else { atanh_ratio_x87(x) };
        let mut ok = 0u32;
        for (x, want) in &rows { if f(*x).to_bits() == *want { ok += 1; } }
        println!("piecewise STOREDpair|x87ratio T={t:e}: {ok}/{}", rows.len());
    }

    // Piecewise: pair for |x| < T, ratio for |x| >= T. Sweep T over candidate switches.
    let cands = [8e-5f64, 9e-5, 9.5e-5, 1e-4, 1.05e-4, 1.1e-4, 1.15e-4, 1.2e-4, 1.25e-4, 1.3e-4, 1.4e-4];
    let mut best = (0u32, 0f64, "");
    for &t in &cands {
        for (lbl, rf) in [("x87ratio", 0u8), ("sse", 1u8)] {
            let f = |x: f64| -> f64 {
                if x.abs() < t { atanh_pair_x87(x) }
                else if rf == 0 { atanh_ratio_x87(x) } else { atanh_ratio_sse(x) }
            };
            let mut ok = 0u32;
            for (x, want) in &rows { if f(*x).to_bits() == *want { ok += 1; } }
            if ok > best.0 { best = (ok, t, lbl); }
            println!("piecewise pair|{lbl} T={t:e}: {ok}/{}", rows.len());
        }
    }
    println!("BEST piecewise: {}/{} at T={:e} ({})", best.0, rows.len(), best.1, best.2);

    // Miss profile of the best piecewise (x87 pair below T, x87 ratio above)
    let t = best.1;
    let f = |x: f64| if x.abs() < t { atanh_pair_x87(x) } else if best.2 == "sse" { atanh_ratio_sse(x) } else { atanh_ratio_x87(x) };
    let mut shown = 0;
    for (x, want) in &rows {
        if f(*x).to_bits() != *want && shown < 20 {
            shown += 1;
            println!("  MISS x={x:.6e} got-want {:+} ulp", f(*x).to_bits() as i64 - *want as i64);
        }
    }
}

// (switch-boundary corpus appended to the load list via a second bin invocation below)
