//! W109 G6-01 LOG1P LANE - the CONFOUND-FREE log1p discriminator.
//! At |tau|>=1 Excel's internal expm1 returns u-1 = x87exp(tau_double)-1 (100% exact,
//! no |tau|<1 Kahan wall). So for fv=0,ty=0, GENERAL-rate PMT:
//!     pmt = RN( RN(pv/em) * r ),  em = excel_exp(RN(-n*log1p(r))) - 1
//! the ONLY unknown is log1p. Scoring forward-PMT on general-rate |tau|>=1 witnesses
//! therefore tests log1p with NO expm1-wall confound and NO em-pinning. Winner = true log1p.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN as CW, ext_add, ext_from_f64 as ef, ext_fyl2x, ext_fyl2xp1, ext_ln2, ext_one,
    ext_to_f64,
};

fn l1p_cr(r: f64) -> f64 {
    rx::excel_log1p(r)
}
fn l1p_std(r: f64) -> f64 {
    r.ln_1p()
}
fn l1p_lnfl(r: f64) -> f64 {
    rx::excel_ln(1.0 + r)
}
fn l1p_fyl(r: f64) -> f64 {
    if r.abs() < 0.292893218813452 {
        ext_to_f64(&ext_fyl2xp1(&ext_ln2(), &ef(r), CW), CW)
    } else {
        ext_to_f64(
            &ext_fyl2x(&ext_ln2(), &ext_add(&ext_one(), &ef(r), CW), CW),
            CW,
        )
    }
}
// fdlibm __log1p (exact) - re-used from race_log1p_clean logic (small-x path only needed here)
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
    let mut setf = false;
    if hx < 0x3FDA827A {
        if ax >= 0x3ff00000 {
            if x == -1.0 {
                return f64::NEG_INFINITY;
            } else {
                return (x - x) / (x - x);
            }
        }
        if ax < 0x3e200000 {
            if TWO54 + x > 0.0 && ax < 0x3c900000 {
                return x;
            } else {
                return x - x * x * 0.5;
            }
        }
        if hx > 0 || hx <= (0xbfd2bec3u32 as i32) {
            k = 0;
            f = x;
            hu = 1;
            setf = true;
        }
    }
    if hx >= 0x7ff00000 {
        return x + x;
    }
    if !setf {
        let base = if hx < 0x43400000 {
            let u = 1.0 + x;
            hu = (u.to_bits() >> 32) as u32;
            k = (hu >> 20) as i32 - 1023;
            c = if k > 0 { 1.0 - (u - x) } else { x - (u - 1.0) };
            c /= u;
            u
        } else {
            let u = x;
            hu = (u.to_bits() >> 32) as u32;
            k = (hu >> 20) as i32 - 1023;
            c = 0.0;
            u
        };
        hu &= 0x000fffff;
        let mut ub = base.to_bits();
        if hu < 0x6a09e {
            ub = (ub & 0xffffffff) | (((hu | 0x3ff00000) as u64) << 32);
            f = f64::from_bits(ub) - 1.0;
        } else {
            k += 1;
            ub = (ub & 0xffffffff) | (((hu | 0x3fe00000) as u64) << 32);
            hu = (0x00100000 - hu) >> 2;
            f = f64::from_bits(ub) - 1.0;
        }
    }
    let hfsq = 0.5 * f * f;
    if hu == 0 {
        if f == 0.0 {
            if k == 0 {
                return 0.0;
            } else {
                return k as f64 * LN2_HI + (c + k as f64 * LN2_LO);
            }
        }
        let r = hfsq * (1.0 - 0.66666666666666666 * f);
        if k == 0 {
            return f - r;
        } else {
            return k as f64 * LN2_HI - ((r - (k as f64 * LN2_LO + c)) - f);
        }
    }
    let s = f / (2.0 + f);
    let z = s * s;
    let r = z * (LP1 + z * (LP2 + z * (LP3 + z * (LP4 + z * (LP5 + z * (LP6 + z * LP7))))));
    if k == 0 {
        f - (hfsq - s * (hfsq + r))
    } else {
        k as f64 * LN2_HI - ((hfsq - (s * (hfsq + r) + (k as f64 * LN2_LO + c))) - f)
    }
}

fn onepr_exact(r: f64) -> bool {
    (1.0 + r) - 1.0 == r
}
fn pmt_full(r: f64, n: f64, pv: f64, fv: f64, ty: f64, l1p: fn(f64) -> f64) -> f64 {
    if r == 0.0 {
        return -(pv + fv) / n;
    }
    let tau = -(n * l1p(r));
    let em = rx::excel_expm1_internal(tau);
    let v = 1.0 + em;
    let tf = 1.0 + r * ty;
    let num = pv + fv * v;
    ((num / em) / tf) * r
}

type Cand = (&'static str, fn(f64) -> f64);
fn main() {
    let cands: [Cand; 6] = [
        ("CR", l1p_cr),
        ("FYL2XP1", l1p_fyl),
        ("std/UCRT", l1p_std),
        ("fdlibm", l1p_fdlibm),
        ("lnfl(ctrl)", l1p_lnfl),
        ("(dummy)", l1p_cr),
    ];
    let files = [
        "answers-pmt-em.json",
        "answers-pmt-heldout.json",
        "answers-pmt-log1p.json",
        "answers-pmt-denselog1p.json",
        "answers-pmt-collide.json",
        "answers-pmt-genrate.json",
    ];
    // Buckets: (general-rate, |tau|>=1)=CLEAN log1p ; (general, |tau|<1)=confounded ; (exact-1+r)=no-log1p
    // We score forward PMT (fv=0,ty=0) per bucket.
    let mut clean: Vec<(f64, f64, f64, u64)> = Vec::new(); // r,n,pv,want  general & |tau|>=1
    let mut confl: Vec<(f64, f64, f64, u64)> = Vec::new(); // general & |tau|<1
    for fname in files {
        let p = format!("../../work/w109/G6-solvers/{}", fname);
        let ws: WitnessSet = match std::fs::read_to_string(&p) {
            Ok(s) => serde_json::from_str(&s).unwrap(),
            Err(_) => continue,
        };
        for w in &ws.witnesses {
            let a: Vec<f64> = w
                .args
                .iter()
                .filter_map(|x| match x {
                    WitnessArg::Scalar(s) => parse_bits_hex(s),
                    _ => None,
                })
                .collect();
            if a.len() != 5 || a[3] != 0.0 || a[4] != 0.0 || a[0] == 0.0 {
                continue;
            }
            let (r, n, pv) = (a[0], a[1], a[2]);
            if onepr_exact(r) {
                continue;
            }
            let want = parse_bits_hex(&w.expected_bits).unwrap().to_bits();
            let tau_abs = (n * rx::excel_log1p(r)).abs();
            if tau_abs >= 1.0 {
                clean.push((r, n, pv, want));
            } else {
                confl.push((r, n, pv, want));
            }
        }
    }
    // dedup rows
    clean.sort_by(|a, b| {
        a.3.cmp(&b.3)
            .then(a.0.to_bits().cmp(&b.0.to_bits()))
            .then(a.1.to_bits().cmp(&b.1.to_bits()))
            .then(a.2.to_bits().cmp(&b.2.to_bits()))
    });
    clean.dedup();
    confl.dedup();
    println!(
        "CONFOUND-FREE bucket (general rate, |tau|>=1, fv=0): {} witnesses",
        clean.len()
    );
    println!("   -> em = x87exp(tau)-1 EXACT, so this tests log1p ONLY\n");
    for (nm, cf) in cands {
        if nm == "(dummy)" {
            continue;
        }
        let ok = clean
            .iter()
            .filter(|(r, n, pv, want)| pmt_full(*r, *n, *pv, 0.0, 0.0, cf).to_bits() == *want)
            .count();
        println!(
            "   {:<12} {}/{}  ({:.1}%)",
            nm,
            ok,
            clean.len(),
            100.0 * ok as f64 / clean.len() as f64
        );
    }
    println!(
        "\nCONFOUNDED bucket (general rate, |tau|<1, fv=0): {} witnesses  [expm1 wall present]",
        confl.len()
    );
    for (nm, cf) in cands {
        if nm == "(dummy)" {
            continue;
        }
        let ok = confl
            .iter()
            .filter(|(r, n, pv, want)| pmt_full(*r, *n, *pv, 0.0, 0.0, cf).to_bits() == *want)
            .count();
        println!(
            "   {:<12} {}/{}  ({:.1}%)",
            nm,
            ok,
            confl.len(),
            100.0 * ok as f64 / confl.len() as f64
        );
    }
}
