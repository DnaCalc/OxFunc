//! W109 G6-01: the MIXED-PROVENANCE linkage the inverse solve points to. Numerator uses the
//! SPILLED double u (a=u_dbl-1, num=a*tau_dbl — everything proven); the denominator computes
//! lnu = FYL2X(ln2, u_EXT) of the UN-spilled 80-bit exp result, then spills THAT to double.
//! FYL2X(u_ext) vs FYL2X(u_dbl) differ by the ~11 hidden mantissa bits of u, log-amplified to
//! the +1 double-ULP denominator the inverse solve demands — invisible to worksheet LN(u_dbl).
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x,
    ext_from_f64 as ef, ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};
const CW: u16 = CW_PC64_RN;
const RN53: u16 = CW_PC53_RN;

fn b(s: &str) -> f64 { f64::from_bits(u64::from_str_radix(s, 16).unwrap()) }

fn exp_ext(tau: &Ext80, l2e: &Ext80) -> Ext80 {
    let y = ext_mul(tau, l2e, CW);
    let kk = ext_rndint(&y, CW);
    let f = ext_sub(&y, &kk, CW);
    let w = ext_f2xm1(&f, CW);
    ext_scale(&ext_add(&w, &ext_one(), CW), &kk, CW)
}

fn main() {
    let csv = std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let ln2 = ext_ln2();
    let l2e = ext_l2e();

    let names = [
        "0 baseline num_dbl / lnu(captured)        ",
        "1 num_dbl / RN53(FYL2X(u_EXT))            ",
        "2 num_dbl / FYL2X(u_EXT) EXT, x87 /PC64   ",
        "3 num_dbl / RN53(FYL2X(u_EXT)), only uext==cap rows",
    ];
    let mut score = [0u32; 4];
    let mut score_miss = [0u32; 4];
    let (mut tot, mut nmiss) = (0u32, 0u32);
    let mut den_diff_dir: std::collections::HashMap<i64,u32> = std::collections::HashMap::new();
    let mut uext_ok = 0u32;
    let mut fix1: Vec<(i32,u32)> = Vec::new();
    let mut break1 = 0u32;

    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 { continue; }
        let k: i32 = f[0].parse().unwrap();
        let n: u32 = f[1].parse().unwrap();
        let tau = b(f[2]); let u = b(f[3]); let lnu = b(f[4]); let em = b(f[5]);
        let emb = em.to_bits();
        tot += 1;
        let r = 2f64.powi(k);

        // extended u from the F2XM1 chain, consistent with captured tau (use captured tau_dbl widened)
        let tau_ext = ef(tau); // start from Excel's exact tau double (isolate the u-extension effect)
        // BUT to get u's hidden bits we need u at extended: recompute exp(tau) fully extended.
        // Using tau_ext = widened double tau underweights hidden bits of tau; that's intended —
        // we isolate the u-extension (exp's own rounding kept in-register).
        let u_ext = exp_ext(&tau_ext, &l2e);
        let u_resp = ext_to_f64(&u_ext, RN53);
        let uok = u_resp.to_bits() == u.to_bits();
        if uok { uext_ok += 1; }

        let a = u - 1.0;
        let num_dbl = a * tau;
        let num_e = ef(num_dbl);

        let lnu_ext = ext_fyl2x(&ln2, &u_ext, CW);
        let lnu_new = ext_to_f64(&lnu_ext, RN53);
        let dd = lnu_new.to_bits() as i64 - lnu.to_bits() as i64;
        *den_diff_dir.entry(dd).or_insert(0) += 1;

        let c1 = (num_dbl / lnu_new).to_bits();
        let c2 = ext_to_f64(&ext_div(&num_e, &lnu_ext, CW), RN53).to_bits();
        let c3 = if uok { c1 } else { u64::MAX };
        let c0 = (num_dbl / lnu).to_bits();

        let cands = [c0, c1, c2, c3];
        let base_miss = c0 != emb;
        if base_miss { nmiss += 1; }
        for i in 0..4 { if cands[i]==emb { score[i]+=1; if base_miss { score_miss[i]+=1; } } }
        if base_miss { if c1==emb { fix1.push((k,n)); } } else if c1!=emb { break1+=1; }
    }

    println!("N={} misses={} | u_ext spills==captured u on {}/{}", tot, nmiss, uext_ok, tot);
    let mut dds: Vec<_> = den_diff_dir.iter().collect(); dds.sort();
    print!("denominator(new-captured) ulp-diff distribution: ");
    for (d,c) in dds { print!("{}:{} ", d, c); } println!();
    println!("{:<50} {:>12} {:>10}", "candidate", "all", "on-miss");
    for i in 0..4 {
        println!("{:<50} {:>4}/{} ({:4.1}%) {:>4}/{}", names[i], score[i], tot,
                 100.0*score[i] as f64/tot as f64, score_miss[i], nmiss);
    }
    println!("\nuext-denom(1) FIXES {} miss rows, BREAKS {} hit rows", fix1.len(), break1);
    print!("fixed: "); for x in fix1.iter().take(24) { print!("({},{}) ", x.0, x.1); } println!();
}
