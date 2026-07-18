//! W109 excel-primitive op server: line-oriented request/response so Python
//! candidate-graph explorers can call the REAL hardware chains per op.
//!
//! Request per stdin line:  "<op> <hex> [<hex>]"
//!   exp x      — excel_exp (fFEXP chain, RN53 publish)
//!   expz x     — excel_exp_rz (chop publish, series r-site)
//!   ln x       — excel_ln (fyl2x chain)
//!   expm1 x    — excel_expm1_internal (Kahan)
//!   mul a b    — x87 double-rounded product RN53(RN64(a*b))
//!   div a b    — x87 double-rounded quotient RN53(RN64(a/b))
//!   recip x    — x87 double-rounded reciprocal
//!   cexpext a b — chain-exp of the EXTENDED product a*b (PC=64, no spill
//!                 between the multiply and the fFEXP chain entry)
//!   lnext x     — the EXTENDED fyl2x ln result, printed as "hi lo" (two
//!                 hex doubles whose exact sum is the 64-bit value)
//!   lp1 x       — hardware log1p via FYL2XP1: RN53( fyl2xp1(ln2, x) )
//!   lp1ext x    — same, EXTENDED result printed as "hi lo"
//!   bd0dir x np mask — direct-branch bd0 = x*L + (np - x) variants:
//!                 bit0 quotient q=x/np extended (RN64, unspilled; else
//!                      plain RN53 double)
//!                 bit1 L = fyl2x(q) consumed EXTENDED (else spilled RN53)
//!                 bit2 whole return compound extended with ONE final RN53
//!                      (else per-op RN53 as the pinned model)
//!                 bit3 association A: (x*L + np) - x (else B: x*L+(np-x))
//!   cexpext2 hi lo — chain-exp of the EXTENDED argument hi+lo (two doubles
//!                 reconstructing a 64-bit-mantissa value exactly via one
//!                 RN64 add; chain entered extended, RN53 publish)
//!   mulex a b   — RN53( chainexp_ext(a) * b ): the fFEXP chain run on double
//!                 argument a with its EXTENDED result multiplied by double b
//!                 (unspilled return composing on the stack), single store
//!   mulee a b   — RN53( chainexp_ext(a) * chainexp_ext(b) ): both chain
//!                 results extended, product at PC=64, single store
//!   pmfk k lam mask — POISSON pmf candidate graph pow(lam,k)*exp(-lam)/k!
//!                 with spill/rounding mask (k integer 1..=3):
//!                   bit0 ln spilled to RN53   bit1 t=k*ln spilled
//!                   bit2 pow spilled          bit3 e spilled
//!                   bit4 pre-division product spilled
//!                   bit5 final store RZ (else RN)
//!                   bit6 pow by REPEATED FMUL of lam (extended; bits 0-1
//!                        ignored; bit2 still spills the pow result)
//!   dbin k n p mask — BINOM pmf log-composed EXTENDED graph:
//!                 exp_chain( lnC + k*ln p + (n-k)*ln q ) with fyl2x
//!                 extended lns, RN64 products/sums, chain on the extended
//!                 argument, RN53 publish.  mask spills to RN53:
//!                   bit0 lnC  bit1 lnp  bit2 lnq  bit3 t1=k*lnp
//!                   bit4 t2=(n-k)*lnq  bit5 sum (=> double-arg chain)
//!                   bit6 sum association: 0 (lnC+t1)+t2, 1 lnC+(t1+t2)
//!                   bit7 final store RZ
//! Response: one hex64 per line, same order.  Unknown op -> "ERR".

use oxfunc_core::excel_numeric::research as rx;
use std::io::{BufRead, Write};

/// fFEXP chain on an EXTENDED argument, result left EXTENDED.
fn exp_chain_from_ext(xe: &rx::Ext80) -> rx::Ext80 {
    use rx::{
        CW_PC64_RN, ext_abs, ext_add, ext_div, ext_f2xm1, ext_l2e, ext_mul, ext_one, ext_rndint,
        ext_scale, ext_sub, ext_to_f64,
    };
    let cw = CW_PC64_RN;
    let t = ext_mul(xe, &ext_l2e(), cw);
    let k = ext_rndint(&t, cw);
    let f = ext_sub(&t, &k, cw);
    let neg = ext_to_f64(&f, cw) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, cw), cw);
    let mut m = ext_add(&w, &ext_one(), cw);
    if neg {
        m = ext_div(&ext_one(), &m, cw);
    }
    ext_scale(&m, &k, cw)
}

/// POISSON pmf candidate graph with per-intermediate spill mask (see header).
fn pmfk_candidate(k: u32, lam: f64, mask: u32) -> f64 {
    use rx::{CW_PC64_RN, ext_from_f64, ext_fyl2x, ext_ln2, ext_mul, ext_to_f64};
    let cw = CW_PC64_RN;
    let spill = |e: &rx::Ext80, on: bool| -> rx::Ext80 {
        if on {
            ext_from_f64(ext_to_f64(e, cw))
        } else {
            *e
        }
    };
    let pow_raw = if mask & 64 != 0 {
        let le = ext_from_f64(lam);
        let mut p = le;
        for _ in 1..k {
            p = ext_mul(&p, &le, cw);
        }
        p
    } else {
        let ln_e = spill(&ext_fyl2x(&ext_ln2(), &ext_from_f64(lam), cw), mask & 1 != 0);
        let t_e = spill(&ext_mul(&ext_from_f64(k as f64), &ln_e, cw), mask & 2 != 0);
        exp_chain_from_ext(&t_e)
    };
    let pow_e = spill(&pow_raw, mask & 4 != 0);
    let e_e = spill(&exp_chain_ext(-lam), mask & 8 != 0);
    let prod = spill(&ext_mul(&pow_e, &e_e, cw), mask & 16 != 0);
    let kf = [1.0f64, 1.0, 2.0, 6.0][k as usize];
    let fin = rx::ext_div(&prod, &ext_from_f64(kf), cw);
    let cw_store = if mask & 32 != 0 {
        CW_PC64_RN | 0x0C00
    } else {
        CW_PC64_RN
    };
    ext_to_f64(&fin, cw_store)
}

/// BINOM pmf extended log-composed candidate graph (see header).
fn dbin_candidate(k: u32, n: u32, p: f64, mask: u32) -> f64 {
    use rx::{CW_PC64_RN, ext_add, ext_from_f64, ext_fyl2x, ext_ln2, ext_mul, ext_to_f64};
    let cw = CW_PC64_RN;
    let spill = |e: &rx::Ext80, on: bool| -> rx::Ext80 {
        if on {
            ext_from_f64(ext_to_f64(e, cw))
        } else {
            *e
        }
    };
    let q = 1.0 - p;
    let kk = k.min(n - k);
    let mut c_int: u128 = 1;
    for j in 0..kk {
        c_int = c_int * ((n - j) as u128) / ((j + 1) as u128); // exact: integer at each step
    }
    let c = c_int as f64; // RN for C > 2^53
    let lnc = spill(&ext_fyl2x(&ext_ln2(), &ext_from_f64(c), cw), mask & 1 != 0);
    let lnp = spill(&ext_fyl2x(&ext_ln2(), &ext_from_f64(p), cw), mask & 2 != 0);
    let lnq = spill(&ext_fyl2x(&ext_ln2(), &ext_from_f64(q), cw), mask & 4 != 0);
    let t1 = spill(&ext_mul(&ext_from_f64(k as f64), &lnp, cw), mask & 8 != 0);
    let t2 = spill(
        &ext_mul(&ext_from_f64((n - k) as f64), &lnq, cw),
        mask & 16 != 0,
    );
    let sum_raw = if mask & 64 != 0 {
        ext_add(&lnc, &ext_add(&t1, &t2, cw), cw)
    } else {
        ext_add(&ext_add(&lnc, &t1, cw), &t2, cw)
    };
    let sum = spill(&sum_raw, mask & 32 != 0);
    let r = exp_chain_from_ext(&sum);
    let cw_store = if mask & 128 != 0 {
        CW_PC64_RN | 0x0C00
    } else {
        CW_PC64_RN
    };
    ext_to_f64(&r, cw_store)
}

/// fFEXP chain on a double argument, result left EXTENDED (no final store).
fn exp_chain_ext(x: f64) -> rx::Ext80 {
    use rx::{
        CW_PC64_RN, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_l2e, ext_mul, ext_one,
        ext_rndint, ext_scale, ext_sub, ext_to_f64,
    };
    let cw = CW_PC64_RN;
    let t = ext_mul(&ext_from_f64(x), &ext_l2e(), cw);
    let k = ext_rndint(&t, cw);
    let f = ext_sub(&t, &k, cw);
    let neg = ext_to_f64(&f, cw) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, cw), cw);
    let mut m = ext_add(&w, &ext_one(), cw);
    if neg {
        m = ext_div(&ext_one(), &m, cw);
    }
    ext_scale(&m, &k, cw)
}

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    let g = |s: &str| f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap());
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let mut it = line.split_whitespace();
        let Some(op) = it.next() else { continue };
        let a = it.next().map(g);
        let b = it.next().map(g);
        let r = match (op, a, b) {
            ("exp", Some(x), _) => rx::excel_exp(x),
            ("expz", Some(x), _) => rx::excel_exp_rz(x),
            ("ln", Some(x), _) => rx::excel_ln(x),
            ("expm1", Some(x), _) => rx::excel_expm1_internal(x),
            ("mul", Some(x), Some(y)) => rx::x87_mul(x, y),
            ("div", Some(x), Some(y)) => {
                use rx::{CW_PC64_RN, ext_div, ext_from_f64, ext_to_f64};
                ext_to_f64(
                    &ext_div(&ext_from_f64(x), &ext_from_f64(y), CW_PC64_RN),
                    CW_PC64_RN,
                )
            }
            ("recip", Some(x), _) => rx::x87_recip(x),
            ("dbin", Some(x), Some(y)) => {
                // tokens: k n p mask  (k,n as hex-bits floats; mask decimal)
                let p = it.next().map(g).unwrap_or(f64::NAN);
                let mask: u32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                dbin_candidate(x as u32, y as u32, p, mask)
            }
            ("pmfk", Some(x), Some(y)) => {
                // third token = mask (decimal)
                let mask: u32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                pmfk_candidate(x as u32, y, mask)
            }
            ("lp1", Some(x), _) => {
                use rx::{CW_PC64_RN, ext_from_f64, ext_fyl2xp1, ext_ln2, ext_to_f64};
                ext_to_f64(
                    &ext_fyl2xp1(&ext_ln2(), &ext_from_f64(x), CW_PC64_RN),
                    CW_PC64_RN,
                )
            }
            ("lp1ext", Some(x), _) => {
                use rx::{CW_PC64_RN, ext_from_f64, ext_fyl2xp1, ext_ln2, ext_sub, ext_to_f64};
                let cw = CW_PC64_RN;
                let v = ext_fyl2xp1(&ext_ln2(), &ext_from_f64(x), cw);
                let hi = ext_to_f64(&v, cw);
                let lo = ext_to_f64(&ext_sub(&v, &ext_from_f64(hi), cw), cw);
                writeln!(out, "{:016x} {:016x}", hi.to_bits(), lo.to_bits()).unwrap();
                continue;
            }
            ("bd0dir", Some(x), Some(np)) => {
                use rx::{
                    CW_PC64_RN, ext_add, ext_div, ext_from_f64, ext_fyl2x, ext_ln2, ext_mul,
                    ext_sub, ext_to_f64,
                };
                let cw = CW_PC64_RN;
                let mask: u32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                let q = if mask & 1 != 0 {
                    ext_div(&ext_from_f64(x), &ext_from_f64(np), cw)
                } else {
                    ext_from_f64(x / np)
                };
                let l_raw = ext_fyl2x(&ext_ln2(), &q, cw);
                let l = if mask & 2 != 0 {
                    l_raw
                } else {
                    ext_from_f64(ext_to_f64(&l_raw, cw))
                };
                let xl = ext_mul(&ext_from_f64(x), &l, cw);
                let r = if mask & 8 != 0 {
                    // A: (x*L + np) - x
                    let t = ext_add(&xl, &ext_from_f64(np), cw);
                    let t = if mask & 4 != 0 {
                        t
                    } else {
                        ext_from_f64(ext_to_f64(&t, cw))
                    };
                    ext_sub(&t, &ext_from_f64(x), cw)
                } else {
                    // B: x*L + (np - x)
                    let d = ext_sub(&ext_from_f64(np), &ext_from_f64(x), cw);
                    let xl2 = if mask & 4 != 0 {
                        xl
                    } else {
                        ext_from_f64(ext_to_f64(&xl, cw))
                    };
                    ext_add(&xl2, &d, cw)
                };
                ext_to_f64(&r, cw)
            }
            ("lnext", Some(x), _) => {
                use rx::{
                    CW_PC64_RN, ext_from_f64, ext_fyl2x, ext_ln2, ext_sub, ext_to_f64,
                };
                let cw = CW_PC64_RN;
                let v = ext_fyl2x(&ext_ln2(), &ext_from_f64(x), cw);
                let hi = ext_to_f64(&v, cw);
                let lo = ext_to_f64(&ext_sub(&v, &ext_from_f64(hi), cw), cw);
                writeln!(out, "{:016x} {:016x}", hi.to_bits(), lo.to_bits()).unwrap();
                continue;
            }
            ("cexpext2", Some(x), Some(y)) => {
                use rx::{CW_PC64_RN, ext_add, ext_from_f64, ext_to_f64};
                let a = ext_add(&ext_from_f64(x), &ext_from_f64(y), CW_PC64_RN);
                let r = exp_chain_from_ext(&a);
                ext_to_f64(&r, CW_PC64_RN)
            }
            ("cexpext2p53r", Some(x), Some(y)) | ("cexpext2p53s", Some(x), Some(y)) => {
                // Mixed-PC chains: p53r = reduction (mul/rndint/sub) at PC53,
                // finals PC64; p53s = the converse.
                use rx::{
                    CW_PC53_RN, CW_PC64_RN, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64,
                    ext_l2e, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
                };
                let red53 = op == "cexpext2p53r";
                let cwr = if red53 { CW_PC53_RN } else { CW_PC64_RN };
                let cwf = if red53 { CW_PC64_RN } else { CW_PC53_RN };
                let a = ext_add(&ext_from_f64(x), &ext_from_f64(y), cwr);
                let t = ext_mul(&a, &ext_l2e(), cwr);
                let k = ext_rndint(&t, cwr);
                let f = ext_sub(&t, &k, cwr);
                let neg = ext_to_f64(&f, CW_PC64_RN) < 0.0;
                let w = ext_f2xm1(&ext_abs(&f, cwf), cwf);
                let mut m = ext_add(&w, &ext_one(), cwf);
                if neg {
                    m = ext_div(&ext_one(), &m, cwf);
                }
                let r = ext_scale(&m, &k, cwf);
                ext_to_f64(&r, CW_PC64_RN)
            }
            ("cexpext2mem", Some(x), Some(y)) => {
                // Entry value round-tripped through a tbyte park (expected
                // TRANSPARENT per the campaign's parking proof — included as
                // the explicit control).
                use rx::{CW_PC64_RN, ext_add, ext_from_f64, ext_to_f64};
                let a = ext_add(&ext_from_f64(x), &ext_from_f64(y), CW_PC64_RN);
                let parked = rx::Ext80(a.0); // byte-exact tbyte store/reload
                let r = exp_chain_from_ext(&parked);
                ext_to_f64(&r, CW_PC64_RN)
            }
            ("cexpext2rz", Some(x), Some(y)) => {
                // Chop (RC=RZ) on the reduction subtract f = t - k only
                // (site-dependent-chop precedent).
                use rx::{
                    CW_PC64_RN, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_l2e,
                    ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
                };
                let cw = CW_PC64_RN;
                let cwz = CW_PC64_RN | 0x0C00;
                let a = ext_add(&ext_from_f64(x), &ext_from_f64(y), cw);
                let t = ext_mul(&a, &ext_l2e(), cw);
                let k = ext_rndint(&t, cw);
                let f = ext_sub(&t, &k, cwz);
                let neg = ext_to_f64(&f, cw) < 0.0;
                let w = ext_f2xm1(&ext_abs(&f, cw), cw);
                let mut m = ext_add(&w, &ext_one(), cw);
                if neg {
                    m = ext_div(&ext_one(), &m, cw);
                }
                let r = ext_scale(&m, &k, cw);
                ext_to_f64(&r, cw)
            }
            ("cexpext2p53", Some(x), Some(y)) => {
                // The INLINED-intrinsic hypothesis: chain arithmetic under the
                // application default CW (PC=53) — reduction mul/rndint/sub and
                // the +1/reciprocal/scale round at 53 bits in-register, while
                // F2XM1/transcendental precision is unaffected by PC.
                use rx::{
                    CW_PC53_RN, CW_PC64_RN, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64,
                    ext_l2e, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
                };
                let cw = CW_PC53_RN;
                let a = ext_add(&ext_from_f64(x), &ext_from_f64(y), cw);
                let t = ext_mul(&a, &ext_l2e(), cw);
                let k = ext_rndint(&t, cw);
                let f = ext_sub(&t, &k, cw);
                let neg = ext_to_f64(&f, CW_PC64_RN) < 0.0;
                let w = ext_f2xm1(&ext_abs(&f, cw), cw);
                let mut m = ext_add(&w, &ext_one(), cw);
                if neg {
                    m = ext_div(&ext_one(), &m, cw);
                }
                let r = ext_scale(&m, &k, cw);
                ext_to_f64(&r, CW_PC64_RN)
            }
            ("mulex", Some(x), Some(y)) => {
                use rx::{CW_PC64_RN, ext_from_f64, ext_mul, ext_to_f64};
                let e = exp_chain_ext(x);
                ext_to_f64(&ext_mul(&e, &ext_from_f64(y), CW_PC64_RN), CW_PC64_RN)
            }
            ("mulee", Some(x), Some(y)) => {
                use rx::{CW_PC64_RN, ext_mul, ext_to_f64};
                let ea = exp_chain_ext(x);
                let eb = exp_chain_ext(y);
                ext_to_f64(&ext_mul(&ea, &eb, CW_PC64_RN), CW_PC64_RN)
            }
            ("cexpext", Some(x), Some(y)) => {
                use rx::{
                    CW_PC64_RN, ext_abs, ext_add, ext_div, ext_f2xm1, ext_from_f64, ext_l2e,
                    ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
                };
                let cw = CW_PC64_RN;
                let xe = ext_mul(&ext_from_f64(x), &ext_from_f64(y), cw);
                let t = ext_mul(&xe, &ext_l2e(), cw);
                let k = ext_rndint(&t, cw);
                let f = ext_sub(&t, &k, cw);
                let neg = ext_to_f64(&f, cw) < 0.0;
                let w = ext_f2xm1(&ext_abs(&f, cw), cw);
                let mut m = ext_add(&w, &ext_one(), cw);
                if neg {
                    m = ext_div(&ext_one(), &m, cw);
                }
                let r = ext_scale(&m, &k, cw);
                ext_to_f64(&r, cw)
            }
            _ => {
                writeln!(out, "ERR").unwrap();
                continue;
            }
        };
        writeln!(out, "{:016x}", r.to_bits()).unwrap();
    }
}
