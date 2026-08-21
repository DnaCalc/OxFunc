//! W109 G6-01 (Fable H2): bespoke fFEXPM1 that reuses the fFEXP reduction's
//! w=F2XM1(f) and assembles em directly, exposing F2XM1's extended tail (hidden
//! in u~1). For tau<0, K=0: em = -(w*m), m=1/(1+w). Test with per-op SPILL
//! (fl53(fl64(.))) vs extended. Score on the K=0 subset (214) and overall.
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_l2e,
    ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};

const RN53: u16 = CW_PC53_RN;
const CW: u16 = CW_PC64_RN;
fn sp(v: &Ext80) -> f64 {
    ext_to_f64(v, RN53)
} // spill extended -> double (fl53)

// returns (K, w_ext, onepw_ext, neg)
fn reduce(tau: f64) -> (Ext80, Ext80, Ext80, bool) {
    let t = ext_mul(&ext_from_f64(tau), &ext_l2e(), CW);
    let k = ext_rndint(&t, CW);
    let f = ext_sub(&t, &k, CW);
    let neg = ext_to_f64(&f, CW) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, CW), CW); // 2^|f| - 1
    let onepw = ext_add(&w, &ext_one(), CW); // 2^|f|
    (k, w, onepw, neg)
}

fn main() {
    let csv =
        std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let labels = [
        "A -(w_sp * m_sp)      [dbl ops]",
        "B -(w_ext * m_ext) spill final",
        "C -(w_sp / onepw_sp)  [dbl div]",
        "D -(w_ext / onepw_ext) spill",
        "E -(w_ext * m_ext_reg) reg->store",
        "F general 2^K*m-1 via pieces spill",
        "G -(w_sp * m_ext) mixed",
        "H -(w_ext*m_ext) all-ext(one store)",
    ];
    let mut score = [0u32; 8];
    let mut score_k0 = [0u32; 8];
    let mut tot = 0u32;
    let mut tot_k0 = 0u32;
    for line in csv.lines().skip(1) {
        let ff: Vec<&str> = line.split(',').collect();
        if ff.len() < 6 {
            continue;
        }
        let tau = f64::from_bits(u64::from_str_radix(ff[2], 16).unwrap());
        let pin = u64::from_str_radix(ff[5], 16).unwrap();
        let (k, w, onepw, neg) = reduce(tau);
        let kval = ext_to_f64(&k, CW);
        let is_k0 = kval == 0.0 && neg; // pure reciprocal-branch, no scale
        // m = 1/(1+w) extended and spilled
        let m_ext = ext_div(&ext_one(), &onepw, CW);
        let m_sp = sp(&m_ext);
        let w_sp = sp(&w);
        let onepw_sp = sp(&onepw);

        // A: both operands double, product double-rounded (spill)
        let a =
            (-(ext_to_f64(&ext_mul(&ext_from_f64(w_sp), &ext_from_f64(m_sp), CW), RN53))).to_bits();
        // B: extended operands, product spilled
        let b = (-(ext_to_f64(&ext_mul(&w, &m_ext, CW), RN53))).to_bits();
        // C: double divide w/(1+w), spilled
        let c = (-(ext_to_f64(
            &ext_div(&ext_from_f64(w_sp), &ext_from_f64(onepw_sp), CW),
            RN53,
        )))
        .to_bits();
        // D: extended divide, spilled
        let d = (-(ext_to_f64(&ext_div(&w, &onepw, CW), RN53))).to_bits();
        // E: same as B (kept for symmetry with a reg variant) -> use w spilled, m extended
        let e = (-(ext_to_f64(&ext_mul(&ext_from_f64(w_sp), &m_ext, CW), RN53))).to_bits();
        // F: general em = 2^K*m - 1 assembled = (2^K-1) + 2^K*(m-1); m-1 = -w*m
        let two_k = ext_scale(&ext_one(), &k, CW);
        let two_k_m1 = ext_sub(&two_k, &ext_one(), CW);
        let mm1 = ext_sub(&m_ext, &ext_one(), CW); // = -w*m
        let f_em = ext_to_f64(&ext_add(&two_k_m1, &ext_mul(&two_k, &mm1, CW), CW), RN53).to_bits();
        // G already covered by E; reuse
        let g = e;
        // H: all extended, one final store (my earlier 94)
        let h = ext_to_f64(&mm1, RN53).to_bits();

        let cands = [a, b, c, d, e, f_em, g, h];
        for i in 0..8 {
            if cands[i] == pin {
                score[i] += 1;
                if is_k0 {
                    score_k0[i] += 1;
                }
            }
        }
        if is_k0 {
            tot_k0 += 1;
        }
        tot += 1;
    }
    println!(
        "=== fFEXPM1 assembly (Fable H2), N={}  (K0 neg-branch subset={}) ===",
        tot, tot_k0
    );
    for i in 0..8 {
        println!(
            "  {:34} all {:3}/{} ({:4.1}%)   K0 {:3}/{} ({:4.1}%)",
            labels[i],
            score[i],
            tot,
            100.0 * score[i] as f64 / tot as f64,
            score_k0[i],
            tot_k0,
            100.0 * score_k0[i] as f64 / tot_k0 as f64
        );
    }
}
