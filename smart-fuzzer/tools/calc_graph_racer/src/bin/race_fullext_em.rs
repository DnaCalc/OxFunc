//! W109 G6-01: the FULLY-EXTENDED hand-written x87 routine hypothesis. Instead of spilling each
//! op to double, the routine keeps EVERYTHING at 80-bit from (r,n) and stores to double ONCE at
//! the end: l1p=FYL2XP1(ln2,r); tau=-n*l1p; u=exp(tau) via F2XM1 reduction; a=u-1 (=w when K=0);
//! lnu=FYL2X(ln2,u); em = RN53( (a*tau)/lnu ), all intermediates 80-bit. Real hardware
//! F2XM1/FYL2X/FYL2XP1. Sanity-checks ext-tau/ext-u against Excel's captures, then scores em.
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x,
    ext_fyl2xp1, ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};
const CW: u16 = CW_PC64_RN;
const RN53: u16 = CW_PC53_RN;

fn b(s: &str) -> f64 { f64::from_bits(u64::from_str_radix(s, 16).unwrap()) }

// exp(tau) at 80-bit via the fFEXP F2XM1 reduction. Returns (u_ext, w_ext, K) where u=scale(1+w,K).
fn exp_ext(tau: &Ext80, l2e: &Ext80) -> (Ext80, Ext80, Ext80) {
    let y = ext_mul(tau, l2e, CW);        // tau*log2(e)
    let kk = ext_rndint(&y, CW);          // round-nearest integer
    let f = ext_sub(&y, &kk, CW);         // |f| <= 0.5
    let w = ext_f2xm1(&f, CW);            // 2^f - 1
    let onepw = ext_add(&w, &ext_one(), CW);
    let u = ext_scale(&onepw, &kk, CW);   // 2^K*(1+w)
    (u, w, kk)
}

fn main() {
    let csv = std::fs::read_to_string("../../work/w109/G6-solvers/expm1_intermediates.csv").unwrap();
    let ln2 = ext_ln2();
    let l2e = ext_l2e();
    let one = ext_one();

    let names = [
        "0 baseline captured-double (a*tau/lnu)   ",
        "1 FULL-EXT chain, a=u-1, store once      ",
        "2 FULL-EXT chain, a=w (K=0 direct)       ",
        "3 FULL-EXT but lnu=captured-dbl          ",
        "4 FULL-EXT, num&den ext, final /RN53 store",
    ];
    let mut score = [0u32; 5];
    let mut score_miss = [0u32; 5];
    let (mut tot, mut nmiss) = (0u32, 0u32);
    let (mut tau_ok, mut u_ok) = (0u32, 0u32);
    let mut fix1: Vec<(i32,u32)> = Vec::new();
    let mut break1 = 0u32;

    for line in csv.lines().skip(1) {
        let f: Vec<&str> = line.split(',').collect();
        if f.len() < 6 { continue; }
        let k: i32 = f[0].parse().unwrap();
        let n: u32 = f[1].parse().unwrap();
        let tau_cap = b(f[2]); let u_cap = b(f[3]); let lnu_cap = b(f[4]); let em = b(f[5]);
        let emb = em.to_bits();
        tot += 1;
        let r = 2f64.powi(k);

        // fully-extended chain from (r,n)
        let l1p = ext_fyl2xp1(&ln2, &ext_from_f64(r), CW);      // ln(1+r) 80-bit
        let tau_ext = ext_mul(&ext_from_f64(-(n as f64)), &l1p, CW); // -n*ln(1+r)
        if ext_to_f64(&tau_ext, RN53).to_bits() == tau_cap.to_bits() { tau_ok += 1; }
        let (u_ext, w_ext, _kk) = exp_ext(&tau_ext, &l2e);
        if ext_to_f64(&u_ext, RN53).to_bits() == u_cap.to_bits() { u_ok += 1; }

        let a_ext = ext_sub(&u_ext, &one, CW);
        let lnu_ext = ext_fyl2x(&ln2, &u_ext, CW);
        let num1 = ext_mul(&a_ext, &tau_ext, CW);
        let num2 = ext_mul(&w_ext, &tau_ext, CW);

        // c1: full-ext, a=u-1
        let c1 = ext_to_f64(&ext_div(&num1, &lnu_ext, CW), RN53).to_bits();
        // c2: full-ext, a=w
        let c2 = ext_to_f64(&ext_div(&num2, &lnu_ext, CW), RN53).to_bits();
        // c3: full-ext numerator but denominator = captured double lnu
        let c3 = ext_to_f64(&ext_div(&num1, &ext_from_f64(lnu_cap), CW), RN53).to_bits();
        // c4: same as c1 (num&den ext) — kept for clarity
        let c4 = c1;
        let c0 = ((u_cap - 1.0) * tau_cap / lnu_cap).to_bits();

        let cands = [c0, c1, c2, c3, c4];
        let base_miss = c0 != emb;
        if base_miss { nmiss += 1; }
        for i in 0..5 { if cands[i]==emb { score[i]+=1; if base_miss { score_miss[i]+=1; } } }
        if base_miss { if c1==emb { fix1.push((k,n)); } } else if c1!=emb { break1+=1; }
    }

    println!("N={} misses={} | ext-tau==cap: {}/{}  ext-u==cap: {}/{}", tot, nmiss, tau_ok, tot, u_ok, tot);
    println!("{:<44} {:>12} {:>10}", "candidate", "all", "on-miss");
    for i in 0..5 {
        println!("{:<44} {:>4}/{} ({:4.1}%) {:>4}/{}", names[i], score[i], tot,
                 100.0*score[i] as f64/tot as f64, score_miss[i], nmiss);
    }
    println!("\nfull-ext(1) FIXES {} miss rows, BREAKS {} hit rows", fix1.len(), break1);
    print!("fixed: "); for x in fix1.iter().take(24) { print!("({},{}) ", x.0, x.1); } println!();
}
