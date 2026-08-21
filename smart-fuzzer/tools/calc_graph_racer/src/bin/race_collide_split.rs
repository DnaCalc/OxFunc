//! W109 G6-01 CORE LANE: hi/lo-split expm1 on the COLLISION oracle.
//! Motivation: on collision, double-Kahan (tau_double) = 83/116 but CR-extended = 72/116, and the
//! best PURELY-extended hardware chain = 72 producing FEWER distinct em than Excel. Yet em needs the
//! tau tail. Model: em = expm1_double(tau0) + expm1'(tau0) * tau_lo, tau0 = RN53(tau_ext),
//! tau_lo = tau_ext - tau0 (extended tail). base expm1 = Excel's x87 excel_expm1_internal (the 83-model).
//! Sweep log1p delivery, derivative (exp vs 1+E), product/add PC, final rounding. Report any >83.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, ext_add, ext_chs, ext_from_f64 as ef, ext_fyl2x, ext_fyl2xp1,
    ext_ln2, ext_mul, ext_one, ext_sub, ext_to_f64,
};

const RC_RZ: u16 = 0x0C00;
fn ln1p_ext(r: f64, var: u8, cw: u16) -> Ext80 {
    match var {
        0 => {
            if r.abs() < 0.292893218813452 {
                ext_fyl2xp1(&ext_ln2(), &ef(r), cw)
            } else {
                ext_fyl2x(&ext_ln2(), &ext_add(&ext_one(), &ef(r), cw), cw)
            }
        }
        _ => ext_fyl2x(&ext_ln2(), &ext_add(&ext_one(), &ef(r), cw), cw),
    }
}
#[derive(Clone, Copy)]
struct Cfg {
    lnvar: u8,
    deriv: u8,
    cw_prod: u16,
    cw_add: u16,
    fin: u16,
    base: u8,
}
fn em(r: f64, n: f64, c: Cfg) -> f64 {
    let ln1p = ln1p_ext(r, c.lnvar, CW_PC64_RN);
    let tau_ext = ext_chs(&ext_mul(&ef(n), &ln1p, CW_PC64_RN), CW_PC64_RN);
    let tau0 = ext_to_f64(&tau_ext, CW_PC64_RN); // spill to double
    let tlo = ext_sub(&tau_ext, &ef(tau0), CW_PC64_RN); // extended tail
    let e_base = match c.base {
        0 => rx::excel_expm1_internal(tau0),
        _ => {
            // CR-ish double expm1 via ext exp - 1 at PC64 of tau0
            let ee = rx::excel_exp(tau0);
            ee - 1.0
        }
    };
    let u = match c.deriv {
        0 => rx::excel_exp(tau0),
        _ => 1.0 + e_base,
    };
    let corr = ext_mul(&ef(u), &tlo, c.cw_prod);
    ext_to_f64(&ext_add(&ef(e_base), &corr, c.cw_add), c.fin)
}
fn main() {
    let ws: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string("../../work/w109/G6-solvers/answers-pmt-collide.json").unwrap(),
    )
    .unwrap();
    let wl = &ws.witnesses;
    let ncfg = wl.len() / 128;
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
    let pcs = [("53", CW_PC53_RN), ("64", CW_PC64_RN)];
    let fins = [("RN", CW_PC64_RN), ("RZ", CW_PC64_RN | RC_RZ)];
    let mut best: Vec<(u32, String)> = Vec::new();
    for lnvar in 0u8..2 {
        for base in 0u8..2 {
            for deriv in 0u8..2 {
                for (pn, cw_prod) in pcs {
                    for (an, cw_add) in pcs {
                        for (fnn, fin) in fins {
                            let c = Cfg {
                                lnvar,
                                deriv,
                                cw_prod,
                                cw_add,
                                fin,
                                base,
                            };
                            let ok = pins
                                .iter()
                                .filter(|(r, n, ep, _)| em(*r, *n, c).to_bits() == ep.to_bits())
                                .count() as u32;
                            best.push((
                                ok,
                                format!(
                                    "lnvar={} base={} deriv={} prod={} add={} fin={}",
                                    lnvar,
                                    if base == 0 { "x87int" } else { "exp-1" },
                                    if deriv == 0 { "exp" } else { "1+E" },
                                    pn,
                                    an,
                                    fnn
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }
    best.sort_by(|a, b| b.0.cmp(&a.0));
    println!("\nTOP split configs on collision ({} pts):", ntot);
    for (ok, d) in best.iter().take(12) {
        println!("  {:>3}/{}  {}", ok, ntot, d);
    }
    println!(
        "configs EXCEEDING {}: {}",
        kbase,
        best.iter().filter(|x| x.0 > kbase as u32).count()
    );
}
