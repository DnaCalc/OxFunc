//! W109 G6-01 CORE LANE: extended-tau expm1 staging scored on the COLLISION oracle.
//! Two axes the prior spill-search (race_collide_search) did NOT cover:
//!   (1) log1p delivery = fyl2x(ln2, EXACT ext(1+r)) vs fyl2xp1(ln2,r)  [preserves r's tail either way,
//!       different 64-bit hardware tail]
//!   (2) expm1 staging incl. F2XM1-native-signed and fFEXP-recip-minus-1, with per-NODE PC53/PC64.
//! em pinned model-free per config (128-pv ·r reconstruction). Reports any config EXCEEDING 83/116,
//! plus a per-group residual dump for the leading candidate to see if it tracks the within-group tail.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1,
    ext_from_f64 as ef, ext_fyl2x, ext_fyl2xp1, ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint,
    ext_scale, ext_sub, ext_to_f64,
};

const RC_RZ: u16 = 0x0C00;
fn tf(x: &Ext80) -> f64 {
    ext_to_f64(x, CW_PC64_RN)
}

// ---------- log1p deliveries (produce ln(1+r) extended, base e) ----------
// variant 0: fyl2xp1(ln2,r) for |r|<0.293 else fyl2x(ln2,1+r)
// variant 1: fyl2x(ln2, EXACT ext(1+r)) always  (ext_add(1,r) is exact for these small r)
// variant 2: double excel_log1p spilled to ext
fn ln1p_ext(r: f64, var: u8, cw: u16) -> Ext80 {
    match var {
        0 => {
            if r.abs() < 0.292893218813452 {
                ext_fyl2xp1(&ext_ln2(), &ef(r), cw)
            } else {
                ext_fyl2x(&ext_ln2(), &ext_add(&ext_one(), &ef(r), cw), cw)
            }
        }
        1 => ext_fyl2x(&ext_ln2(), &ext_add(&ext_one(), &ef(r), cw), cw),
        _ => ef(rx::excel_log1p(r)),
    }
}
// log2(1+r) extended for the base-2 F2XM1-native path
fn log2_1p_ext(r: f64, var: u8, cw: u16) -> Ext80 {
    match var {
        0 => {
            if r.abs() < 0.292893218813452 {
                ext_fyl2xp1(&ext_one(), &ef(r), cw)
            } else {
                ext_fyl2x(&ext_one(), &ext_add(&ext_one(), &ef(r), cw), cw)
            }
        }
        _ => ext_fyl2x(&ext_one(), &ext_add(&ext_one(), &ef(r), cw), cw),
    }
}

// fFEXP: exp of extended tau (Excel's confirmed recip chain). cw applies to all nodes.
fn fexp(tau: &Ext80, cw: u16) -> Ext80 {
    let t = ext_mul(tau, &ext_l2e(), cw);
    let k = ext_rndint(&t, cw);
    let f = ext_sub(&t, &k, cw);
    let neg = ext_to_f64(&f, cw) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, cw), cw);
    let mut m = ext_add(&w, &ext_one(), cw);
    if neg {
        m = ext_div(&ext_one(), &m, cw);
    }
    ext_scale(&m, &k, cw)
}

#[derive(Clone, Copy)]
struct Cfg {
    lnvar: u8,
    cw_ln: u16,
    cw_mul: u16,
    cw_exp: u16,
    cw_corr: u16,
    form: u8,
    lndel: u8,
    assoc: u8,
    fin: u16,
}
// form: 0 = u-1 (fFEXP recip, minus 1)
//       1 = F2XM1-native-signed (t=tau*l2e; direct F2XM1(t) if|t|<=1 else FSCALE reduce)  [base e via l2e]
//       2 = F2XM1-native base-2 (y=-n*log2(1+r); direct/FSCALE)
//       3 = Kahan (u-1)*tau/ln(u)
fn em(r: f64, n: f64, c: Cfg) -> f64 {
    let ln1p = ln1p_ext(r, c.lnvar, c.cw_ln);
    let tau = ext_chs(&ext_mul(&ef(n), &ln1p, c.cw_mul), c.cw_mul);
    let em = match c.form {
        0 => {
            let u = fexp(&tau, c.cw_exp);
            ext_sub(&u, &ext_one(), c.cw_exp)
        }
        1 => {
            // 2^t - 1, t = tau*log2e
            let t = ext_mul(&tau, &ext_l2e(), c.cw_exp);
            if tf(&t).abs() <= 1.0 {
                ext_f2xm1(&t, c.cw_exp)
            } else {
                let k = ext_rndint(&t, c.cw_exp);
                let f = ext_sub(&t, &k, c.cw_exp);
                let w = ext_f2xm1(&f, c.cw_exp);
                ext_sub(
                    &ext_scale(&ext_add(&w, &ext_one(), c.cw_exp), &k, c.cw_exp),
                    &ext_one(),
                    c.cw_exp,
                )
            }
        }
        2 => {
            // base-2 native: y=-n*log2(1+r)
            let y = ext_chs(
                &ext_mul(&ef(n), &log2_1p_ext(r, c.lnvar, c.cw_ln), c.cw_mul),
                c.cw_mul,
            );
            if tf(&y).abs() <= 1.0 {
                ext_f2xm1(&y, c.cw_exp)
            } else {
                let k = ext_rndint(&y, c.cw_exp);
                let f = ext_sub(&y, &k, c.cw_exp);
                let w = ext_f2xm1(&f, c.cw_exp);
                ext_sub(
                    &ext_scale(&ext_add(&w, &ext_one(), c.cw_exp), &k, c.cw_exp),
                    &ext_one(),
                    c.cw_exp,
                )
            }
        }
        _ => {
            let u = fexp(&tau, c.cw_exp);
            let b = ext_sub(&u, &ext_one(), c.cw_corr);
            let l = if c.lndel == 0 {
                ext_fyl2x(&ext_ln2(), &u, c.cw_corr)
            } else {
                ext_fyl2xp1(&ext_ln2(), &ext_sub(&u, &ext_one(), c.cw_corr), c.cw_corr)
            };
            match c.assoc {
                0 => ext_div(&ext_mul(&b, &tau, c.cw_corr), &l, c.cw_corr),
                1 => ext_mul(&b, &ext_div(&tau, &l, c.cw_corr), c.cw_corr),
                _ => ext_mul(&ext_div(&tau, &l, c.cw_corr), &b, c.cw_corr),
            }
        }
    };
    ext_to_f64(&em, c.fin)
}

fn pin_gen(rows: &[(f64, u64)], center: f64) -> Option<f64> {
    let cb = center.to_bits() as i64;
    for d in -20..=20i64 {
        let e = f64::from_bits((cb + d) as u64);
        if e >= 0.0 {
            continue;
        }
        if rows
            .iter()
            .all(|(pv, want)| ((pv / e) * (rows[0].0 * 0.0 + 1.0)).to_bits() == *want || true)
        {
        }
    }
    None
}
// proper pin: need r; do it inline in main
fn main() {
    let ws: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string("../../work/w109/G6-solvers/answers-pmt-collide.json").unwrap(),
    )
    .unwrap();
    let wl = &ws.witnesses;
    let ncfg = wl.len() / 128;
    // (r,n,em_pinned, tau0_bits)
    let mut pins: Vec<(f64, f64, f64, u64)> = Vec::new();
    for ci in 0..ncfg {
        let mut rows = Vec::new();
        let mut rn = (0.0f64, 0.0f64);
        for j in 0..128 {
            let w = &wl[ci * 128 + j];
            let a: Vec<f64> = w
                .args
                .iter()
                .filter_map(|x| match x {
                    WitnessArg::Scalar(s) => parse_bits_hex(s),
                    _ => None,
                })
                .collect();
            rn = (a[0], a[1]);
            rows.push((a[2], parse_bits_hex(&w.expected_bits).unwrap().to_bits()));
        }
        let (r, n) = rn;
        let tau0 = -(n * rx::excel_log1p(r));
        let center = rx::excel_expm1_internal(tau0);
        let cb = center.to_bits() as i64;
        let mut found = None;
        for d in -20..=20i64 {
            let e = f64::from_bits((cb + d) as u64);
            if e >= 0.0 {
                continue;
            }
            if rows
                .iter()
                .all(|(pv, want)| ((pv / e) * r).to_bits() == *want)
            {
                found = Some(e);
                break;
            }
        }
        if let Some(e) = found {
            pins.push((r, n, e, tau0.to_bits()));
        }
    }
    let _ = pin_gen;
    let ntot = pins.len();
    let kbase = pins
        .iter()
        .filter(|(r, n, ep, _)| {
            rx::excel_expm1_internal(-(n * rx::excel_log1p(*r))).to_bits() == ep.to_bits()
        })
        .count();
    println!(
        "collision pinned: {}   double-Kahan baseline: {}/{}",
        ntot, kbase, ntot
    );

    // ---- brute force ----
    let pcs = [("53", CW_PC53_RN), ("64", CW_PC64_RN)];
    let fins = [("RN", CW_PC64_RN), ("RZ", CW_PC64_RN | RC_RZ)];
    let mut best: Vec<(u32, String, Cfg)> = Vec::new();
    for lnvar in 0u8..3 {
        for (_lnn, cw_ln) in pcs {
            for (_mn, cw_mul) in pcs {
                for (_en, cw_exp) in pcs {
                    for form in 0u8..4 {
                        let corrspace: &[u16] = if form == 3 {
                            &[CW_PC53_RN, CW_PC64_RN]
                        } else {
                            &[CW_PC64_RN]
                        };
                        for &cw_corr in corrspace {
                            let (lnspace, assocspace): (&[u8], &[u8]) = if form == 3 {
                                (&[0, 1], &[0, 1, 2])
                            } else {
                                (&[0], &[0])
                            };
                            for &lndel in lnspace {
                                for &assoc in assocspace {
                                    for (_fnn, fin) in fins {
                                        let c = Cfg {
                                            lnvar,
                                            cw_ln,
                                            cw_mul,
                                            cw_exp,
                                            cw_corr,
                                            form,
                                            lndel,
                                            assoc,
                                            fin,
                                        };
                                        let ok = pins
                                            .iter()
                                            .filter(|(r, n, ep, _)| {
                                                em(*r, *n, c).to_bits() == ep.to_bits()
                                            })
                                            .count()
                                            as u32;
                                        let desc = format!(
                                            "lnvar={} cwln={} cwmul={} cwexp={} form={} corr={:04x} ln={} assoc={} fin={}",
                                            lnvar,
                                            if cw_ln == CW_PC53_RN { "53" } else { "64" },
                                            if cw_mul == CW_PC53_RN { "53" } else { "64" },
                                            if cw_exp == CW_PC53_RN { "53" } else { "64" },
                                            form,
                                            cw_corr,
                                            lndel,
                                            assoc,
                                            if fin == CW_PC64_RN { "RN" } else { "RZ" }
                                        );
                                        best.push((ok, desc, c));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    best.sort_by(|a, b| b.0.cmp(&a.0));
    println!("\nTOP 12 configs on collision ({} pts):", ntot);
    for (ok, d, _) in best.iter().take(12) {
        println!("  {:>3}/{}  {}", ok, ntot, d);
    }
    let over = best.iter().filter(|x| x.0 > kbase as u32).count();
    println!("\nconfigs EXCEEDING double-Kahan {}: {}", kbase, over);

    // best among TRULY-EXTENDED configs (all PC64, lnvar 0 or 1 = extended log1p, no double spill)
    let ext_best = best
        .iter()
        .filter(|(_, _, c)| {
            c.lnvar < 2
                && c.cw_ln == CW_PC64_RN
                && c.cw_mul == CW_PC64_RN
                && c.cw_exp == CW_PC64_RN
                && (c.form != 3 || c.cw_corr == CW_PC64_RN)
        })
        .max_by_key(|x| x.0);
    if let Some((ok, d, _)) = ext_best {
        println!(
            "BEST purely-EXTENDED (all PC64) config: {}/{}  {}",
            ok, ntot, d
        );
    }

    // ---- per-group residual dump for the single best config ----
    let (bok, bd, bc) = best[0].clone();
    println!(
        "\n=== per-group residual (em_cand - em_pinned, ulps) for BEST: {}/{}  [{}] ===",
        bok, ntot, bd
    );
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<u64, Vec<(f64, f64, f64)>> = BTreeMap::new();
    for (r, n, ep, tb) in &pins {
        groups.entry(*tb).or_default().push((*r, *n, *ep));
    }
    fn sord(u: u64) -> i64 {
        if u < 1 << 63 {
            u as i64
        } else {
            -((u ^ (1u64 << 63)) as i64)
        }
    }
    let mut shown = 0;
    for (tb, lst) in &groups {
        if shown >= 6 {
            break;
        }
        shown += 1;
        let mut hit = 0;
        let mut line = format!("group tau0={:016x} ({}): ", tb, lst.len());
        for (r, n, ep) in lst {
            let g = em(*r, *n, bc);
            let d = sord(g.to_bits()) - sord(ep.to_bits());
            if d == 0 {
                hit += 1;
            }
            line.push_str(&format!("{:+} ", d));
        }
        println!("  {}  [{}/{}]", line, hit, lst.len());
    }
    // Does the BEST purely-extended config reproduce the DISTINCT-em structure of the collision groups?
    if let Some((_, exd, exc)) = best
        .iter()
        .filter(|(_, _, c)| {
            c.lnvar < 2
                && c.cw_ln == CW_PC64_RN
                && c.cw_mul == CW_PC64_RN
                && c.cw_exp == CW_PC64_RN
                && (c.form != 3 || c.cw_corr == CW_PC64_RN)
        })
        .max_by_key(|x| x.0)
    {
        println!(
            "\n=== distinct-em reproduction by BEST-EXTENDED [{}] ===",
            exd
        );
        println!("  (Excel distinct em / config-count  vs  candidate distinct em)");
        let mut shown = 0;
        for (tb, lst) in &groups {
            if shown >= 8 {
                break;
            }
            shown += 1;
            use std::collections::BTreeSet;
            let exset: BTreeSet<u64> = lst.iter().map(|(_, _, ep)| ep.to_bits()).collect();
            let cnset: BTreeSet<u64> = lst
                .iter()
                .map(|(r, n, _)| em(*r, *n, *exc).to_bits())
                .collect();
            let hit = lst
                .iter()
                .filter(|(r, n, ep)| em(*r, *n, *exc).to_bits() == ep.to_bits())
                .count();
            println!(
                "  group {:016x} ({:2}): Excel {} distinct, cand {} distinct, exact {}/{}",
                tb,
                lst.len(),
                exset.len(),
                cnset.len(),
                hit,
                lst.len()
            );
        }
    }
}
