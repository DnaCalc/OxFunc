//! W109 G6-01 LOG1P LANE: identify Excel's internal FAITHFUL log1p by the
//! CONFOUND-FREE "clean function" test.
//!
//! Excel PMT internal: tau_double = RN(-n*log1p_EXCEL(r)); em = expm1_x87(tau_double).
//! Since em is a pure function of the *double* tau, the TRUE log1p candidate must
//! satisfy: any two configs that share tau_cand_double MUST share em_pinned.
//! Groups where they don't = CONFLICTS. This test never invokes the expm1 model,
//! so it is immune to the |tau|<1 expm1 op-graph wall that confounds em-reproduction.
//!
//! Metrics per candidate, over the collide (128 pinned configs) and genrate sets:
//!   (A) em-reproduction  : #configs where expm1_x87(RN(-n*l1p(r))) == em_pinned  (CONFOUNDED)
//!   (B) clean-function   : #conflicting tau-groups + #configs in conflict        (CONFOUND-FREE)
//!   (C) heldout full PMT : /875
//! Candidates: CR (portable), FYL2XP1 (x87 hw), std ln_1p, UCRT extern log1p,
//!             fdlibm __log1p (exact), Cephes log1p (exact), kahan-companion, lnfl (control).
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_from_f64 as ef, ext_fyl2xp1, ext_fyl2x, ext_ln2, ext_add, ext_one, ext_to_f64, CW_PC64_RN as CW};
use std::collections::BTreeMap;

// ---- candidate log1p routines (all -> f64) ----
fn l1p_cr(r: f64) -> f64 { rx::excel_log1p(r) }
fn l1p_std(r: f64) -> f64 { r.ln_1p() }
unsafe extern "C" { fn log1p(x: f64) -> f64; }
fn l1p_ucrt(r: f64) -> f64 { unsafe { log1p(r) } }
fn l1p_lnfl(r: f64) -> f64 { rx::excel_ln(1.0 + r) } // negative control
fn l1p_kahan(r: f64) -> f64 { let u = 1.0 + r; if u == 1.0 { r } else { rx::excel_ln(u) * r / (u - 1.0) } }
fn l1p_fyl(r: f64) -> f64 {
    if r.abs() < 0.292893218813452 { ext_to_f64(&ext_fyl2xp1(&ext_ln2(), &ef(r), CW), CW) }
    else { ext_to_f64(&ext_fyl2x(&ext_ln2(), &ext_add(&ext_one(), &ef(r), CW), CW), CW) }
}

// fdlibm __log1p (Sun 1993), exact f64 reproduction.
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
    let (mut k, mut f, mut hu, mut c): (i32, f64, u32, f64) = (1, 0.0, 0, 0.0);
    let mut hu_flag_setf = false;
    if hx < 0x3FDA827A {
        if ax >= 0x3ff00000 {
            if x == -1.0 { return f64::NEG_INFINITY; } else { return (x - x) / (x - x); }
        }
        if ax < 0x3e200000 {
            if TWO54 + x > 0.0 && ax < 0x3c900000 { return x; }
            else { return x - x * x * 0.5; }
        }
        if hx > 0 || hx <= (0xbfd2bec3u32 as i32) {
            k = 0; f = x; hu = 1; hu_flag_setf = true;
        }
    }
    if hx >= 0x7ff00000 { return x + x; }
    if !hu_flag_setf {
        if hx < 0x43400000 {
            let u = 1.0 + x;
            hu = (u.to_bits() >> 32) as u32;
            k = (hu >> 20) as i32 - 1023;
            c = if k > 0 { 1.0 - (u - x) } else { x - (u - 1.0) };
            c /= u;
            hu &= 0x000fffff;
            let mut ub = u.to_bits();
            if hu < 0x6a09e {
                ub = (ub & 0x00000000ffffffff) | (((hu | 0x3ff00000) as u64) << 32);
                let u2 = f64::from_bits(ub);
                f = u2 - 1.0;
            } else {
                k += 1;
                ub = (ub & 0x00000000ffffffff) | (((hu | 0x3fe00000) as u64) << 32);
                let u2 = f64::from_bits(ub);
                hu = (0x00100000 - hu) >> 2;
                f = u2 - 1.0;
            }
        } else {
            let u = x;
            hu = (u.to_bits() >> 32) as u32;
            k = (hu >> 20) as i32 - 1023;
            c = 0.0;
            hu &= 0x000fffff;
            let mut ub = u.to_bits();
            if hu < 0x6a09e {
                ub = (ub & 0x00000000ffffffff) | (((hu | 0x3ff00000) as u64) << 32);
                let u2 = f64::from_bits(ub);
                f = u2 - 1.0;
            } else {
                k += 1;
                ub = (ub & 0x00000000ffffffff) | (((hu | 0x3fe00000) as u64) << 32);
                let u2 = f64::from_bits(ub);
                hu = (0x00100000 - hu) >> 2;
                f = u2 - 1.0;
            }
        }
    }
    let hfsq = 0.5 * f * f;
    if hu == 0 {
        if f == 0.0 {
            if k == 0 { return 0.0; } else { let c2 = c + k as f64 * LN2_LO; return k as f64 * LN2_HI + c2; }
        }
        let r = hfsq * (1.0 - 0.66666666666666666 * f);
        if k == 0 { return f - r; } else { return k as f64 * LN2_HI - ((r - (k as f64 * LN2_LO + c)) - f); }
    }
    let s = f / (2.0 + f);
    let z = s * s;
    let r = z * (LP1 + z * (LP2 + z * (LP3 + z * (LP4 + z * (LP5 + z * (LP6 + z * LP7))))));
    if k == 0 { f - (hfsq - s * (hfsq + r)) }
    else { k as f64 * LN2_HI - ((hfsq - (s * (hfsq + r) + (k as f64 * LN2_LO + c)) as f64) - f) }
}

// Cephes log1p (Moshier), exact f64 reproduction. Poly branch only for the
// collision domain (z=1+x in [SQRTH, SQRT2]); log(z) fallback delegates to CR.
fn l1p_cephes(x: f64) -> f64 {
    const LP: [f64; 7] = [
        4.5270000862445199635215e-5, 4.9854102823193375972212e-1, 6.5787325942061044846969e0,
        2.9911919328553073277375e1, 6.0949667980987787057556e1, 5.7112963590585538103336e1,
        2.0039553499201281259648e1,
    ];
    const LQ: [f64; 6] = [
        1.5062909083469192043167e1, 8.3047565967967209469434e1, 2.2176239823732856465394e2,
        3.0909872225312059774938e2, 2.1642788614495947685003e2, 6.0118660497603843919306e1,
    ];
    const SQRTH: f64 = 0.70710678118654752440;
    const SQRT2: f64 = 1.41421356237309504880;
    let z0 = 1.0 + x;
    if z0 < SQRTH || z0 > SQRT2 { return rx::excel_ln(z0); }
    // polevl(x, LP, 6)
    let mut pn = LP[0];
    for i in 1..7 { pn = pn * x + LP[i]; }
    // p1evl(x, LQ, 6) : monic degree 6
    let mut pd = x + LQ[0];
    for i in 1..6 { pd = pd * x + LQ[i]; }
    let xx = x * x;
    let z = -0.5 * xx + x * (xx * pn / pd);
    x + z
}

fn em_for(r: f64, n: f64, l1p: fn(f64) -> f64) -> f64 { rx::excel_expm1_internal(-(n * l1p(r))) }
fn tau_for(r: f64, n: f64, l1p: fn(f64) -> f64) -> f64 { -(n * l1p(r)) }

fn pin_gen(rows: &[(f64, u64)], r: f64, center: f64) -> Option<f64> {
    let cb = center.to_bits() as i64;
    for d in -48..=48i64 {
        let em = f64::from_bits((cb + d) as u64);
        if em >= 0.0 { continue }
        if rows.iter().all(|(pv, want)| ((pv / em) * r).to_bits() == *want) { return Some(em); }
    }
    None
}
fn load(p: &str) -> BTreeMap<(u64, u64), Vec<(f64, u64)>> {
    let ws: WitnessSet = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    let mut m = BTreeMap::new();
    for w in &ws.witnesses {
        let a: Vec<f64> = w.args.iter().filter_map(|x| match x { WitnessArg::Scalar(s) => parse_bits_hex(s), _ => None }).collect();
        if a.len() != 5 || a[3] != 0.0 || a[4] != 0.0 { continue }
        let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
        m.entry((a[0].to_bits(), a[1].to_bits())).or_insert_with(Vec::new).push((a[2], want));
    }
    m
}
fn pmt_full(r: f64, n: f64, pv: f64, fv: f64, ty: f64, l1p: fn(f64) -> f64) -> f64 {
    if r == 0.0 { return -(pv + fv) / n; }
    let tau = -(n * l1p(r)); let em = rx::excel_expm1_internal(tau); let v = 1.0 + em;
    let tf = 1.0 + r * ty; let num = pv + fv * v; ((num / em) / tf) * r
}
// 1+r exact iff two_sum low part is zero
fn onepr_exact(r: f64) -> bool { let u = 1.0 + r; (u - 1.0) == r && (1.0 - (u - r)) == 0.0 || ((u - 1.0) == r) }

type Cand = (&'static str, fn(f64) -> f64);

fn main() {
    let cands: [Cand; 8] = [
        ("CR", l1p_cr), ("FYL2XP1", l1p_fyl), ("std_ln1p", l1p_std), ("UCRT", l1p_ucrt),
        ("fdlibm", l1p_fdlibm), ("cephes", l1p_cephes), ("kahan", l1p_kahan), ("lnfl", l1p_lnfl),
    ];

    for (setname, path) in [
        ("collide", "../../work/w109/G6-solvers/answers-pmt-collide.json"),
        ("genrate", "../../work/w109/G6-solvers/answers-pmt-genrate.json"),
    ] {
        let data = load(path);
        // pin em per config (model-free: 128 pv rows constrain; CR-kahan center is only a seed)
        let mut pins: Vec<(f64, f64, f64, bool)> = Vec::new(); // r,n,em,onepr_exact
        for ((rb, nb), rows) in &data {
            let r = f64::from_bits(*rb); let n = f64::from_bits(*nb);
            let km = rx::excel_expm1_internal(-(n * rx::excel_log1p(r)));
            if let Some(e) = pin_gen(rows, r, km) { pins.push((r, n, e, onepr_exact(r))); }
        }
        let n_exact = pins.iter().filter(|p| p.3).count();
        println!("\n==== {} : {} configs pinned ({} with 1+r exact) ====", setname, pins.len(), n_exact);
        println!("{:<10} {:>10} {:>8} {:>8} {:>10}", "cand", "em-repro", "groups", "conflG", "conflCfg");
        for (nm, cf) in cands {
            // Metric A: em reproduction
            let repro = pins.iter().filter(|(r, n, ep, _)| em_for(*r, *n, cf).to_bits() == ep.to_bits()).count();
            // Metric B: clean-function test - group by tau_cand bits
            let mut groups: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
            for (r, n, ep, _) in &pins {
                groups.entry(tau_for(*r, *n, cf).to_bits()).or_default().push(ep.to_bits());
            }
            let ngroups = groups.len();
            let mut confl_g = 0usize; let mut confl_cfg = 0usize;
            for (_t, ems) in &groups {
                let distinct: std::collections::BTreeSet<u64> = ems.iter().copied().collect();
                if distinct.len() > 1 { confl_g += 1; confl_cfg += ems.len(); }
            }
            println!("{:<10} {:>7}/{:<3} {:>8} {:>8} {:>10}", nm, repro, pins.len(), ngroups, confl_g, confl_cfg);
        }
    }

    // ---- DUMP collide families to CSV for partition analysis ----
    {
        let data = load("../../work/w109/G6-solvers/answers-pmt-collide.json");
        let mut rows: Vec<(f64, f64, f64)> = Vec::new();
        for ((rb, nb), r_rows) in &data {
            let r = f64::from_bits(*rb); let n = f64::from_bits(*nb);
            let km = rx::excel_expm1_internal(-(n * rx::excel_log1p(r)));
            if let Some(e) = pin_gen(r_rows, r, km) { rows.push((r, n, e)); }
        }
        let mut out = String::from("r_bits,n,em_pinned,tau_cr,tau_fyl,tau_ucrt,tau_fdlibm,tau_cephes\n");
        for (r, n, em) in &rows {
            out.push_str(&format!("{:016x},{},{:016x},{:016x},{:016x},{:016x},{:016x},{:016x}\n",
                r.to_bits(), *n as u64, em.to_bits(),
                tau_for(*r, *n, l1p_cr).to_bits(), tau_for(*r, *n, l1p_fyl).to_bits(),
                tau_for(*r, *n, l1p_ucrt).to_bits(), tau_for(*r, *n, l1p_fdlibm).to_bits(),
                tau_for(*r, *n, l1p_cephes).to_bits()));
        }
        std::fs::write("../../work/w109/G6-solvers/log1p_collide_dump.csv", out).unwrap();
        println!("\nwrote log1p_collide_dump.csv ({} configs)", rows.len());
    }

    // heldout full PMT
    let ws: WitnessSet = serde_json::from_str(&std::fs::read_to_string(
        "../../work/w109/G6-solvers/answers-pmt-heldout.json").unwrap()).unwrap();
    println!("\n==== heldout full PMT ({} rows) ====", ws.witnesses.len());
    for (nm, cf) in cands {
        let mut ok = 0u32; let mut tot = 0u32;
        for w in &ws.witnesses {
            let a: Vec<f64> = w.args.iter().filter_map(|x| match x { WitnessArg::Scalar(s) => parse_bits_hex(s), _ => None }).collect();
            if a.len() != 5 { continue }
            let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
            tot += 1; if pmt_full(a[0], a[1], a[2], a[3], a[4], cf).to_bits() == want { ok += 1; }
        }
        println!("  {:<10} {}/{}", nm, ok, tot);
    }

    // Cross-check: does UCRT == std_ln1p on all collide r? (proxy validation)
    let data = load("../../work/w109/G6-solvers/answers-pmt-collide.json");
    let mut diff = 0;
    for ((rb, _), _) in &data { let r = f64::from_bits(*rb); if l1p_std(r).to_bits() != l1p_ucrt(r).to_bits() { diff += 1; } }
    println!("\nstd_ln1p vs UCRT extern log1p differ on {}/{} collide rates", diff, data.len());
}
