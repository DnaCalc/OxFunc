//! agent-P: score em candidates at the RAW pmt level (the true observable),
//! to check whether pinned-em mismatches are real pmt divergences or absorbed
//! by the pmt divide. Loads batch+answers (args [rate,nper,pv,fv,type]).
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

const CW: u16 = rx::CW_PC64_RN;

// best em candidate: internal-Kahan, f64 (u-1)*t/lnu
fn em_kahan(r: f64, n: f64) -> f64 {
    let t = rx::x87_mul(-n, rx::excel_log1p(r)); // for n=1 this is exact negation
    let u = rx::excel_exp(t);
    if u == 1.0 {
        return t;
    }
    if t.abs() < 1.0 {
        (u - 1.0) * t / rx::excel_ln(u)
    } else {
        u - 1.0
    }
}

// x87 double-rounded divide: RN53(RN64(a/b))
fn x87_div(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_div(&rx::ext_from_f64(a), &rx::ext_from_f64(b), CW),
        CW,
    )
}
// fully-extended pmt: (pv*r)/em all extended, single store. em given extended.
fn pmt_ext(pv: f64, r: f64, em_ext: &rx::Ext80) -> f64 {
    let numr = rx::ext_mul(&rx::ext_from_f64(pv), &rx::ext_from_f64(r), CW);
    rx::ext_to_f64(&rx::ext_div(&numr, em_ext, CW), CW)
}
// em as extended Ext80 (Kahan chain, but keep the final quotient extended)
fn em_kahan_ext(r: f64, n: f64) -> rx::Ext80 {
    let t = rx::x87_mul(-n, rx::excel_log1p(r));
    let u = rx::excel_exp(t);
    if u == 1.0 {
        return rx::ext_from_f64(t);
    }
    if t.abs() < 1.0 {
        let um1 = rx::ext_from_f64(u - 1.0);
        let te = rx::ext_from_f64(t);
        let lnu = rx::ext_from_f64(rx::excel_ln(u));
        // product stored to f64 (confirmed), divide kept extended
        let p = rx::ext_from_f64(rx::ext_to_f64(&rx::ext_mul(&um1, &te, CW), CW));
        rx::ext_div(&p, &lnu, CW)
    } else {
        rx::ext_from_f64(u - 1.0)
    }
}

fn load(path: &str) -> Vec<(f64, f64, f64, u64)> {
    let ws: WitnessSet =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("parse");
    let mut o = Vec::new();
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
        if a[3] != 0.0 || a[4] != 0.0 {
            continue;
        }
        if let Some(want) = parse_bits_hex(&w.expected_bits) {
            o.push((a[0], a[1], a[2], want.to_bits()));
        }
    }
    o
}

fn main() {
    let dir = "../../work/w109/G6-solvers/";
    let rows = load(&format!("{dir}answers-pmt-em.json"));
    // group by (rbits,n) -> {pv: pmt}
    let mut byrn: BTreeMap<(u64, i64), BTreeMap<u64, u64>> = BTreeMap::new();
    for (r, n, pv, want) in &rows {
        byrn.entry((r.to_bits(), *n as i64))
            .or_default()
            .insert(pv.to_bits(), *want);
    }
    // three pmt divide models, scored on BOTH-pv exact:
    //  SSE:  fl(pv*r/em)          [current oracle assumption]
    //  X87:  RN53(RN64(pv*r/em))  [x87 double-rounded divide, em f64]
    //  EXT:  RN53((pv*r)/em_ext)  [full x87 spill, em kept extended]
    for target_n in [1i64, 2, 3, 4, 6, 12] {
        let (mut both, mut sse, mut x87, mut ext) = (0u32, 0u32, 0u32, 0u32);
        for ((rb, n), pm) in byrn.iter() {
            if *n != target_n {
                continue;
            }
            let r = f64::from_bits(*rb);
            let (pv1, pv15) = (pm.get(&1.0f64.to_bits()), pm.get(&1.5f64.to_bits()));
            if pv1.is_none() || pv15.is_none() {
                continue;
            }
            both += 1;
            let em = em_kahan(r, *n as f64);
            let emx = em_kahan_ext(r, *n as f64);
            // SSE
            if (r / em).to_bits() == *pv1.unwrap() && (1.5 * r / em).to_bits() == *pv15.unwrap() {
                sse += 1;
            }
            // X87 double-rounded divide
            if x87_div(r, em).to_bits() == *pv1.unwrap()
                && x87_div(1.5 * r, em).to_bits() == *pv15.unwrap()
            {
                x87 += 1;
            }
            // full extended spill
            if pmt_ext(1.0, r, &emx).to_bits() == *pv1.unwrap()
                && pmt_ext(1.5, r, &emx).to_bits() == *pv15.unwrap()
            {
                ext += 1;
            }
        }
        println!(
            "n={:3}: both={:4}  BOTH-pv exact:  SSE-div={:4} ({:5.1}%)  X87-div={:4} ({:5.1}%)  EXT-spill={:4} ({:5.1}%)",
            target_n,
            both,
            sse,
            100.0 * sse as f64 / both as f64,
            x87,
            100.0 * x87 as f64 / both as f64,
            ext,
            100.0 * ext as f64 / both as f64,
        );
    }
}
