//! W109 G6-01: end-to-end PMT race, positive-power classic forms vs the landed
//! expm1 form, against the REAL PMT oracles (no em-pinning confound).
//! args = [r, n, pv, fv, ty]. Excel PMT (fv=0,ty=0) = -pv*P*r/(P-1), P=(1+r)^n.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{CW_PC53_RN, CW_PC64_RN, Ext80, ext_from_f64, ext_mul, ext_one, ext_to_f64};

// (1+r)^n, integer n, SSE2 double exponentiation-by-squaring
fn powi_d(opr: f64, mut e: u32) -> f64 {
    let mut res = 1.0; let mut b = opr;
    while e > 0 { if e & 1 == 1 { res *= b; } e >>= 1; if e > 0 { b *= b; } }
    res
}
// (1+r)^n, integer n, x87 ext binexp, spilled to double at the end
fn powi_ext(opr: f64, mut e: u32) -> f64 {
    let cw = CW_PC64_RN; let mut res = ext_one(); let mut b = ext_from_f64(opr);
    while e > 0 { if e & 1 == 1 { res = ext_mul(&res, &b, cw); } e >>= 1; if e > 0 { b = ext_mul(&b, &b, cw); } }
    ext_to_f64(&res, CW_PC53_RN)
}
// P = (1+r)^n choosing substrate; non-integer n -> excel_pow_positive (exp*log)
fn bigp(r: f64, n: f64, mode: u8) -> f64 {
    let opr = 1.0 + r;
    if n.fract() == 0.0 && n >= 0.0 && n <= 1e9 {
        let e = n as u32;
        match mode { 0 => powi_d(opr, e), 1 => powi_ext(opr, e), _ => rx::excel_pow_positive(opr, n) }
    } else {
        rx::excel_pow_positive(opr, n)
    }
}

// ---- landed expm1 form (baseline) ----
fn v_landed(r: f64, n: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let nl = -(n * rx::excel_log1p(r));
    let em = rx::excel_expm1_internal(nl);
    if em == 0.0 { return f64::NAN; }
    let v = 1.0 + em; let tf = 1.0 + r * ty; let num = pv + fv * v;
    ((num / em) / tf) * r
}
// ---- classic positive-power, Welinder fvifa arrangement ----
// pmt = -(pv*P+fv)/((1+r*ty)*((P-1)/r)),  P substrate = mode
fn v_classic(r: f64, n: f64, pv: f64, fv: f64, ty: f64, mode: u8) -> f64 {
    let p = bigp(r, n, mode);
    let fvifa = (p - 1.0) / r;
    let denom = (1.0 + r * ty) * fvifa;
    -(pv * p + fv) / denom
}
// ---- classic, *r-last arrangement: pmt = -(pv*P+fv)*r/((1+r*ty)*(P-1)) ----
fn v_classic_rlast(r: f64, n: f64, pv: f64, fv: f64, ty: f64, mode: u8) -> f64 {
    let p = bigp(r, n, mode);
    let denom = (1.0 + r * ty) * (p - 1.0);
    -(pv * p + fv) * r / denom
}
// ---- reciprocal em from positive P, landed combine skeleton ----
// v=1/P, em=v-1, pmt=((pv+fv*v)/em/tf)*r
fn v_recip_em(r: f64, n: f64, pv: f64, fv: f64, ty: f64, mode: u8) -> f64 {
    let p = bigp(r, n, mode);
    let v = 1.0 / p; let em = v - 1.0;
    if em == 0.0 { return f64::NAN; }
    let tf = 1.0 + r * ty; let num = pv + fv * v;
    ((num / em) / tf) * r
}
// ---- classic quotient-first, divide by tf: pmt = -(pv*P+fv)/fvifa/tf ----
fn v_classic_qf(r: f64, n: f64, pv: f64, fv: f64, ty: f64, mode: u8) -> f64 {
    let p = bigp(r, n, mode);
    let fvifa = (p - 1.0) / r;
    let tf = 1.0 + r * ty;
    (-(pv * p + fv) / fvifa) / tf
}

fn main() {
    let corpora = ["po2", "heldout", "combsweep", "genrate", "fvty", "fv1sweep", "po2n"];
    // (label, fn)
    type F = Box<dyn Fn(f64, f64, f64, f64, f64) -> f64>;
    let mut cands: Vec<(String, F)> = Vec::new();
    cands.push(("landed expm1".into(), Box::new(v_landed)));
    for (mn, ml) in [(0u8, "Pd"), (1u8, "Pext"), (2u8, "Pexplog")] {
        cands.push((format!("classic/{}", ml), Box::new(move |r, n, pv, fv, ty| v_classic(r, n, pv, fv, ty, mn))));
        cands.push((format!("classic_rlast/{}", ml), Box::new(move |r, n, pv, fv, ty| v_classic_rlast(r, n, pv, fv, ty, mn))));
        cands.push((format!("recip_em/{}", ml), Box::new(move |r, n, pv, fv, ty| v_recip_em(r, n, pv, fv, ty, mn))));
        cands.push((format!("classic_qf/{}", ml), Box::new(move |r, n, pv, fv, ty| v_classic_qf(r, n, pv, fv, ty, mn))));
    }
    // header
    print!("{:16}", "corpus/N");
    for (name, _) in &cands { print!(" {:>16}", name); }
    println!();
    for cn in corpora {
        let p = format!("../../work/w109/G6-solvers/answers-pmt-{}.json", cn);
        if !std::path::Path::new(&p).exists() { continue; }
        let ws: WitnessSet = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let mut sc = vec![0u32; cands.len()];
        let mut tot = 0u32;
        for w in &ws.witnesses {
            let a: Vec<f64> = w.args.iter().filter_map(|x| match x { WitnessArg::Scalar(s) => parse_bits_hex(s), _ => None }).collect();
            if a.len() != 5 { continue; }
            let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
            tot += 1;
            for (i, (_, f)) in cands.iter().enumerate() {
                if f(a[0], a[1], a[2], a[3], a[4]).to_bits() == want { sc[i] += 1; }
            }
        }
        print!("{:10}{:>6}", cn, tot);
        for i in 0..cands.len() { print!(" {:>15.0}%", 100.0 * sc[i] as f64 / tot as f64); }
        println!();
    }
}
