//! W109 G6-01 SLP: extended tau -> DOUBLE u -> double Kahan. Hypothesis: PMT forms
//! tau in x87 extended (from extended log1p), so u=RN53(exp(tau_ext)) differs from
//! RN53(exp(tau_d)) by 1 ULP near boundaries = the miss set. Captured u used tau_d;
//! this recomputes u from tau_ext. Kahan stays double (proven best form).
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN as CW, Ext80, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1, ext_from_f64 as e,
    ext_fyl2x, ext_fyl2xp1, ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub,
    ext_to_f64,
};
use std::collections::BTreeMap;

fn tf(x: &Ext80) -> f64 {
    ext_to_f64(x, CW)
}
// extended log1p deliveries
fn log1p_ext_fyl2x(r: f64) -> Ext80 {
    ext_mul(
        &ext_ln2(),
        &ext_fyl2x(&ext_one(), &ext_add(&ext_one(), &e(r), CW), CW),
        CW,
    )
}
fn log1p_ext_fyl2xp1(r: f64) -> Ext80 {
    if r.abs() < 0.292893218813452 {
        ext_mul(&ext_ln2(), &ext_fyl2xp1(&ext_one(), &e(r), CW), CW)
    } else {
        log1p_ext_fyl2x(r)
    }
}
fn exp_ext(tau: &Ext80) -> Ext80 {
    let t = ext_mul(tau, &ext_l2e(), CW);
    let k = ext_rndint(&t, CW);
    let f = ext_sub(&t, &k, CW);
    let neg = tf(&f) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, CW), CW);
    let mut m = ext_add(&w, &ext_one(), CW);
    if neg {
        m = ext_div(&ext_one(), &m, CW);
    }
    ext_scale(&m, &k, CW)
}
fn tau_d(r: f64, n: f64) -> f64 {
    -(n * rx::excel_log1p(r))
}

// variants: which log1p delivery, and numerator tau (ext-spilled vs tau_d)
fn em(r: f64, n: f64, fyl2xp1: bool, num_from_ext: bool) -> f64 {
    let l1p = if fyl2xp1 {
        log1p_ext_fyl2xp1(r)
    } else {
        log1p_ext_fyl2x(r)
    };
    let tau_e = ext_chs(&ext_mul(&e(n), &l1p, CW), CW); // -n*log1p ext
    let t_d = tf(&tau_e); // tau spilled to double
    let u = ext_to_f64(&exp_ext(&tau_e), CW); // u = RN53(exp(tau_ext))
    if u == 1.0 {
        return t_d;
    }
    if t_d.abs() >= 1.0 {
        return u - 1.0;
    }
    let lnu = rx::excel_ln(u);
    let tnum = if num_from_ext { t_d } else { tau_d(r, n) };
    (u - 1.0) * tnum / lnu
}
// reference: pure double (captured u path)
fn em_ref(r: f64, n: f64) -> f64 {
    rx::excel_expm1_internal(tau_d(r, n))
}

fn pin_po2(rows: &[(f64, u64)], r: f64) -> Option<f64> {
    let (pv, pmtb) = rows[rows.len() / 2];
    let pmt = f64::from_bits(pmtb);
    let center = pv / (pmt / r);
    let cb = center.to_bits() as i64;
    for d in -8..=8i64 {
        let em = f64::from_bits((cb + d) as u64);
        if em == -1.0 {
            continue;
        }
        if rows
            .iter()
            .all(|(pv, want)| ((pv / em) * r).to_bits() == *want)
        {
            return Some(em);
        }
    }
    None
}
fn pin_gen(rows: &[(f64, u64)], r: f64, center: f64) -> Option<f64> {
    let cb = center.to_bits() as i64;
    for d in -12..=12i64 {
        let em = f64::from_bits((cb + d) as u64);
        if em >= 0.0 {
            continue;
        }
        if rows
            .iter()
            .all(|(pv, want)| ((pv / em) * r).to_bits() == *want)
        {
            return Some(em);
        }
    }
    None
}
fn load(p: &str) -> BTreeMap<(u64, u64), Vec<(f64, u64)>> {
    let ws: WitnessSet = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    let mut m = BTreeMap::new();
    for w in &ws.witnesses {
        let a: Vec<f64> = w
            .args
            .iter()
            .filter_map(|x| match x {
                WitnessArg::Scalar(s) => parse_bits_hex(s),
                _ => None,
            })
            .collect();
        if a.len() != 5 || a[3] != 0.0 || a[4] != 0.0 {
            continue;
        }
        let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
        m.entry((a[0].to_bits(), a[1].to_bits()))
            .or_insert_with(Vec::new)
            .push((a[2], want));
    }
    m
}
fn main() {
    for (src, path, po2) in [
        (
            "po2n",
            "../../work/w109/G6-solvers/answers-pmt-po2n.json",
            true,
        ),
        (
            "gen",
            "../../work/w109/G6-solvers/answers-pmt-genrate.json",
            false,
        ),
    ] {
        let data = load(path);
        let mut pins: Vec<(f64, f64, f64)> = Vec::new();
        for ((rb, nb), rows) in &data {
            let r = f64::from_bits(*rb);
            let n = f64::from_bits(*nb);
            if tau_d(r, n).abs() >= 1.0 {
                continue;
            }
            let km = em_ref(r, n);
            let p = if po2 {
                pin_po2(rows, r)
            } else {
                pin_gen(rows, r, km)
            };
            if let Some(e) = p {
                pins.push((r, n, e));
            }
        }
        let refok = pins
            .iter()
            .filter(|(r, n, ep)| em_ref(*r, *n).to_bits() == ep.to_bits())
            .count();
        println!(
            "=== {} ({} pts)  ref(double Kahan)={} ===",
            src,
            pins.len(),
            refok
        );
        for (nm, fp, ne) in [
            ("fyl2x, num=tau_d", false, false),
            ("fyl2x, num=ext", false, true),
            ("fyl2xp1, num=tau_d", true, false),
            ("fyl2xp1, num=ext", true, true),
        ] {
            let ok = pins
                .iter()
                .filter(|(r, n, ep)| em(*r, *n, fp, ne).to_bits() == ep.to_bits())
                .count();
            // how many does the ext-u path FIX vs ref, and how many does it BREAK
            let mut fixed = 0;
            let mut broke = 0;
            for (r, n, ep) in &pins {
                let rf = em_ref(*r, *n).to_bits() == ep.to_bits();
                let ex = em(*r, *n, fp, ne).to_bits() == ep.to_bits();
                if ex && !rf {
                    fixed += 1;
                }
                if rf && !ex {
                    broke += 1;
                }
            }
            println!(
                "  {:<20} {:>3}/{}   (fixed {} / broke {})",
                nm,
                ok,
                pins.len(),
                fixed,
                broke
            );
        }
    }
}
