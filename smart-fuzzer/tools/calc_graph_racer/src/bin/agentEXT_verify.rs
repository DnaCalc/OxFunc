//! ANGLE(extended-precision): is Excel PMT expm1 (|tau|<1) an x87 EXTENDED
//! (80-bit, single final store) computation, or a double (x87-spill) Kahan?
//! Uses the REAL x87 microcode (F2XM1/FYL2X/FSCALE) so the ~1/2000 microcode
//! bits are faithful, not mpmath-CR. Reads expm1_intermediates.csv (Excel's
//! pinned tau,u,lnu,em doubles).
use oxfunc_core::excel_numeric::research as rx;
const CW: u16 = rx::CW_PC64_RN;
fn fb(h: &str) -> f64 { f64::from_bits(u64::from_str_radix(h.trim().trim_start_matches("0x"), 16).unwrap()) }
fn e(x: f64) -> rx::Ext80 { rx::ext_from_f64(x) }
fn tf(x: &rx::Ext80) -> f64 { rx::ext_to_f64(x, CW) }

struct Row { k: i32, n: i32, tau: f64, u: f64, lnu: f64, em: f64 }

fn load() -> Vec<Row> {
    let path = r"C:/Work/DnaCalc/OxFunc/smart-fuzzer/work/w109/G6-solvers/expm1_intermediates.csv";
    let txt = std::fs::read_to_string(path).unwrap();
    let mut out = Vec::new();
    for (i, line) in txt.lines().enumerate() {
        if i == 0 { continue; }
        let c: Vec<&str> = line.split(',').collect();
        if c.len() < 7 { continue; }
        out.push(Row { k: c[0].trim().parse().unwrap(), n: c[1].trim().parse().unwrap(),
            tau: fb(c[2]), u: fb(c[3]), lnu: fb(c[4]), em: fb(c[5]) });
    }
    out
}

/// Extended exp: u80 = 2^(x*log2e) via faithful fFEXP (F2XM1+FSCALE), kept Ext80.
fn exp_ext(x: f64) -> rx::Ext80 {
    let t = rx::ext_mul(&e(x), &rx::ext_l2e(), CW);         // x*log2e (ext)
    let k = rx::ext_rndint(&t, CW);                          // rint
    let f = rx::ext_sub(&t, &k, CW);                         // frac, |f|<=0.5
    let af = rx::ext_abs(&f, CW);
    let w = rx::ext_f2xm1(&af, CW);                          // 2^|f|-1
    let m = rx::ext_add(&w, &rx::ext_one(), CW);             // 2^|f|
    // sign of f: reconstruct via compare to zero using f64 view
    let f_neg = tf(&f) < 0.0;
    let m = if f_neg { rx::ext_div(&rx::ext_one(), &m, CW) } else { m };
    rx::ext_scale(&m, &k, CW)                                // *2^k
}
/// Extended tau80 = -n * fyl2xp1(ln2, r)  (r=2^k exact).
fn tau_ext(k: i32, n: i32) -> rx::Ext80 {
    let r = e(2f64.powi(k));
    let lp = rx::ext_fyl2xp1(&rx::ext_ln2(), &r, CW);        // ln(1+r) ext
    rx::ext_mul(&e(-(n as f64)), &lp, CW)
}
/// Extended ln(u80) = fyl2x(ln2, u80).
fn ln_ext(u: &rx::Ext80) -> rx::Ext80 { rx::ext_fyl2x(&rx::ext_ln2(), u, CW) }

fn main() {
    let rows = load();
    let n = rows.len();
    // candidates
    let mut cnt = std::collections::BTreeMap::<&str, u32>::new();
    let bump = |name: &'static str, got: f64, em: f64, m: &mut std::collections::BTreeMap<&str,u32>| {
        if got.to_bits() == em.to_bits() { *m.entry(name).or_insert(0) += 1; } else { m.entry(name).or_insert(0); }
    };
    for r in &rows {
        let (tau, u, lnu, em) = (r.tau, r.u, r.lnu, r.em);
        let b = u - 1.0;
        // -- doubles --
        bump("A1 dbl (u-1)*tau/lnu", (b*tau)/lnu, em, &mut cnt);
        bump("M1 x87mul(b,tau)/lnu", rx::x87_mul(b,tau)/lnu, em, &mut cnt);
        bump("M1x x87mul then x87div", tf(&rx::ext_div(&e(rx::x87_mul(b,tau)), &e(lnu), CW)), em, &mut cnt);
        // production internal
        bump("PROD excel_expm1_internal(tau)", rx::excel_expm1_internal(tau), em, &mut cnt);
        // -- real-microcode extended --
        let u80 = exp_ext(tau);                       // extended exp of DOUBLE tau
        let taue = tau_ext(r.k, r.n);                 // extended tau from scratch
        let one = rx::ext_one();
        // V1: fully extended, tau80 from scratch, u80'=exp_ext(tf(taue))? use taue directly
        let u80b = exp_ext(tf(&taue));
        let em_v1 = {
            let bnum = rx::ext_sub(&u80b, &one, CW);
            let num = rx::ext_mul(&bnum, &taue, CW);
            let den = ln_ext(&u80b);
            tf(&rx::ext_div(&num, &den, CW))
        };
        bump("V1 full-ext (tau80,u80,lnu80)", em_v1, em, &mut cnt);
        // V3: tau stored to double, expm1 internals extended (u80 from double tau)
        let em_v3 = {
            let bnum = rx::ext_sub(&u80, &one, CW);
            let num = rx::ext_mul(&bnum, &e(tau), CW);
            let den = ln_ext(&u80);
            tf(&rx::ext_div(&num, &den, CW))
        };
        bump("V3 ext-internals, tau=dbl", em_v3, em, &mut cnt);
        // V3b: like V3 but denominator = double lnu (measured), numerator ext
        let em_v3b = {
            let bnum = rx::ext_sub(&u80, &one, CW);
            let num = rx::ext_mul(&bnum, &e(tau), CW);
            tf(&rx::ext_div(&num, &e(lnu), CW))
        };
        bump("V3b ext-num, den=lnu_dbl", em_v3b, em, &mut cnt);
        // V2: extended 'expm1' via direct F2XM1 when |tau*l2e|<=1 (else fFEXP-1)
        let em_v2 = {
            let t = rx::ext_mul(&e(tau), &rx::ext_l2e(), CW);
            if tf(&rx::ext_abs(&t, CW)) <= 1.0 {
                tf(&rx::ext_f2xm1(&t, CW))            // 2^(tau*l2e)-1 = e^tau-1, ONE instr
            } else {
                tf(&rx::ext_sub(&exp_ext(tau), &one, CW))
            }
        };
        bump("V2 direct-F2XM1 expm1", em_v2, em, &mut cnt);
    }
    // consistency checks vs captured intermediates
    let (mut u_ok, mut lnu_ok) = (0u32, 0u32);
    for r in &rows {
        if rx::excel_exp(r.tau).to_bits() == r.u.to_bits() { u_ok += 1; }
        if rx::excel_ln(r.u).to_bits() == r.lnu.to_bits() { lnu_ok += 1; }
    }
    println!("rows={n}");
    println!("consistency: excel_exp(tau)==u {u_ok}/{n}   excel_ln(u)==lnu {lnu_ok}/{n}");
    println!("--- candidate em match counts (REAL x87 microcode) ---");
    let mut v: Vec<_> = cnt.into_iter().collect();
    v.sort_by_key(|x| std::cmp::Reverse(x.1));
    for (name, c) in v { println!("  {:34} {:3}/{}", name, c, n); }
}
