//! W109 G6-01 LOG1P LANE (agent-L): clean log1p isolation + provenance.
//!
//! Two problems with prior log1p races: BOTH the `collide` em-oracle and the full
//! `heldout` PMT route through the imperfect `excel_expm1_internal` (~163/234 at
//! |tau|<1), so a log1p miss and an expm1 miss are indistinguishable.
//!
//! CLEAN ISOLATOR: split heldout by |tau|. On |tau|>=1 the annuity factor
//! em = RN(exp_x87(tau)-1) is 100% exact (proven, notes), so em is an injective
//! function of tau and the ONLY error source is log1p. => the |tau|>=1 bucket is a
//! pure log1p discriminator; the |tau|<1 bucket is expm1-confounded.
//!
//! Candidates: CR (portable), FYL2XP1-hw (hybrid 0.293 threshold), std ln_1p
//! (= UCRT log1p on windows-msvc), Kahan companion (2 assoc), ln(fl(1+r)) control,
//! fdlibm s_log1p port, Cephes log1p port, and a double-rounded-(1+r) fyl2x.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64 as ef, ext_fyl2x, ext_fyl2xp1, ext_ln2, ext_one, ext_to_f64,
         CW_PC64_RN as CW};

// -------------------- log1p candidates --------------------
fn l1p_cr(r: f64) -> f64 { rx::excel_log1p(r) }
fn l1p_std(r: f64) -> f64 { r.ln_1p() }
fn l1p_kahan(r: f64) -> f64 { let u = 1.0 + r; if u == 1.0 { r } else { rx::excel_ln(u) * r / (u - 1.0) } }
fn l1p_kahan2(r: f64) -> f64 { let u = 1.0 + r; if u == 1.0 { r } else { rx::excel_ln(u) * (r / (u - 1.0)) } }
fn l1p_lnfl(r: f64) -> f64 { rx::excel_ln(1.0 + r) } // ln(fl(1+r)) double-rounded 1+r, ln=x87 -- control
fn l1p_fyl(r: f64) -> f64 {
    if r.abs() < 0.292893218813452 { ext_to_f64(&ext_fyl2xp1(&ext_ln2(), &ef(r), CW), CW) }
    else { ext_to_f64(&ext_fyl2x(&ext_ln2(), &ext_add(&ext_one(), &ef(r), CW), CW), CW) }
}
// FYL2XP1 over the FULL range (no threshold) delivered to double.
fn l1p_fylall(r: f64) -> f64 { ext_to_f64(&ext_fyl2xp1(&ext_ln2(), &ef(r), CW), CW) }

// -------------------- fdlibm s_log1p.c (portable double port) --------------------
fn l1p_fdlibm(x: f64) -> f64 {
    const LN2_HI: f64 = 6.93147180369123816490e-01;
    const LN2_LO: f64 = 1.90821492927058770002e-10;
    const TWO54: f64 = 1.80143985094819840000e+16;
    const LP1: f64 = 6.666666666666735130e-01;
    const LP2: f64 = 3.999999999940941908e-01;
    const LP3: f64 = 2.857142874366239149e-01;
    const LP4: f64 = 2.222219843214978396e-01;
    const LP5: f64 = 1.818357216161805012e-01;
    const LP6: f64 = 1.531383769920937332e-01;
    const LP7: f64 = 1.479819860511658591e-01;
    let hx = (x.to_bits() >> 32) as u32 as i32;
    let ax = hx & 0x7fffffff;
    let mut k = 1i32;
    let mut f = 0.0f64;
    let mut hu = 0i32;
    let mut c = 0.0f64;
    if hx < 0x3FDA827A {
        if ax >= 0x3ff00000 {
            if x == -1.0 { return -TWO54 / 0.0; } else { return (x - x) / (x - x); }
        }
        if ax < 0x3e200000 {
            if TWO54 + x > 0.0 && ax < 0x3c900000 { return x; } else { return x - x * x * 0.5; }
        }
        if hx > 0 || hx <= (0xbfd2bec4u32 as i32) { k = 0; f = x; hu = 1; }
    }
    if hx >= 0x7ff00000 { return x + x; }
    if k != 0 {
        if hx < 0x43400000 {
            let u = 1.0 + x;
            hu = (u.to_bits() >> 32) as u32 as i32;
            k = (hu >> 20) - 1023;
            c = if k > 0 { 1.0 - (u - x) } else { x - (u - 1.0) };
            c /= u;
            let mut hub = hu & 0x000fffff;
            let mut u2 = u;
            if hub < 0x6a09e {
                let bits = (u2.to_bits() & 0x0000_0000_ffff_ffff) | (((hub | 0x3ff00000) as u32 as u64) << 32);
                u2 = f64::from_bits(bits);
            } else {
                k += 1;
                let bits = (u2.to_bits() & 0x0000_0000_ffff_ffff) | (((hub | 0x3fe00000) as u32 as u64) << 32);
                u2 = f64::from_bits(bits);
                hub = (0x00100000 - hub) >> 2;
            }
            hu = hub;
            f = u2 - 1.0;
        } else {
            let u = x;
            hu = (u.to_bits() >> 32) as u32 as i32;
            k = (hu >> 20) - 1023;
            c = 0.0;
            let mut hub = hu & 0x000fffff;
            let mut u2 = u;
            if hub < 0x6a09e {
                let bits = (u2.to_bits() & 0x0000_0000_ffff_ffff) | (((hub | 0x3ff00000) as u32 as u64) << 32);
                u2 = f64::from_bits(bits);
            } else {
                k += 1;
                let bits = (u2.to_bits() & 0x0000_0000_ffff_ffff) | (((hub | 0x3fe00000) as u32 as u64) << 32);
                u2 = f64::from_bits(bits);
                hub = (0x00100000 - hub) >> 2;
            }
            hu = hub;
            f = u2 - 1.0;
        }
    }
    let hfsq = 0.5 * f * f;
    if hu == 0 {
        if f == 0.0 {
            if k == 0 { return 0.0; } else { let c2 = c + (k as f64) * LN2_LO; return (k as f64) * LN2_HI + c2; }
        }
        let r = hfsq * (1.0 - 0.66666666666666666 * f);
        if k == 0 { return f - r; } else { return (k as f64) * LN2_HI - ((r - ((k as f64) * LN2_LO + c)) - f); }
    }
    let s = f / (2.0 + f);
    let z = s * s;
    let r = z * (LP1 + z * (LP2 + z * (LP3 + z * (LP4 + z * (LP5 + z * (LP6 + z * LP7))))));
    if k == 0 { f - (hfsq - s * (hfsq + r)) }
    else { (k as f64) * LN2_HI - ((hfsq - (s * (hfsq + r) + ((k as f64) * LN2_LO + c)) - f)) }
}

// -------------------- Cephes log1p (double) --------------------
fn polevl(x: f64, c: &[f64]) -> f64 { let mut r = c[0]; for &ci in &c[1..] { r = r * x + ci; } r }
fn p1evl(x: f64, c: &[f64]) -> f64 { let mut r = x + c[0]; for &ci in &c[1..] { r = r * x + ci; } r }
fn l1p_cephes(x: f64) -> f64 {
    const SQRTH: f64 = 0.70710678118654752440;
    const SQRT2: f64 = 1.41421356237309504880;
    const LP: [f64; 7] = [
        4.5270000862445199635215e-5, 4.9854102823193375972212e-1, 6.5787325942061044846969e0,
        2.9911919328553073277375e1, 6.0949667980987787057556e1, 5.7112963590585538103336e1,
        2.0039553499201281259648e1,
    ];
    const LQ: [f64; 6] = [
        1.5062909083469192043167e1, 8.3047565967967209469434e1, 2.2176239823732856465394e2,
        3.0909872225312059774938e2, 2.1642788614495947685003e2, 6.0118660497603843919306e1,
    ];
    let z = 1.0 + x;
    if z < SQRTH || z > SQRT2 { return rx::excel_ln(z); } // cephes uses log(z); use x87 ln (matches Excel ln substrate)
    let z2 = x * x;
    let z3 = -0.5 * z2 + x * (z2 * polevl(x, &LP) / p1evl(x, &LQ));
    x + z3
}

// Cephes with libm/CR ln for the out-of-range tail (isolate the polynomial core).
fn l1p_cephes_crln(x: f64) -> f64 {
    const SQRTH: f64 = 0.70710678118654752440;
    const SQRT2: f64 = 1.41421356237309504880;
    const LP: [f64; 7] = [
        4.5270000862445199635215e-5, 4.9854102823193375972212e-1, 6.5787325942061044846969e0,
        2.9911919328553073277375e1, 6.0949667980987787057556e1, 5.7112963590585538103336e1,
        2.0039553499201281259648e1,
    ];
    const LQ: [f64; 6] = [
        1.5062909083469192043167e1, 8.3047565967967209469434e1, 2.2176239823732856465394e2,
        3.0909872225312059774938e2, 2.1642788614495947685003e2, 6.0118660497603843919306e1,
    ];
    let z = 1.0 + x;
    if z < SQRTH || z > SQRT2 { return z.ln(); }
    let z2 = x * x;
    let z3 = -0.5 * z2 + x * (z2 * polevl(x, &LP) / p1evl(x, &LQ));
    x + z3
}

type L1p = fn(f64) -> f64;
const CANDS: [(&str, L1p); 10] = [
    ("CR", l1p_cr), ("FYL2XP1hyb", l1p_fyl), ("FYL2XP1all", l1p_fylall), ("std_ln1p", l1p_std),
    ("kahan", l1p_kahan), ("kahan2", l1p_kahan2), ("fdlibm", l1p_fdlibm),
    ("cephes", l1p_cephes), ("cephes_crln", l1p_cephes_crln), ("lnfl_ctrl", l1p_lnfl),
];

// full PMT (discount combine) with a given log1p
fn pmt_full(r: f64, n: f64, pv: f64, fv: f64, ty: f64, l1p: L1p) -> f64 {
    if r == 0.0 { return -(pv + fv) / n; }
    let tau = -(n * l1p(r));
    let em = rx::excel_expm1_internal(tau);
    let v = 1.0 + em;
    let tf = 1.0 + r * ty;
    let num = pv + fv * v;
    ((num / em) / tf) * r
}

fn main() {
    let ws: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string("../../work/w109/G6-solvers/answers-pmt-heldout.json").unwrap()).unwrap();
    // buckets by |tau_cr|
    let mut hi = [0u32; 10]; let mut hi_tot = 0u32;   // |tau|>=1 : CLEAN log1p
    let mut lo = [0u32; 10]; let mut lo_tot = 0u32;   // |tau|<1  : expm1-confounded
    for w in &ws.witnesses {
        let a: Vec<f64> = w.args.iter().filter_map(|x| match x { WitnessArg::Scalar(s) => parse_bits_hex(s), _ => None }).collect();
        if a.len() != 5 { continue; }
        let (r, n, pv, fv, ty) = (a[0], a[1], a[2], a[3], a[4]);
        if r == 0.0 { continue; }
        let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
        let tau_cr = -(n * l1p_cr(r));
        let clean = tau_cr.abs() >= 1.0;
        if clean { hi_tot += 1; } else { lo_tot += 1; }
        for (i, (_nm, f)) in CANDS.iter().enumerate() {
            if pmt_full(r, n, pv, fv, ty, *f).to_bits() == want {
                if clean { hi[i] += 1; } else { lo[i] += 1; }
            }
        }
    }
    println!("HELDOUT split by |tau_cr| (clean=|tau|>=1 isolates log1p; expm1 exact there)\n");
    println!("{:<14} {:>14} {:>18} {:>14}", "candidate", "clean |tau|>=1", "confound |tau|<1", "TOTAL");
    for (i, (nm, _)) in CANDS.iter().enumerate() {
        println!("{:<14} {:>8}/{:<5} {:>12}/{:<5} {:>8}/{:<5}",
            nm, hi[i], hi_tot, lo[i], lo_tot, hi[i] + lo[i], hi_tot + lo_tot);
    }

    // Cross-check candidate log1p values against each other on the clean rows to see
    // which are BIT-IDENTICAL (provenance clustering).
    println!("\n--- pairwise identity of log1p variants over 20000 rates in [1e-6, 0.5] ---");
    let names: Vec<&str> = CANDS.iter().map(|c| c.0).collect();
    let mut diff = vec![vec![0u32; CANDS.len()]; CANDS.len()];
    let mut cnt = 0u32;
    let mut rr = 1e-6f64;
    while rr < 0.5 {
        cnt += 1;
        let vals: Vec<u64> = CANDS.iter().map(|c| (c.1)(rr).to_bits()).collect();
        for i in 0..CANDS.len() { for j in 0..CANDS.len() { if vals[i] != vals[j] { diff[i][j] += 1; } } }
        rr *= 1.0009;
    }
    print!("{:<12}", "");
    for n in &names { print!("{:>7}", &n[..n.len().min(6)]); }
    println!("  (of {} rates, #bit-differ)", cnt);
    for i in 0..CANDS.len() {
        print!("{:<12}", &names[i][..names[i].len().min(11)]);
        for j in 0..CANDS.len() { print!("{:>7}", diff[i][j]); }
        println!();
    }

    // ---- Characterize the CLEAN-bucket (|tau|>=1) CR misses ----
    // For each miss, find the minimal signed tau nudge (in ULP of tau) that makes
    // the FULL pmt match. Tells us whether the residual is a <=few-ULP tau (log1p/xN)
    // issue vs a deeper em/combine issue.
    println!("\n--- CLEAN-bucket CR-miss anatomy: minimal tau-ULP nudge that fixes ---");
    use std::collections::BTreeMap;
    let mut hist: BTreeMap<i64, u32> = BTreeMap::new();
    let mut unfixed = 0u32;
    let mut miss = 0u32;
    for w in &ws.witnesses {
        let a: Vec<f64> = w.args.iter().filter_map(|x| match x { WitnessArg::Scalar(s) => parse_bits_hex(s), _ => None }).collect();
        if a.len() != 5 { continue; }
        let (r, n, pv, fv, ty) = (a[0], a[1], a[2], a[3], a[4]);
        if r == 0.0 { continue; }
        let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
        let tau_cr = -(n * l1p_cr(r));
        if tau_cr.abs() < 1.0 { continue; }
        if pmt_full(r, n, pv, fv, ty, l1p_cr).to_bits() == want { continue; }
        miss += 1;
        // nudge tau directly
        let tb = tau_cr.to_bits() as i64;
        let mut fixed: Option<i64> = None;
        'search: for mag in 1..=6i64 {
            for &sgn in &[-1i64, 1] {
                let d = sgn * mag;
                let tau = f64::from_bits((tb + d) as u64);
                let em = rx::excel_expm1_internal(tau);
                let v = 1.0 + em; let tf = 1.0 + r * ty; let num = pv + fv * v;
                if (((num / em) / tf) * r).to_bits() == want { fixed = Some(d); break 'search; }
            }
        }
        match fixed { Some(d) => *hist.entry(d).or_insert(0) += 1, None => unfixed += 1 }
    }
    println!("clean CR misses: {}", miss);
    for (d, c) in &hist { println!("  tau nudge {:+} ulp : {}", d, c); }
    println!("  unfixed within +-6 ulp (NOT a small-tau issue): {}", unfixed);
}
