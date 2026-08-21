//! W109 G6-01: identify Excel's internal FAITHFUL (non-CR) log1p. The collision variation
//! is DOUBLE-level: tau_double=RN(-n*log1p_EXCEL(r)) with log1p_EXCEL faithful ~1 ULP, not CR.
//! Race candidates {portable-CR, FYL2XP1-hw->double, std ln_1p} through the x87 expm1 against
//! the em oracles (collision, genrate) and full-PMT heldout.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN as CW, ext_add, ext_from_f64 as ef, ext_fyl2x, ext_fyl2xp1, ext_ln2, ext_one,
    ext_to_f64,
};
use std::collections::BTreeMap;

fn l1p_cr(r: f64) -> f64 {
    rx::excel_log1p(r)
}
fn l1p_std(r: f64) -> f64 {
    r.ln_1p()
}
fn l1p_kahan(r: f64) -> f64 {
    let u = 1.0 + r;
    if u == 1.0 {
        r
    } else {
        rx::excel_ln(u) * r / (u - 1.0)
    }
} // Kahan companion, ln=x87 fyl2x
fn l1p_kahan2(r: f64) -> f64 {
    let u = 1.0 + r;
    if u == 1.0 {
        r
    } else {
        rx::excel_ln(u) * (r / (u - 1.0))
    }
} // corr-first assoc
fn l1p_lnfl(r: f64) -> f64 {
    rx::excel_ln(1.0 + r)
} // ln(fl(1+r)) - negative control
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
fn em_for(r: f64, n: f64, l1p: fn(f64) -> f64) -> f64 {
    rx::excel_expm1_internal(-(n * l1p(r)))
}

fn pin_gen(rows: &[(f64, u64)], r: f64, center: f64) -> Option<f64> {
    let cb = center.to_bits() as i64;
    for d in -16..=16i64 {
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
// full PMT for heldout (all fv,ty) - discount combine
fn pmt_full(r: f64, n: f64, pv: f64, fv: f64, ty: f64, l1p: fn(f64) -> f64) -> f64 {
    if r == 0.0 {
        return -(pv + fv) / n;
    }
    let tau = -(n * l1p(r));
    let em = rx::excel_expm1_internal(tau);
    let v = 1.0 + em;
    let tf = 1.0 + r * ty;
    let num = pv + fv * v;
    ((num / em) / tf) * r
}

fn main() {
    let cands: [(&str, fn(f64) -> f64); 6] = [
        ("CR", l1p_cr),
        ("FYL2XP1", l1p_fyl),
        ("std_ln1p", l1p_std),
        ("kahan", l1p_kahan),
        ("kahan2", l1p_kahan2),
        ("lnfl", l1p_lnfl),
    ];
    // em oracles: collision (128-pv configs) + genrate
    for (nm, path) in [
        (
            "collide-configs",
            "../../work/w109/G6-solvers/answers-pmt-collide.json",
        ),
        (
            "genrate",
            "../../work/w109/G6-solvers/answers-pmt-genrate.json",
        ),
    ] {
        // collision file is 128-consecutive-config; genrate is grouped by (r,n). Use generic loader (groups by (r,n)).
        let data = load(path);
        let mut pins: Vec<(f64, f64, f64)> = Vec::new();
        for ((rb, nb), rows) in &data {
            let r = f64::from_bits(*rb);
            let n = f64::from_bits(*nb);
            if (-(n * rx::excel_log1p(r))).abs() >= 1.0 {
                continue;
            }
            let km = em_for(r, n, l1p_cr);
            if let Some(e) = pin_gen(rows, r, km) {
                pins.push((r, n, e));
            }
        }
        print!("{:16} ({} pinned): ", nm, pins.len());
        for (cn, cf) in cands {
            let ok = pins
                .iter()
                .filter(|(r, n, ep)| em_for(*r, *n, cf).to_bits() == ep.to_bits())
                .count();
            print!("  {}:{}/{}", cn, ok, pins.len());
        }
        println!();
    }
    // heldout full PMT
    let ws: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string("../../work/w109/G6-solvers/answers-pmt-heldout.json").unwrap(),
    )
    .unwrap();
    print!("heldout (full PMT, {} rows):", ws.witnesses.len());
    for (cn, cf) in cands {
        let mut ok = 0u32;
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
            let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
            tot += 1;
            if pmt_full(a[0], a[1], a[2], a[3], a[4], cf).to_bits() == want {
                ok += 1;
            }
        }
        print!("  {}:{}/{}", cn, ok, tot);
    }
    println!();
}
