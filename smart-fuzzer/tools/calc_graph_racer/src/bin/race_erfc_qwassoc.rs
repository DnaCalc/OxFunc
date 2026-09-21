//! excel_exp(-z*z) vs -RN53(z*z) on w-only miss-1. Cody Q on b11c. Not an ID.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    excel_exp, excel_exp_rz, ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN,
};
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const C: [f64; 9] = [
    0.564188496988670089,
    8.88314979438837594,
    66.1191906371416295,
    298.635138197400131,
    881.95222124176909,
    1712.04761263407058,
    2051.07837782607147,
    1230.33935479799725,
    2.15311535474403846e-8,
];
const D: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn cody_cd(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(C[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW),
        CW,
    )
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut w_only = Vec::new();
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let ff = cody_cd(r.z);
        let w = f::w_rn53(r.z);
        if w == 0.0 || ff == 0.0 {
            continue;
        }
        let qg = w * ff;
        let qo = f64::from_bits(r.qbits);
        if ulp_distance(qg, qo).unwrap_or(99) != 1 {
            continue;
        }
        let df = ulp_distance(ff, qo / w).unwrap_or(99);
        let dw = ulp_distance(w, qo / ff).unwrap_or(99);
        if dw <= 1 && df > 1 {
            w_only.push((r.z, r.qbits, ff, w));
        }
    }
    println!("w-only n={}", w_only.len());
    let variants: [(&str, fn(f64) -> f64); 5] = [
        ("exp(-RN53(z*z))", |z| excel_exp(-(z * z))),
        ("w_rn53 identified", f::w_rn53),
        ("exp(-(z*z) no negstore)", |z| excel_exp(-(z * z))),
        ("exp(x87 -z*z)", |z| {
            let zz = ext_to_f64(&ext_mul(&ef(z), &ef(z), CW), CW);
            excel_exp(-zz)
        }),
        ("exp_rz(-RN53(z*z))", |z| excel_exp_rz(-(z * z))),
    ];
    // first two are same if z*z already exact... w_rn53 is exp(-RN53(z*z))
    for (name, wf) in [
        ("w_rn53", f::w_rn53 as fn(f64) -> f64),
        ("exp(-z*z)", |z| excel_exp(-(z * z))),
        ("exp(-RN53(z*z))", |z| excel_exp(-(z * z))),
        ("exp(x87 -z*z)", |z| {
            let zz = ext_to_f64(&ext_mul(&ef(z), &ef(z), CW), CW);
            excel_exp(-zz)
        }),
        ("exp_rz(-z*z)", |z| excel_exp_rz(-(z * z))),
        ("libm exp(-z*z)", |z| libm::exp(-(z * z))),
    ] {
        let mut hit = 0usize;
        for &(z, qb, ff, _) in &w_only {
            let w = wf(z);
            let qg = w * ff;
            if ulp_distance(qg, f64::from_bits(qb)).unwrap_or(99) == 0 {
                hit += 1;
            }
        }
        println!("{name:22} Q-exact on w-only {hit}/{}", w_only.len());
    }
    let _ = variants;

    let path = format!("{dir}/answers-b11c.json");
    let bank: WitnessSet = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let mut ex = 0usize;
    let mut n = 0usize;
    let mut max = 0u64;
    for w in &bank.witnesses {
        let x = match &w.args[0] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => continue,
        };
        let z = x.abs();
        if z < 0.5 || z >= 4.0 {
            continue;
        }
        let Some(e) = parse_bits_hex(&w.expected_bits) else {
            continue;
        };
        let qg = f::w_rn53(z) * cody_cd(z);
        let d = ulp_distance(qg, e).unwrap_or(u64::MAX);
        n += 1;
        if d == 0 {
            ex += 1;
        } else {
            max = max.max(d);
        }
    }
    println!("b11c ERFC z in[0.5,4) Cody Q {ex}/{n} max={max}");
}
