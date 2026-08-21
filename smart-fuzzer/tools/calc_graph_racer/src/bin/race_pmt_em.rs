//! W109 G6-01 PMT em-isolation (2026-07-21, Fable H1 test with REAL x87).
//! On the collapsed grid (pv, fv=0, ty=0) Excel's pmt = RN(pv*r/em). So scoring
//! `fl(pv*r/em_candidate) == pmt_excel` directly identifies Excel's internal em.
//! mpmath can't emulate the F2XM1/FYL2XP1 instruction chain; this bin runs the
//! real x87 primitives with the -1 fold and RN-vs-chop final store.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

const CW: u16 = rx::CW_PC64_RN;
const CW_RZ: u16 = rx::CW_PC64_RN | 0x0C00; // chop final store

fn tf64(e: &rx::Ext80) -> f64 {
    rx::ext_to_f64(e, CW)
}
fn tf64rz(e: &rx::Ext80) -> f64 {
    rx::ext_to_f64(e, CW_RZ)
}

// natural log1p(r) as Ext80
#[derive(Clone, Copy)]
enum LP {
    Fyl2xp1,
    Log1pCR,
    LnOf1PlusR,
}
fn logp_ext(lp: LP, r: f64) -> rx::Ext80 {
    match lp {
        LP::Fyl2xp1 => rx::ext_fyl2xp1(&rx::ext_ln2(), &rx::ext_from_f64(r), CW), // ln2*log2(1+r)=ln(1+r)
        LP::Log1pCR => rx::ext_from_f64(rx::excel_log1p(r)),
        LP::LnOf1PlusR => rx::ext_from_f64(rx::excel_ln(1.0 + r)),
    }
}

// expm1 of t (t = -n*log1p) via several providers. `t` given as Ext80.
#[derive(Clone, Copy)]
enum EP {
    Internal,
    PortCR,
    F2xm1FoldRN,
    F2xm1FoldRZ,
    F2xm1NoFold,
    ExpM1RN,
}
fn expm1_of(ep: EP, t: &rx::Ext80) -> f64 {
    match ep {
        EP::Internal => rx::excel_expm1_internal(tf64(t)),
        EP::PortCR => rx::excel_expm1(tf64(t)),
        EP::ExpM1RN => rx::excel_exp(tf64(t)) - 1.0, // exp then -1 in double (RN store of exp first)
        _ => {
            // native x87 base-2 expm1: z=t*log2e; k=rndint; f=z-k; w=F2XM1(|f| leg)
            let z = rx::ext_mul(t, &rx::ext_l2e(), CW);
            let k = rx::ext_rndint(&z, CW);
            let f = rx::ext_sub(&z, &k, CW);
            let neg = rx::ext_to_f64(&f, CW) < 0.0;
            let w = rx::ext_f2xm1(&rx::ext_abs(&f, CW), CW); // 2^|f|-1
            // 2^f - 1 accounting for sign of f: 2^(-|f|)-1 = -(2^|f|-1)/2^|f|
            let fm1 = if neg {
                let twof = rx::ext_add(&w, &rx::ext_one(), CW);
                rx::ext_div(&rx::ext_sub(&rx::ext_from_f64(0.0), &w, CW), &twof, CW)
            } else {
                w
            };
            let kf = rx::ext_to_f64(&k, CW);
            match ep {
                EP::F2xm1FoldRN | EP::F2xm1FoldRZ => {
                    // em = scale(1+fm1, k) - 1, fold -1 in EXTENDED, single store
                    let p2f = rx::ext_add(&fm1, &rx::ext_one(), CW);
                    let em_ext = if kf == 0.0 {
                        fm1
                    } else {
                        let scaled = rx::ext_scale(&p2f, &k, CW);
                        rx::ext_sub(&scaled, &rx::ext_one(), CW)
                    };
                    if matches!(ep, EP::F2xm1FoldRZ) {
                        tf64rz(&em_ext)
                    } else {
                        tf64(&em_ext)
                    }
                }
                EP::F2xm1NoFold => {
                    // P=2^t stored to double first, then P-1 in double (should be worse)
                    let p2f = rx::ext_add(&fm1, &rx::ext_one(), CW);
                    let p = tf64(&rx::ext_scale(&p2f, &k, CW));
                    p - 1.0
                }
                _ => unreachable!(),
            }
        }
    }
}

// em as EXTENDED Ext80 (F2XM1 fold, NO store) — for the pure x87-spill body.
fn em_ext(lp: LP, n: f64, r: f64) -> rx::Ext80 {
    let l = logp_ext(lp, r);
    let t = rx::ext_mul(&rx::ext_from_f64(-n), &l, CW);
    let z = rx::ext_mul(&t, &rx::ext_l2e(), CW);
    let k = rx::ext_rndint(&z, CW);
    let f = rx::ext_sub(&z, &k, CW);
    let neg = rx::ext_to_f64(&f, CW) < 0.0;
    let w = rx::ext_f2xm1(&rx::ext_abs(&f, CW), CW);
    let fm1 = if neg {
        let twof = rx::ext_add(&w, &rx::ext_one(), CW);
        rx::ext_div(&rx::ext_sub(&rx::ext_from_f64(0.0), &w, CW), &twof, CW)
    } else {
        w
    };
    let kf = rx::ext_to_f64(&k, CW);
    if kf == 0.0 {
        fm1
    } else {
        let p2f = rx::ext_add(&fm1, &rx::ext_one(), CW);
        rx::ext_sub(&rx::ext_scale(&p2f, &k, CW), &rx::ext_one(), CW)
    }
}
// pure x87 spill: everything extended, single final store to double.
// pmt = (pv + fv*v)*r / (tf*em); here fv=0, ty=0 => (pv*r)/em_ext.
fn pmt_spill(lp: LP, r: f64, n: f64, pv: f64, chop: bool) -> f64 {
    let em = em_ext(lp, n, r);
    let numr = rx::ext_mul(
        &rx::ext_from_f64(pv),
        &rx::ext_mul(&rx::ext_from_f64(1.0), &rx::ext_from_f64(r), CW),
        CW,
    );
    let q = rx::ext_div(&numr, &em, CW);
    if chop {
        rx::ext_to_f64(&q, CW_RZ)
    } else {
        rx::ext_to_f64(&q, CW)
    }
}

fn load(path: &str) -> Vec<(f64, f64, f64, u64)> {
    let ws: WitnessSet =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("parse");
    let mut o = Vec::new();
    for w in &ws.witnesses {
        let a: Vec<f64> = w
            .args
            .iter()
            .filter_map(|x| match x {
                WitnessArg::Scalar(s) => parse_bits_hex(s),
                _ => None,
            })
            .collect();
        if a.len() != 5 {
            continue;
        }
        if a[3] != 0.0 || a[4] != 0.0 {
            continue;
        } // fv=0, ty=0 only
        if let Some(want) = parse_bits_hex(&w.expected_bits) {
            o.push((a[0], a[1], a[2], want.to_bits()));
        }
    }
    o
}

fn main() {
    let av: Vec<String> = std::env::args().collect();
    let path = av
        .get(1)
        .cloned()
        .unwrap_or_else(|| "../../work/w109/G6-solvers/answers-pmt-em.json".into());
    let rows = load(&path);
    let pv1: Vec<_> = rows
        .iter()
        .filter(|(_, _, pv, _)| *pv == 1.0)
        .cloned()
        .collect();
    println!("collapsed-grid rows: {} (pv=1: {})", rows.len(), pv1.len());
    let lps = [
        ("fyl2xp1", LP::Fyl2xp1),
        ("log1pCR", LP::Log1pCR),
        ("ln(1+r)", LP::LnOf1PlusR),
    ];
    let eps = [
        ("internal", EP::Internal),
        ("portCR", EP::PortCR),
        ("f2xm1FoldRN", EP::F2xm1FoldRN),
        ("f2xm1FoldRZ", EP::F2xm1FoldRZ),
        ("f2xm1NoFold", EP::F2xm1NoFold),
        ("expM1", EP::ExpM1RN),
    ];
    // also try t stored to double vs kept extended before expm1
    println!("\nscore fl(pv*r/em)==pmt over pv=1 rows ({}):", pv1.len());
    let mut best = (0u32, String::new());
    for (ln, lp) in lps {
        for (en, ep) in eps {
            for tstore in [false, true] {
                let mut ok = 0u32;
                for (r, n, pv, want) in &pv1 {
                    let l = logp_ext(lp, *r);
                    let mut t = rx::ext_mul(&rx::ext_from_f64(-*n), &l, CW);
                    if tstore {
                        t = rx::ext_from_f64(rx::ext_to_f64(&t, CW));
                    }
                    let em = expm1_of(ep, &t);
                    let g = (pv * r) / em;
                    if g.to_bits() == *want {
                        ok += 1;
                    }
                }
                let tag = format!("L={:8} E={:12} tstore={}", ln, en, tstore);
                if ok > best.0 {
                    best = (ok, tag.clone());
                }
                println!("  {:5}/{}  {}", ok, pv1.len(), tag);
            }
        }
    }
    println!(
        "\nBEST pv=1 (em-stored, SSE2 div): {}/{}  {}",
        best.0,
        pv1.len(),
        best.1
    );

    // pure x87 spill: em EXTENDED, whole body extended, single final store.
    println!("\n=== pure x87 spill (em extended, single final store) pv=1 ===");
    let mut bs = (0u32, String::new());
    for (ln, lp) in lps {
        for chop in [false, true] {
            let mut ok = 0u32;
            for (r, n, pv, want) in &pv1 {
                if pmt_spill(lp, *r, *n, *pv, chop).to_bits() == *want {
                    ok += 1;
                }
            }
            let tag = format!("L={:8} chop={}", ln, chop);
            if ok > bs.0 {
                bs = (ok, tag.clone());
            }
            println!("  {:5}/{}  {}", ok, pv1.len(), tag);
        }
    }
    println!("BEST spill pv=1: {}/{}  {}", bs.0, pv1.len(), bs.1);

    // regime split by |t|=|n*log1p(r)|: is em a different chain for small vs large t?
    // Candidate A: internal-Kahan em (best above). Candidate B: v=pow_chain(1+r,-n), em=v-1.
    // Candidate C: v=exp(-n*log1p r) [x87], em=v-1.
    println!("\n=== em by |t| regime (pv=1): exact counts ===");
    let em_internal = |r: f64, n: f64| {
        let l = rx::excel_log1p(r);
        rx::excel_expm1_internal(rx::x87_mul(-n, l))
    };
    let em_powchain = |r: f64, n: f64| {
        // v=exp(x87mul(-n, ln(1+r))), em=v-1
        let ln1pr = rx::excel_ln(1.0 + r);
        let v = rx::excel_exp(rx::x87_mul(-n, ln1pr));
        v - 1.0
    };
    let em_expfold = |r: f64, n: f64| {
        let l = rx::excel_log1p(r);
        let v = rx::excel_exp(rx::x87_mul(-n, l));
        v - 1.0
    };
    let bins = [(0.0, 0.25), (0.25, 1.0), (1.0, 4.0), (4.0, 1e9)];
    for (name, f) in [
        ("internalKahan", &em_internal as &dyn Fn(f64, f64) -> f64),
        ("powchain(1+r)", &em_powchain),
        ("expfold(log1p)", &em_expfold),
    ] {
        print!("  {:16}", name);
        for (lo, hi) in bins {
            let mut ok = 0u32;
            let mut tot = 0u32;
            for (r, n, pv, want) in &pv1 {
                let t = (*n * rx::excel_log1p(*r)).abs();
                if t < lo || t >= hi {
                    continue;
                }
                tot += 1;
                if ((pv * r) / f(*r, *n)).to_bits() == *want {
                    ok += 1;
                }
            }
            print!("  |t|[{:.2},{:.2}): {:4}/{:4}", lo, hi, ok, tot);
        }
        println!();
    }
    // regime-split combined model: expfold for |t|<thr, powchain for |t|>=thr
    println!("\nregime-split combined (expfold if |t|<thr else powchain):");
    for thr in [0.5f64, 1.0, 2.0, 4.0] {
        let mut ok = 0u32;
        for (r, n, pv, want) in &pv1 {
            let t = (*n * rx::excel_log1p(*r)).abs();
            let em = if t < thr {
                em_expfold(*r, *n)
            } else {
                em_powchain(*r, *n)
            };
            if ((pv * r) / em).to_bits() == *want {
                ok += 1;
            }
        }
        println!("  thr={:.1}: {:5}/{}", thr, ok, pv1.len());
    }

    // ALSO score the spill model on the full held-out corpus for cross-check
    let ho = load("../../work/w109/G6-solvers/answers-pmt-heldout.json");
    if !ho.is_empty() {
        // held-out has fv!=0/ty!=0 rows filtered out by load(); this is the fv=0,ty=0 subset
        let mut ok = 0u32;
        for (r, n, pv, want) in &ho {
            if pmt_spill(LP::Log1pCR, *r, *n, *pv, false).to_bits() == *want {
                ok += 1;
            }
        }
        println!(
            "\nspill (log1pCR,RN) on held-out fv=0/ty=0 subset: {}/{}",
            ok,
            ho.len()
        );
    }
}
