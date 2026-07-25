//! W109 G6-01: is the GENERAL-RATE wall a log1p identity issue? Production uses
//! a PORTABLE CR log1p; Excel may use x87 FYL2XP1 (differs from CR on non-2^-k
//! rates). The |tau|>=1 combine is correct (arrangements tie), so upstream em is
//! suspect. Test log1p variants x n-product spill on general-rate corpora.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC53_RN, CW_PC64_RN, Ext80, ext_from_f64, ext_fyl2xp1, ext_ln2, ext_mul, ext_to_f64};

const RN53: u16 = CW_PC53_RN;
const CW: u16 = CW_PC64_RN;

// log1p(r) via x87 FYL2XP1 = ln2*log2(1+r), spilled to double
fn log1p_x87(r: f64) -> f64 { ext_to_f64(&ext_fyl2xp1(&ext_ln2(), &ext_from_f64(r), CW), RN53) }
// nl = -n*log1p(r), fully extended (fyl2xp1 kept in reg, * -n at PC64), spilled once
fn nl_x87_ext(n: f64, r: f64) -> f64 {
    let l = ext_fyl2xp1(&ext_ln2(), &ext_from_f64(r), CW);
    ext_to_f64(&ext_mul(&ext_from_f64(-n), &l, CW), RN53)
}

fn em_kahan(nl: f64) -> f64 { rx::excel_expm1_internal(nl) }

fn pmt_a1(r: f64, pv: f64, fv: f64, ty: f64, em: f64) -> f64 {
    if em == 0.0 { return f64::NAN; }
    let v = 1.0 + em; let tf = 1.0 + r * ty; ((pv + fv * v) / em / tf) * r
}

fn main() {
    let corpora = ["genrate", "heldout", "fvty", "fv1sweep", "po2", "combsweep"];
    let names = ["L0 portable-CR [landed]", "L1 x87 FYL2XP1 spill", "L2 x87 nl fully-ext"];
    print!("{:11}{:>7}", "corpus", "N");
    for nm in &names { print!(" {:>24}", nm); }
    println!();
    for cn in corpora {
        let p = format!("../../work/w109/G6-solvers/answers-pmt-{}.json", cn);
        if !std::path::Path::new(&p).exists() { continue; }
        let ws: WitnessSet = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let mut sc = [0u32; 3]; let mut tot = 0u32;
        for w in &ws.witnesses {
            let a: Vec<f64> = w.args.iter().filter_map(|x| match x { WitnessArg::Scalar(s) => parse_bits_hex(s), _ => None }).collect();
            if a.len() != 5 { continue; }
            let (r, n, pv, fv, ty) = (a[0], a[1], a[2], a[3], a[4]);
            let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits(); tot += 1;
            let nls = [
                -(n * rx::excel_log1p(r)),   // L0 portable CR
                -(n * log1p_x87(r)),          // L1 x87 fyl2xp1 spilled then *n
                nl_x87_ext(n, r),             // L2 fully extended nl
            ];
            for i in 0..3 {
                let em = em_kahan(nls[i]);
                if pmt_a1(r, pv, fv, ty, em).to_bits() == want { sc[i] += 1; }
            }
        }
        print!("{:11}{:>7}", cn, tot);
        for i in 0..3 { print!(" {:>23.1}%", 100.0 * sc[i] as f64 / tot as f64); }
        println!();
    }
}
