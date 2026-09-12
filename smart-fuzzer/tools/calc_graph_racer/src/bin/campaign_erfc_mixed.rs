//! Firehorse mixed F-body campaign: C/D/Facc Lentz stores, piecewise NSWC/CF
//! store-masks, Cephes P/Q and MATH77 Chebyshev stores. Scores merged + direct
//! + implied F_or. Heldouts unnamed. No landing.
//!
//!   campaign_erfc_mixed --dir G3-01-dist --out erfc-mixed-campaign --threads 10 --max-hours 14

use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rayon::prelude::*;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const CHUNK: u32 = 4096;
const TAIL_BAR: usize = 1283;
const MID_BAR: usize = 2389;
const ULP_CAP: u64 = 1 << 20;

#[derive(Clone, Copy)]
enum Kind {
    LentzAs714,
    LentzGaut,
    BackAs714,
    CephesPq,
    Cody,
    JointCodyCephes,
    Math77Erc2,
    Math77Erfcc,
}

#[derive(Clone, Copy)]
enum Sites {
    /// C store + facc store (2 bits/step).
    CFacc,
    /// D store + facc store.
    DFacc,
    /// C store + delta=c*d store.
    CDelta,
    /// D, C, facc (3 bits/step).
    CDFacc,
    /// Backward-CF den store (1 bit/step).
    Den,
    /// MATH77 dcsevl b0 store (1 bit/term).
    ChebB0,
    /// Cephes polevl/p1evl + quotient.
    Cephes,
}

#[derive(Clone)]
struct CubeJob {
    axis: String,
    kind: Kind,
    sites: Sites,
    nterms: u32,
    mask_lim: u32,
    cut: Option<f64>,
    note: String,
}

fn cube_jobs() -> Vec<CubeJob> {
    let mut jobs = Vec::new();
    jobs.push(CubeJob {
        axis: "cephes/pq".into(),
        kind: Kind::CephesPq,
        sites: Sites::Cephes,
        nterms: 0,
        mask_lim: 1u32 << 17,
        cut: None,
        note: "x87 Cephes erfce P/Q (ax<8) R/S (ax>=8) Horner+div stores".into(),
    });
    jobs.push(CubeJob {
        axis: "cody/n16".into(),
        kind: Kind::Cody,
        sites: Sites::Cephes,
        nterms: 0,
        mask_lim: 1u32 << 16,
        cut: None,
        note: "x87 Cody C/D Horner stores, no NSWC cut (global F)".into(),
    });
    jobs.push(CubeJob {
        axis: "cody/n17".into(),
        kind: Kind::Cody,
        sites: Sites::Cephes,
        nterms: 0,
        mask_lim: 1u32 << 17,
        cut: None,
        note: "x87 Cody C/D Horner + quotient store (17-bit global F)".into(),
    });
    for &(cut, tag) in &[
        (5.6, "joint/cody074/cephes/c56"),
        (4.9, "joint/cody074/cephes/c49"),
        (4.0, "joint/cody074/cephes/c40"),
    ] {
        jobs.push(CubeJob {
            axis: tag.into(),
            kind: Kind::JointCodyCephes,
            sites: Sites::Cephes,
            nterms: 0,
            mask_lim: 1u32 << 17,
            cut: Some(cut),
            note: format!("Cody 0x74 z<{cut} else x87 Cephes P/Q stores"),
        });
    }
    jobs.push(CubeJob {
        axis: "math77/erc2/n16".into(),
        kind: Kind::Math77Erc2,
        sites: Sites::ChebB0,
        nterms: 16,
        mask_lim: 1u32 << 16,
        cut: None,
        note: "x87 MATH77 dcsevl ERC2CS n=16 b0-store (ysq<=4)".into(),
    });
    jobs.push(CubeJob {
        axis: "math77/erfcc/n16".into(),
        kind: Kind::Math77Erfcc,
        sites: Sites::ChebB0,
        nterms: 16,
        mask_lim: 1u32 << 16,
        cut: None,
        note: "x87 MATH77 dcsevl ERFCCS n=16 b0-store (ysq>4)".into(),
    });
    for &(cut, tag) in &[
        (1.0, "pmid/cephes/c10"),
        (1.6, "pmid/cephes/c16"),
        (1.5, "pmid/cephes/c15"),
        (2.0, "pmid/cephes/c20"),
        (4.9, "ptail/cephes/c49"),
        (4.0, "p4/cephes/c40"),
        (5.6, "ptail/cephes/c56"),
    ] {
        jobs.push(CubeJob {
            axis: tag.into(),
            kind: Kind::CephesPq,
            sites: Sites::Cephes,
            nterms: 0,
            mask_lim: 1u32 << 17,
            cut: Some(cut),
            note: format!("NSWC z<{cut} else x87 Cephes P/Q Horner+div stores"),
        });
    }
    for &(cut, tag) in &[
        (1.0, "pmid/cody/c10"),
        (4.9, "ptail/cody/c49"),
    ] {
        jobs.push(CubeJob {
            axis: tag.into(),
            kind: Kind::Cody,
            sites: Sites::Cephes,
            nterms: 0,
            mask_lim: 1u32 << 16,
            cut: Some(cut),
            note: format!("NSWC z<{cut} else x87 Cody C/D Horner stores"),
        });
    }
    for &(n, cut, tag) in &[
        (16u32, 1.6, "pmid/as714/n16/c16"),
        (16, 5.6, "ptail/as714/n16/c56"),
        (21, 1.6, "pmid/as714/n21/c16"),
        (21, 5.6, "ptail/as714/n21/c56"),
        (21, 4.9, "ptail/as714/n21/c49"),
        (21, 6.2, "ptail/as714/n21/c62"),
    ] {
        jobs.push(CubeJob {
            axis: tag.into(),
            kind: Kind::BackAs714,
            sites: Sites::Den,
            nterms: n,
            mask_lim: 1u32 << n.min(28),
            cut: Some(cut),
            note: format!("NSWC z<{cut} else x87 CF as714 n={n} den-store"),
        });
    }
    for &(sites, tag, n, bits_per, kind, kn) in &[
        (Sites::CFacc, "cfacc", 10u32, 2u32, Kind::LentzAs714, "as714"),
        (Sites::CFacc, "cfacc", 12, 2, Kind::LentzAs714, "as714"),
        (Sites::CDelta, "cdelt", 12, 2, Kind::LentzAs714, "as714"),
        (Sites::DFacc, "dfacc", 12, 2, Kind::LentzAs714, "as714"),
        (Sites::CDFacc, "cdfacc", 8, 3, Kind::LentzAs714, "as714"),
        (Sites::CFacc, "cfacc", 12, 2, Kind::LentzGaut, "gaut"),
        (Sites::CDFacc, "cdfacc", 8, 3, Kind::LentzGaut, "gaut"),
    ] {
        let bits = n * bits_per;
        if bits > 28 {
            continue;
        }
        jobs.push(CubeJob {
            axis: format!("{tag}/{kn}/n{n}"),
            kind,
            sites,
            nterms: n,
            mask_lim: 1u32 << bits,
            cut: None,
            note: format!("x87 lentz {kn} n={n} {}-bit {tag} stores", bits),
        });
    }
    jobs
}

fn spill(x: Ext80) -> Ext80 {
    ext_from_f64(ext_to_f64(&x, CW_PC64_RN))
}

fn maybe_store(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        spill(x)
    } else {
        x
    }
}

fn lentz_as714_sites(x: f64, nterms: u32, mask: u32, sites: Sites) -> f64 {
    let one = ext_from_f64(1.0);
    let a_scale = ext_from_f64(0.5 / (x * x));
    let mut facc = one;
    let mut c = one;
    let mut d = ext_from_f64(0.0);
    let mut bit = 0u32;
    for j in 1..=nterms {
        let a = ext_mul(&ext_from_f64(j as f64), &a_scale, CW_PC64_RN);
        let den = ext_add(&one, &ext_mul(&a, &d, CW_PC64_RN), CW_PC64_RN);
        d = ext_div(&one, &den, CW_PC64_RN);
        c = ext_add(&one, &ext_div(&a, &c, CW_PC64_RN), CW_PC64_RN);
        match sites {
            Sites::CFacc => {
                c = maybe_store(c, mask, bit);
                bit += 1;
                facc = ext_mul(&facc, &ext_mul(&c, &d, CW_PC64_RN), CW_PC64_RN);
                facc = maybe_store(facc, mask, bit);
                bit += 1;
            }
            Sites::DFacc => {
                d = maybe_store(d, mask, bit);
                bit += 1;
                facc = ext_mul(&facc, &ext_mul(&c, &d, CW_PC64_RN), CW_PC64_RN);
                facc = maybe_store(facc, mask, bit);
                bit += 1;
            }
            Sites::CDelta => {
                c = maybe_store(c, mask, bit);
                bit += 1;
                let mut delta = ext_mul(&c, &d, CW_PC64_RN);
                delta = maybe_store(delta, mask, bit);
                bit += 1;
                facc = ext_mul(&facc, &delta, CW_PC64_RN);
            }
            Sites::CDFacc => {
                d = maybe_store(d, mask, bit);
                bit += 1;
                c = maybe_store(c, mask, bit);
                bit += 1;
                facc = ext_mul(&facc, &ext_mul(&c, &d, CW_PC64_RN), CW_PC64_RN);
                facc = maybe_store(facc, mask, bit);
                bit += 1;
            }
            _ => unreachable!(),
        }
    }
    f::RPINV / x / ext_to_f64(&facc, CW_PC64_RN)
}

fn lentz_gaut_sites(x: f64, nterms: u32, mask: u32, sites: Sites) -> f64 {
    let xe = ext_from_f64(x);
    let one = ext_from_f64(1.0);
    let mut facc = xe;
    let mut c = xe;
    let mut d = ext_from_f64(0.0);
    let mut bit = 0u32;
    for j in 1..=nterms {
        let a = ext_from_f64(j as f64 * 0.5);
        let den = ext_add(&xe, &ext_mul(&a, &d, CW_PC64_RN), CW_PC64_RN);
        d = ext_div(&one, &den, CW_PC64_RN);
        c = ext_add(&xe, &ext_div(&a, &c, CW_PC64_RN), CW_PC64_RN);
        match sites {
            Sites::CFacc => {
                c = maybe_store(c, mask, bit);
                bit += 1;
                facc = ext_mul(&facc, &ext_mul(&c, &d, CW_PC64_RN), CW_PC64_RN);
                facc = maybe_store(facc, mask, bit);
                bit += 1;
            }
            Sites::CDFacc => {
                d = maybe_store(d, mask, bit);
                bit += 1;
                c = maybe_store(c, mask, bit);
                bit += 1;
                facc = ext_mul(&facc, &ext_mul(&c, &d, CW_PC64_RN), CW_PC64_RN);
                facc = maybe_store(facc, mask, bit);
                bit += 1;
            }
            _ => {
                d = maybe_store(d, mask, bit);
                bit += 1;
                facc = ext_mul(&facc, &ext_mul(&c, &d, CW_PC64_RN), CW_PC64_RN);
                facc = maybe_store(facc, mask, bit);
                bit += 1;
            }
        }
    }
    f::RPINV / ext_to_f64(&facc, CW_PC64_RN)
}

fn back_as714_mask(x: f64, nterms: u32, mask: u32) -> f64 {
    let one = ext_from_f64(1.0);
    let a_scale = ext_from_f64(0.5 / (x * x));
    let mut den = one;
    let mut bit = 0u32;
    for k in (1..=nterms).rev() {
        let a = ext_mul(&ext_from_f64(k as f64), &a_scale, CW_PC64_RN);
        let q = ext_div(&a, &den, CW_PC64_RN);
        den = ext_add(&one, &q, CW_PC64_RN);
        den = maybe_store(den, mask, bit);
        bit += 1;
    }
    f::RPINV / x / ext_to_f64(&den, CW_PC64_RN)
}

fn polevl_mask(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ext_from_f64(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW_PC64_RN), &ext_from_f64(c), CW_PC64_RN);
        ans = maybe_store(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}

fn p1evl_mask(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ext_add(&x, &ext_from_f64(coef[0]), CW_PC64_RN);
    ans = maybe_store(ans, mask, bit);
    bit += 1;
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW_PC64_RN), &ext_from_f64(c), CW_PC64_RN);
        ans = maybe_store(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}

const CEPHES_P: [f64; 9] = [
    2.46196981473530512524e-10,
    5.64189564831068821977e-1,
    7.46321056442269912687e0,
    4.86371970985681366614e1,
    1.96520832956077098242e2,
    5.26445194995477358631e2,
    9.34528527171957607540e2,
    1.02755188689515710272e3,
    5.57535335369399327526e2,
];
const CEPHES_Q: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
const CEPHES_R: [f64; 6] = [
    5.64189583547755073984e-1,
    1.27536670759978104416e0,
    5.01905042251180477414e0,
    6.16021097993053585195e0,
    7.40974269950448939160e0,
    2.97886665372100240670e0,
];
const CEPHES_S: [f64; 6] = [
    2.26052863220117276590e0,
    9.39603524938001434673e0,
    1.20489539808096656605e1,
    1.70814450747565897222e1,
    9.60896809063285878198e0,
    3.36907645100081516050e0,
];

const CODY_C: [f64; 9] = [
    0.564188496988670089,
    8.88314979438837594,
    66.1191906371416295,
    298.635138197400131,
    881.95222124176909,
    1712.04761263407058,
    2051.07837782607147,
    1230.33935479799725,
    2.15311535474403846e-8,
];
const CODY_D: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];
const CODY_P: [f64; 6] = [
    0.305326634961232344,
    0.360344899949804439,
    0.125781726111229246,
    0.0160837851487422766,
    6.58749161529837803e-4,
    0.0163153871373020978,
];
const CODY_Q: [f64; 5] = [
    2.56852019228982242,
    1.87295284992346047,
    0.527905102951428412,
    0.0605183413124413191,
    0.00233520497626869185,
];

fn cody_mask(y: f64, mask: u32) -> f64 {
    let y = y.abs();
    let ye = ext_from_f64(y);
    if y <= 4.0 {
        let mut xnum = ext_mul(&ext_from_f64(CODY_C[8]), &ye, CW_PC64_RN);
        let mut xden = ye;
        xnum = maybe_store(xnum, mask, 0);
        xden = maybe_store(xden, mask, 1);
        for i in 0..7 {
            xnum = ext_mul(
                &ext_add(&xnum, &ext_from_f64(CODY_C[i]), CW_PC64_RN),
                &ye,
                CW_PC64_RN,
            );
            xden = ext_mul(
                &ext_add(&xden, &ext_from_f64(CODY_D[i]), CW_PC64_RN),
                &ye,
                CW_PC64_RN,
            );
            xnum = maybe_store(xnum, mask, 2 + 2 * i as u32);
            xden = maybe_store(xden, mask, 3 + 2 * i as u32);
        }
        let num = ext_add(&xnum, &ext_from_f64(CODY_C[7]), CW_PC64_RN);
        let den = ext_add(&xden, &ext_from_f64(CODY_D[7]), CW_PC64_RN);
        let mut q = ext_div(&num, &den, CW_PC64_RN);
        q = maybe_store(q, mask, 16);
        ext_to_f64(&q, CW_PC64_RN)
    } else {
        let ysq = ext_div(
            &ext_from_f64(1.0),
            &ext_mul(&ye, &ye, CW_PC64_RN),
            CW_PC64_RN,
        );
        let mut xnum = ext_mul(&ext_from_f64(CODY_P[5]), &ysq, CW_PC64_RN);
        let mut xden = ysq;
        for i in 0..4 {
            xnum = ext_mul(
                &ext_add(&xnum, &ext_from_f64(CODY_P[i]), CW_PC64_RN),
                &ysq,
                CW_PC64_RN,
            );
            xden = ext_mul(
                &ext_add(&xden, &ext_from_f64(CODY_Q[i]), CW_PC64_RN),
                &ysq,
                CW_PC64_RN,
            );
            xnum = maybe_store(xnum, mask, 2 * i as u32);
            xden = maybe_store(xden, mask, 1 + 2 * i as u32);
        }
        let num = ext_add(&xnum, &ext_from_f64(CODY_P[4]), CW_PC64_RN);
        let den = ext_add(&xden, &ext_from_f64(CODY_Q[4]), CW_PC64_RN);
        let r = ext_mul(&ysq, &ext_div(&num, &den, CW_PC64_RN), CW_PC64_RN);
        (f::RPINV - ext_to_f64(&r, CW_PC64_RN)) / y
    }
}

fn cephes_mask(x: f64, mask: u32) -> f64 {
    let ax = x.abs();
    if ax < 1.0 {
        return f::cephes_f(x);
    }
    let xe = ext_from_f64(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl_mask(xe, &CEPHES_P, mask, 0);
        let (q, b) = p1evl_mask(xe, &CEPHES_Q, mask, b);
        (p, q, b)
    } else {
        let (r, b) = polevl_mask(xe, &CEPHES_R, mask, 0);
        let (s, b) = p1evl_mask(xe, &CEPHES_S, mask, b);
        (r, s, b)
    };
    let mut q = ext_div(&num, &den, CW_PC64_RN);
    q = maybe_store(q, mask, bit0);
    ext_to_f64(&q, CW_PC64_RN)
}

fn dcsevl_mask(x: f64, cs: &[f64], n: usize, mask: u32) -> f64 {
    let twox = ext_add(&ext_from_f64(x), &ext_from_f64(x), CW_PC64_RN);
    let mut b2 = ext_from_f64(0.0);
    let mut b1 = ext_from_f64(0.0);
    let mut b0 = ext_from_f64(0.0);
    let mut bit = 0u32;
    for i in (0..n).rev() {
        b2 = b1;
        b1 = b0;
        b0 = ext_add(
            &ext_sub(&ext_mul(&twox, &b1, CW_PC64_RN), &b2, CW_PC64_RN),
            &ext_from_f64(cs[i]),
            CW_PC64_RN,
        );
        b0 = maybe_store(b0, mask, bit);
        bit += 1;
    }
    ext_to_f64(
        &ext_mul(
            &ext_from_f64(0.5),
            &ext_sub(&b0, &b2, CW_PC64_RN),
            CW_PC64_RN,
        ),
        CW_PC64_RN,
    )
}

fn horner_f64(cs: &[f64], x: f64) -> f64 {
    let mut acc = 0.0;
    for &c in cs.iter().rev() {
        acc = acc * x + c;
    }
    acc
}

fn math77_f_mask(y: f64, mask: u32, nterms: usize, erc2: bool) -> f64 {
    let y = y.abs();
    if y <= 0.5 {
        return f64::NAN;
    }
    if y <= 1.0 {
        return horner_f64(&PS, y) / horner_f64(&QS, y);
    }
    let ysq = y * y;
    let cheb = if ysq <= 4.0 {
        if !erc2 {
            dcsevl_mask((8.0 / ysq - 5.0) / 3.0, &ERC2CS, ERC2CS.len().min(24), 0)
        } else {
            dcsevl_mask(
                (8.0 / ysq - 5.0) / 3.0,
                &ERC2CS,
                nterms.min(ERC2CS.len()),
                mask,
            )
        }
    } else if erc2 {
        dcsevl_mask(8.0 / ysq - 1.0, &ERFCCS, ERFCCS.len().min(24), 0)
    } else {
        dcsevl_mask(8.0 / ysq - 1.0, &ERFCCS, nterms.min(ERFCCS.len()), mask)
    };
    cheb / y
}

fn lentz_as714_named(x: f64, nterms: u32, mask: u32) -> f64 {
    // leftover-compatible: D then C when two_bit, else C only.
    let one = ext_from_f64(1.0);
    let a_scale = ext_from_f64(0.5 / (x * x));
    let mut facc = one;
    let mut c = one;
    let mut d = ext_from_f64(0.0);
    let mut bit = 0u32;
    let two_bit = false;
    for j in 1..=nterms {
        let a = ext_mul(&ext_from_f64(j as f64), &a_scale, CW_PC64_RN);
        let den = ext_add(&one, &ext_mul(&a, &d, CW_PC64_RN), CW_PC64_RN);
        d = ext_div(&one, &den, CW_PC64_RN);
        if two_bit {
            d = maybe_store(d, mask, bit);
            bit += 1;
        }
        c = ext_add(&one, &ext_div(&a, &c, CW_PC64_RN), CW_PC64_RN);
        c = maybe_store(c, mask, bit);
        bit += 1;
        facc = ext_mul(&facc, &ext_mul(&c, &d, CW_PC64_RN), CW_PC64_RN);
    }
    f::RPINV / x / ext_to_f64(&facc, CW_PC64_RN)
}

fn lentz_gaut_named(x: f64, nterms: u32, mask: u32) -> f64 {
    let xe = ext_from_f64(x);
    let one = ext_from_f64(1.0);
    let mut facc = xe;
    let mut c = xe;
    let mut d = ext_from_f64(0.0);
    let mut bit = 0u32;
    for j in 1..=nterms {
        let a = ext_from_f64(j as f64 * 0.5);
        let den = ext_add(&xe, &ext_mul(&a, &d, CW_PC64_RN), CW_PC64_RN);
        d = ext_div(&one, &den, CW_PC64_RN);
        c = ext_add(&xe, &ext_div(&a, &c, CW_PC64_RN), CW_PC64_RN);
        c = maybe_store(c, mask, bit);
        bit += 1;
        facc = ext_mul(&facc, &ext_mul(&c, &d, CW_PC64_RN), CW_PC64_RN);
    }
    f::RPINV / ext_to_f64(&facc, CW_PC64_RN)
}

fn eval_job(j: &CubeJob, mask: u32, z: f64) -> f64 {
    if let Kind::JointCodyCephes = j.kind {
        if let Some(cut) = j.cut {
            if z < cut {
                return cody_mask(z, 0x0074);
            }
        }
        return cephes_mask(z, mask);
    }
    if let Some(cut) = j.cut {
        if z < cut {
            return f::nswc_derfc0(z);
        }
    }
    match j.kind {
        Kind::LentzAs714 => lentz_as714_sites(z, j.nterms, mask, j.sites),
        Kind::LentzGaut => lentz_gaut_sites(z, j.nterms, mask, j.sites),
        Kind::BackAs714 => back_as714_mask(z, j.nterms, mask),
        Kind::CephesPq => cephes_mask(z, mask),
        Kind::Cody => cody_mask(z, mask),
        Kind::JointCodyCephes => unreachable!(),
        Kind::Math77Erc2 => math77_f_mask(z, mask, j.nterms as usize, true),
        Kind::Math77Erfcc => math77_f_mask(z, mask, j.nterms as usize, false),
    }
}

#[derive(Clone, Copy, Default)]
struct Split {
    m_mid: usize,
    m_tail: usize,
    d_mid: usize,
    d_tail: usize,
    i_mid: usize,
    i_tail: usize,
}

struct Row {
    z: f64,
    fo: f64,
    mid: bool,
    direct: bool,
}

fn prep_rows(dir: &str) -> Vec<Row> {
    let mut out = Vec::new();
    for r in f::load_q_rows_tagged(dir) {
        if r.z < 0.5 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        out.push(Row {
            z: r.z,
            fo,
            mid: r.z < 4.0,
            direct: r.direct,
        });
    }
    out
}

fn score_rows(rows: &[Row], eval: impl Fn(f64) -> f64) -> Split {
    let mut s = Split::default();
    for r in rows {
        let fg = eval(r.z);
        if !fg.is_finite() {
            continue;
        }
        let d = ulp_distance(fg, r.fo).unwrap_or(u64::MAX);
        if d > ULP_CAP || d != 0 {
            continue;
        }
        if r.mid {
            s.m_mid += 1;
            if r.direct {
                s.d_mid += 1;
            } else {
                s.i_mid += 1;
            }
        } else {
            s.m_tail += 1;
            if r.direct {
                s.d_tail += 1;
            } else {
                s.i_tail += 1;
            }
        }
    }
    s
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct Checkpoint {
    #[serde(default)]
    progress: BTreeMap<String, u32>,
    #[serde(default)]
    best_mid_exact: usize,
    #[serde(default)]
    best_tail_exact: usize,
    #[serde(default)]
    best_mid_label: String,
    #[serde(default)]
    best_tail_label: String,
    #[serde(default)]
    best_direct_mid: usize,
    #[serde(default)]
    best_direct_tail: usize,
    #[serde(default)]
    best_direct_mid_label: String,
    #[serde(default)]
    best_direct_tail_label: String,
    #[serde(default)]
    direct_mid_bar: usize,
    #[serde(default)]
    direct_tail_bar: usize,
    #[serde(default)]
    configs_done: u64,
    #[serde(default)]
    started_unix: u64,
}

#[derive(Serialize, Deserialize)]
struct StatusJson {
    region: String,
    chunk: String,
    configs_done: u64,
    best_mid_exact: usize,
    best_tail_exact: usize,
    best_mid_label: String,
    best_tail_label: String,
    best_direct_mid: usize,
    best_direct_tail: usize,
    best_direct_mid_label: String,
    best_direct_tail_label: String,
    named_mid_bar: usize,
    named_tail_bar: usize,
    direct_mid_bar: usize,
    direct_tail_bar: usize,
    runtime_secs: u64,
    max_hours: f64,
    threads: usize,
    stop_requested: bool,
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
fn write_atomic(path: &Path, text: &str) {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, text).unwrap();
    fs::rename(&tmp, path).unwrap();
}
fn wall_secs(ck: &Checkpoint) -> u64 {
    now_unix().saturating_sub(ck.started_unix)
}
fn timed_out(ck: &Checkpoint, max_hours: f64) -> bool {
    wall_secs(ck) as f64 / 3600.0 >= max_hours
}
fn stop_requested(out: &Path) -> bool {
    out.join("STOP").exists()
}

fn load_ckpt(out: &Path) -> Checkpoint {
    let p = out.join("checkpoint.json");
    if let Ok(t) = fs::read_to_string(&p) {
        if let Ok(c) = serde_json::from_str::<Checkpoint>(&t) {
            return c;
        }
    }
    Checkpoint {
        started_unix: now_unix(),
        ..Checkpoint::default()
    }
}

fn consider(ck: &mut Checkpoint, out: &Path, label: &str, s: Split) {
    if s.m_mid > ck.best_mid_exact {
        ck.best_mid_exact = s.m_mid;
        ck.best_mid_label = label.to_string();
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(out.join("leaders.jsonl"))
            .and_then(|mut f| {
                writeln!(
                    f,
                    "{{\"kind\":\"mid\",\"mid\":{},\"tail\":{},\"dmid\":{},\"dtail\":{},\"label\":{label:?}}}",
                    s.m_mid, s.m_tail, s.d_mid, s.d_tail
                )
            });
        if s.m_mid > MID_BAR {
            let _ = fs::write(
                out.join("HIT_MID"),
                format!("{label} mid={} bar={MID_BAR}\n", s.m_mid),
            );
        }
    }
    if s.m_tail > ck.best_tail_exact {
        ck.best_tail_exact = s.m_tail;
        ck.best_tail_label = label.to_string();
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(out.join("leaders.jsonl"))
            .and_then(|mut f| {
                writeln!(
                    f,
                    "{{\"kind\":\"tail\",\"mid\":{},\"tail\":{},\"dmid\":{},\"dtail\":{},\"label\":{label:?}}}",
                    s.m_mid, s.m_tail, s.d_mid, s.d_tail
                )
            });
        if s.m_tail > TAIL_BAR {
            let _ = fs::write(
                out.join("HIT_TAIL"),
                format!("{label} tail={} bar={TAIL_BAR}\n", s.m_tail),
            );
        }
    }
    if s.d_mid > ck.best_direct_mid {
        ck.best_direct_mid = s.d_mid;
        ck.best_direct_mid_label = label.to_string();
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(out.join("leaders.jsonl"))
            .and_then(|mut f| {
                writeln!(
                    f,
                    "{{\"kind\":\"direct_mid\",\"mid\":{},\"tail\":{},\"dmid\":{},\"dtail\":{},\"label\":{label:?}}}",
                    s.m_mid, s.m_tail, s.d_mid, s.d_tail
                )
            });
        if ck.direct_mid_bar > 0 && s.d_mid > ck.direct_mid_bar {
            let _ = fs::write(
                out.join("HIT_DIRECT_MID"),
                format!(
                    "{label} dmid={} bar={}\n",
                    s.d_mid, ck.direct_mid_bar
                ),
            );
        }
    }
    if s.d_tail > ck.best_direct_tail {
        ck.best_direct_tail = s.d_tail;
        ck.best_direct_tail_label = label.to_string();
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(out.join("leaders.jsonl"))
            .and_then(|mut f| {
                writeln!(
                    f,
                    "{{\"kind\":\"direct_tail\",\"mid\":{},\"tail\":{},\"dmid\":{},\"dtail\":{},\"label\":{label:?}}}",
                    s.m_mid, s.m_tail, s.d_mid, s.d_tail
                )
            });
        if ck.direct_tail_bar > 0 && s.d_tail > ck.direct_tail_bar {
            let _ = fs::write(
                out.join("HIT_DIRECT_TAIL"),
                format!(
                    "{label} dtail={} bar={}\n",
                    s.d_tail, ck.direct_tail_bar
                ),
            );
        }
    }
}

fn write_status(
    out: &Path,
    ck: &Checkpoint,
    jobs: &[CubeJob],
    region: &str,
    chunk: &str,
    max_hours: f64,
    threads: usize,
    extra: &str,
) {
    let hours = wall_secs(ck) as f64 / 3600.0;
    let mut map = String::from(
        "# Mixed F campaign REGION_MAP\n\n| id | space | next | note |\n|---|---|---|---|\n",
    );
    for j in jobs {
        let v = ck.progress.get(&j.axis).copied().unwrap_or(0);
        let next = if v >= j.mask_lim {
            "done".into()
        } else {
            format!("0x{v:07x}/0x{:07x}", j.mask_lim)
        };
        map.push_str(&format!(
            "| {} | 0..0x{:07x} | {next} | {} |\n",
            j.axis, j.mask_lim, j.note
        ));
    }
    map.push_str(&format!(
        "\nbest_mid {} / bar {MID_BAR} `{}`\nbest_tail {} / bar {TAIL_BAR} `{}`\nbest_direct_mid {} / bar {} `{}`\nbest_direct_tail {} / bar {} `{}`\n",
        ck.best_mid_exact,
        ck.best_mid_label,
        ck.best_tail_exact,
        ck.best_tail_label,
        ck.best_direct_mid,
        ck.direct_mid_bar,
        ck.best_direct_mid_label,
        ck.best_direct_tail,
        ck.direct_tail_bar,
        ck.best_direct_tail_label
    ));
    let md = format!(
        "# Mixed F campaign STATUS\n\n\
         - region: `{region}`\n- chunk: `{chunk}`\n- configs_done: {}\n\
         - best_mid: {} / bar {MID_BAR} `{}`\n\
         - best_tail: {} / bar {TAIL_BAR} `{}`\n\
         - best_direct_mid: {} / bar {} `{}`\n\
         - best_direct_tail: {} / bar {} `{}`\n\
         - runtime_hours: {hours:.2} / {max_hours}\n- threads: {threads}\n\
         - stop_requested: {}\n\n{extra}\n\n{map}\n",
        ck.configs_done,
        ck.best_mid_exact,
        ck.best_mid_label,
        ck.best_tail_exact,
        ck.best_tail_label,
        ck.best_direct_mid,
        ck.direct_mid_bar,
        ck.best_direct_mid_label,
        ck.best_direct_tail,
        ck.direct_tail_bar,
        ck.best_direct_tail_label,
        stop_requested(out)
    );
    write_atomic(&out.join("STATUS.md"), &md);
    write_atomic(&out.join("REGION_MAP.md"), &map);
    let sj = StatusJson {
        region: region.into(),
        chunk: chunk.into(),
        configs_done: ck.configs_done,
        best_mid_exact: ck.best_mid_exact,
        best_tail_exact: ck.best_tail_exact,
        best_mid_label: ck.best_mid_label.clone(),
        best_tail_label: ck.best_tail_label.clone(),
        best_direct_mid: ck.best_direct_mid,
        best_direct_tail: ck.best_direct_tail,
        best_direct_mid_label: ck.best_direct_mid_label.clone(),
        best_direct_tail_label: ck.best_direct_tail_label.clone(),
        named_mid_bar: MID_BAR,
        named_tail_bar: TAIL_BAR,
        direct_mid_bar: ck.direct_mid_bar,
        direct_tail_bar: ck.direct_tail_bar,
        runtime_secs: wall_secs(ck),
        max_hours,
        threads,
        stop_requested: stop_requested(out),
    };
    write_atomic(
        &out.join("status.json"),
        &serde_json::to_string_pretty(&sj).unwrap(),
    );
    write_atomic(
        &out.join("checkpoint.json"),
        &serde_json::to_string_pretty(ck).unwrap(),
    );
}

fn parse_args() -> (String, PathBuf, usize, f64, Vec<String>) {
    let mut dir = "../../work/w109/G3-01-dist".into();
    let mut out = PathBuf::from("../../work/w109/erfc-mixed-campaign");
    let mut threads = 10usize;
    let mut max_hours = 14.0;
    let mut only = Vec::new();
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--dir" => dir = it.next().expect("--dir"),
            "--out" => out = PathBuf::from(it.next().expect("--out")),
            "--threads" => threads = it.next().unwrap().parse().unwrap(),
            "--max-hours" => max_hours = it.next().unwrap().parse().unwrap(),
            "--only" => {
                only = it
                    .next()
                    .expect("--only")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            _ => {}
        }
    }
    assert!(!dir.contains("heldout"));
    (dir, out, threads, max_hours, only)
}

fn selected(axis: &str, only: &[String]) -> bool {
    only.is_empty()
        || only
            .iter()
            .any(|p| axis == p || axis.starts_with(&format!("{p}/")) || axis.starts_with(p))
}

fn fmt_split(s: &Split) -> String {
    format!(
        "mid {} tail {}  direct mid {} tail {}  implied mid {} tail {}",
        s.m_mid, s.m_tail, s.d_mid, s.d_tail, s.i_mid, s.i_tail
    )
}

fn main() {
    let (dir, out, threads, max_hours, only) = parse_args();
    fs::create_dir_all(&out).unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .ok();
    let rows = prep_rows(&dir);
    let n_mid = rows.iter().filter(|r| r.mid).count();
    let n_tail = rows.len() - n_mid;
    let n_dmid = rows.iter().filter(|r| r.mid && r.direct).count();
    let n_dtail = rows.iter().filter(|r| !r.mid && r.direct).count();
    let jobs = cube_jobs();
    let mut ck = load_ckpt(&out);
    if ck.started_unix == 0 {
        ck.started_unix = now_unix();
    }

    if ck.progress.get("R0").copied().unwrap_or(0) == 0 && selected("R0", &only) {
        let mut extra = format!(
            "## R0 split corpus\n\nrows={} mid={} tail={} direct_mid={} direct_tail={}\n\n## named F under w_rn53\n\n",
            rows.len(),
            n_mid,
            n_tail,
            n_dmid,
            n_dtail
        );
        let named: [(&str, fn(f64) -> f64); 11] = [
            ("nswc_derfc0", f::nswc_derfc0),
            ("cody_erfcx", f::cody_erfcx_f),
            ("cephes_erfce", f::cephes_f),
            ("cdflib_erfc1", f::cdflib_erfc1_f),
            ("cf_as714_n80", f::cf_as714_f),
            ("cf_gautschi_n80", f::cf_gautschi_f),
            ("lentz_as714_n12", |z| f::cf_lentz_as714_n(z, 12)),
            ("lentz_as714_n21", |z| f::cf_lentz_as714_n(z, 21)),
            ("lentz_as714_n24", |z| f::cf_lentz_as714_n(z, 24)),
            ("evenodd_as714_n12", |z| f::cf_evenodd_as714_n(z, 12)),
            ("math77_f_n16e", |z| math77_f_mask(z, 0, 16, true)),
        ];
        for (name, eval) in named {
            let s = score_rows(&rows, eval);
            extra.push_str(&format!("- {name}: {}\n", fmt_split(&s)));
            consider(&mut ck, &out, name, s);
        }
        extra.push_str("\n## piecewise NSWC / CF-as714-n80\n");
        for k in 0..=80 {
            let cut = 0.5 + k as f64 * 0.1;
            if cut > 8.5 {
                break;
            }
            let s = score_rows(&rows, |z| {
                if z < cut {
                    f::nswc_derfc0(z)
                } else {
                    f::cf_as714_f(z)
                }
            });
            extra.push_str(&format!("- cut={cut:.1}: {}\n", fmt_split(&s)));
            consider(&mut ck, &out, &format!("piece/cut={cut:.1}"), s);
        }
        extra.push_str("\n## compose NSWC + prior HIT_TAIL graphs\n");
        for cut in [1.5, 1.6, 2.0, 4.0, 4.9, 5.5, 5.6, 6.2] {
            let s = score_rows(&rows, |z| {
                if z < cut {
                    f::nswc_derfc0(z)
                } else {
                    lentz_as714_named(z, 24, 0x0400000)
                }
            });
            extra.push_str(&format!(
                "- nswc+lentz_as714_n24/0400000 cut={cut}: {}\n",
                fmt_split(&s)
            ));
            consider(
                &mut ck,
                &out,
                &format!("compose/lentz24hit/cut={cut}"),
                s,
            );
            let s = score_rows(&rows, |z| {
                if z < cut {
                    f::nswc_derfc0(z)
                } else {
                    lentz_gaut_named(z, 21, 0x0142180)
                }
            });
            extra.push_str(&format!(
                "- nswc+lentz_gaut_n21/0142180 cut={cut}: {}\n",
                fmt_split(&s)
            ));
            consider(
                &mut ck,
                &out,
                &format!("compose/lentzgaut21hit/cut={cut}"),
                s,
            );
        }
        extra.push_str("\n## mixed-site mask=0 baselines\n");
        for job in &jobs {
            if job.mask_lim > 1 {
                let s = score_rows(&rows, |z| eval_job(job, 0, z));
                extra.push_str(&format!("- {} /mask=0: {}\n", job.axis, fmt_split(&s)));
                consider(&mut ck, &out, &format!("{} /mask=0000000", job.axis), s);
            }
        }
        ck.direct_mid_bar = ck.best_direct_mid;
        ck.direct_tail_bar = ck.best_direct_tail;
        extra.push_str(&format!(
            "\n## R0 bars locked\nmerged mid {MID_BAR} tail {TAIL_BAR}\ndirect mid {} tail {}\n",
            ck.direct_mid_bar, ck.direct_tail_bar
        ));
        write_atomic(&out.join("R0.md"), &extra);
        ck.progress.insert("R0".into(), 1);
        write_status(&out, &ck, &jobs, "R0", "done", max_hours, threads, &extra);
    }

    if ck.progress.get("compose/cephes").copied().unwrap_or(0) == 0
        && ck.progress.get("cephes/pq").copied().unwrap_or(0) >= (1u32 << 17)
        && selected("compose/cephes", &only)
    {
        let mut extra = String::from("## compose NSWC + Cephes leader masks\n\n");
        let mut masks: Vec<u32> = vec![0, 0x0005, 0x0509, 0x5005, 0x0e82, 0x24a5, 0x0a69, 0x4805];
        if let Ok(t) = fs::read_to_string(out.join("leaders.jsonl")) {
            for line in t.lines() {
                if !line.contains("cephes/pq") {
                    continue;
                }
                if let Some(i) = line.find("mask=") {
                    let hex: String = line[i + 5..]
                        .chars()
                        .take_while(|c| c.is_ascii_hexdigit())
                        .collect();
                    if let Ok(v) = u32::from_str_radix(&hex, 16) {
                        masks.push(v);
                    }
                }
            }
        }
        masks.sort_unstable();
        masks.dedup();
        extra.push_str(&format!("masks: {masks:x?}\n"));
        for &mask in &masks {
            for k in 0..=80 {
                let cut = 0.5 + k as f64 * 0.1;
                if cut > 8.5 {
                    break;
                }
                let s = score_rows(&rows, |z| {
                    if z < cut {
                        f::nswc_derfc0(z)
                    } else {
                        cephes_mask(z, mask)
                    }
                });
                extra.push_str(&format!(
                    "- nswc+cephes/mask={mask:07x} cut={cut:.1}: {}\n",
                    fmt_split(&s)
                ));
                consider(
                    &mut ck,
                    &out,
                    &format!("compose/cephes/{mask:07x}/cut={cut:.1}"),
                    s,
                );
            }
            let s = score_rows(&rows, |z| {
                if z < 4.0 {
                    cephes_mask(z, 0x0509)
                } else {
                    cephes_mask(z, mask)
                }
            });
            extra.push_str(&format!(
                "- split mid0509 + tail {mask:07x}: {}\n",
                fmt_split(&s)
            ));
            consider(
                &mut ck,
                &out,
                &format!("compose/cephes/split0509+{mask:07x}"),
                s,
            );
        }
        write_atomic(&out.join("COMPOSE_CEPHES.md"), &extra);
        ck.progress.insert("compose/cephes".into(), 1);
        write_status(
            &out,
            &ck,
            &jobs,
            "compose/cephes",
            "done",
            max_hours,
            threads,
            &extra,
        );
    }

    if ck.progress.get("compose/three").copied().unwrap_or(0) == 0
        && ck.progress.get("compose/cephes").copied().unwrap_or(0) >= 1
        && selected("compose/three", &only)
    {
        let mut extra = String::from("## three-piece NSWC + Cephes mid-mask + Cephes tail-mask\n\n");
        let mids = [0x0509u32, 0x0409, 0x000d, 0x0005, 0x0e82];
        let tails = [0x5005u32, 0x0a69, 0x4805, 0x0805, 0x24a5];
        let low_cuts = [0.9f64, 1.0, 1.1, 1.6];
        let high_cuts = [4.0f64, 4.8, 4.9, 5.6, 6.0];
        for &lo in &low_cuts {
            for &hi in &high_cuts {
                if hi <= lo {
                    continue;
                }
                for &mm in &mids {
                    for &tm in &tails {
                        let s = score_rows(&rows, |z| {
                            if z < lo {
                                f::nswc_derfc0(z)
                            } else if z < hi {
                                cephes_mask(z, mm)
                            } else {
                                cephes_mask(z, tm)
                            }
                        });
                        extra.push_str(&format!(
                            "- lo={lo:.1} hi={hi:.1} mid={mm:07x} tail={tm:07x}: {}\n",
                            fmt_split(&s)
                        ));
                        consider(
                            &mut ck,
                            &out,
                            &format!("three/lo{lo:.1}/hi{hi:.1}/m{mm:07x}/t{tm:07x}"),
                            s,
                        );
                    }
                }
            }
        }
        write_atomic(&out.join("COMPOSE_THREE.md"), &extra);
        ck.progress.insert("compose/three".into(), 1);
        write_status(
            &out,
            &ck,
            &jobs,
            "compose/three",
            "done",
            max_hours,
            threads,
            &extra,
        );
    }

    if ck.progress.get("compose/cody").copied().unwrap_or(0) == 0
        && ck.progress.get("pmid/cody/c10").copied().unwrap_or(0) >= (1u32 << 16)
        && selected("compose/cody", &only)
    {
        let mut extra = String::from("## compose NSWC + Cody leader masks\n\n");
        let mut masks: Vec<u32> = vec![0, 0x0074, 0x0210];
        if let Ok(t) = fs::read_to_string(out.join("leaders.jsonl")) {
            for line in t.lines() {
                if !line.contains("cody") {
                    continue;
                }
                if let Some(i) = line.find("mask=") {
                    let hex: String = line[i + 5..]
                        .chars()
                        .take_while(|c| c.is_ascii_hexdigit())
                        .collect();
                    if let Ok(v) = u32::from_str_radix(&hex, 16) {
                        masks.push(v);
                    }
                }
            }
        }
        masks.sort_unstable();
        masks.dedup();
        extra.push_str(&format!("masks: {masks:x?}\n"));
        for &mask in &masks {
            for k in 0..=80 {
                let cut = 0.5 + k as f64 * 0.1;
                if cut > 8.5 {
                    break;
                }
                let s = score_rows(&rows, |z| {
                    if z < cut {
                        f::nswc_derfc0(z)
                    } else {
                        cody_mask(z, mask)
                    }
                });
                extra.push_str(&format!(
                    "- nswc+cody/mask={mask:07x} cut={cut:.1}: {}\n",
                    fmt_split(&s)
                ));
                consider(
                    &mut ck,
                    &out,
                    &format!("compose/cody/{mask:07x}/cut={cut:.1}"),
                    s,
                );
            }
            let s = score_rows(&rows, |z| {
                if z < 1.0 {
                    f::nswc_derfc0(z)
                } else if z < 4.9 {
                    cody_mask(z, 0x0074)
                } else {
                    cephes_mask(z, mask)
                }
            });
            extra.push_str(&format!(
                "- nswc+cody0074 + cephes {mask:07x} @4.9: {}\n",
                fmt_split(&s)
            ));
            consider(
                &mut ck,
                &out,
                &format!("compose/cody0074+cephes{mask:07x}"),
                s,
            );
        }
        write_atomic(&out.join("COMPOSE_CODY.md"), &extra);
        ck.progress.insert("compose/cody".into(), 1);
        write_status(
            &out,
            &ck,
            &jobs,
            "compose/cody",
            "done",
            max_hours,
            threads,
            &extra,
        );
    }

    if ck.progress.get("compose/joint").copied().unwrap_or(0) == 0
        && ck.progress.get("compose/cody").copied().unwrap_or(0) >= 1
        && selected("compose/joint", &only)
    {
        let mut extra = String::from("## joint Cody-mid + Cephes-tail\n\n");
        let cody_mids = [0x0074u32, 0x0070, 0x0064, 0x005c, 0x0210];
        let cephes_tails = [0x5005u32, 0x4805, 0x0a69, 0x0805, 0x24a5];
        let cuts = [4.0f64, 4.6, 4.9, 5.6];
        for &cm in &cody_mids {
            for &ct in &cephes_tails {
                for &cut in &cuts {
                    let s = score_rows(&rows, |z| {
                        if z < cut {
                            cody_mask(z, cm)
                        } else {
                            cephes_mask(z, ct)
                        }
                    });
                    extra.push_str(&format!(
                        "- cody {cm:07x} / cephes {ct:07x} cut={cut:.1}: {}\n",
                        fmt_split(&s)
                    ));
                    consider(
                        &mut ck,
                        &out,
                        &format!("joint/cody{cm:07x}/cephes{ct:07x}/cut={cut:.1}"),
                        s,
                    );
                }
            }
        }
        write_atomic(&out.join("COMPOSE_JOINT.md"), &extra);
        ck.progress.insert("compose/joint".into(), 1);
        write_status(
            &out,
            &ck,
            &jobs,
            "compose/joint",
            "done",
            max_hours,
            threads,
            &extra,
        );
    }

    for job in &jobs {
        if timed_out(&ck, max_hours) || stop_requested(&out) {
            break;
        }
        if !selected(&job.axis, &only) {
            continue;
        }
        let mut c0 = ck.progress.get(&job.axis).copied().unwrap_or(0);
        while c0 < job.mask_lim {
            if timed_out(&ck, max_hours) || stop_requested(&out) {
                write_status(
                    &out,
                    &ck,
                    &jobs,
                    &job.axis,
                    &format!("stop@{c0:07x}"),
                    max_hours,
                    threads,
                    &job.note,
                );
                return;
            }
            let c1 = (c0 + CHUNK).min(job.mask_lim);
            let scores: Vec<(u32, Split)> = (c0..c1)
                .into_par_iter()
                .map(|mask| {
                    let s = score_rows(&rows, |z| eval_job(job, mask, z));
                    (mask, s)
                })
                .collect();
            ck.configs_done += scores.len() as u64;
            for (mask, s) in &scores {
                consider(
                    &mut ck,
                    &out,
                    &format!("{} /mask={mask:07x}", job.axis),
                    *s,
                );
            }
            c0 = c1;
            ck.progress.insert(job.axis.clone(), c0);
            let chunk_best_tail = scores.iter().map(|s| s.1.m_tail).max().unwrap_or(0);
            let chunk_best_mid = scores.iter().map(|s| s.1.m_mid).max().unwrap_or(0);
            let chunk_best_dmid = scores.iter().map(|s| s.1.d_mid).max().unwrap_or(0);
            write_status(
                &out,
                &ck,
                &jobs,
                &job.axis,
                &format!("{c0:07x}"),
                max_hours,
                threads,
                &format!(
                    "{}\nchunk_best_mid={chunk_best_mid} tail={chunk_best_tail} dmid={chunk_best_dmid}",
                    job.note
                ),
            );
        }
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(out.join("ruled-out.jsonl"))
            .and_then(|mut f| {
                writeln!(
                    f,
                    "{{\"axis\":{},\"mask_lim\":{},\"best_mid\":{},\"best_tail\":{},\"best_dmid\":{},\"best_dtail\":{},\"unix\":{}}}",
                    serde_json::to_string(&job.axis).unwrap(),
                    job.mask_lim,
                    ck.best_mid_exact,
                    ck.best_tail_exact,
                    ck.best_direct_mid,
                    ck.best_direct_tail,
                    now_unix()
                )
            });
    }

    let exhausted = jobs
        .iter()
        .filter(|j| selected(&j.axis, &only))
        .all(|j| ck.progress.get(&j.axis).copied().unwrap_or(0) >= j.mask_lim);
    write_status(
        &out,
        &ck,
        &jobs,
        if exhausted { "exit-regions" } else { "exit" },
        "-",
        max_hours,
        threads,
        if exhausted {
            "selected mixed cubes finished"
        } else {
            "exiting (time/STOP). resume = same command"
        },
    );
}

const ERC2CS: [f64; 49] = [
    -0.6960134660230950112739150826197e-1,
    -0.4110133936262089348982212084666e-1,
    0.3914495866689626881561143705244e-2,
    -0.4906395650548979161280935450774e-3,
    0.7157479001377036380760894141825e-4,
    -0.1153071634131232833808232847912e-4,
    0.1994670590201997635052314867709e-5,
    -0.3642666471599222873936118430711e-6,
    0.6944372610005012589931277214633e-7,
    -0.1371220902104366019534605141210e-7,
    0.2788389661007137131963860348087e-8,
    -0.5814164724331161551864791050316e-9,
    0.1238920491752753181180168817950e-9,
    -0.2690639145306743432390424937889e-10,
    0.5942614350847910982444709683840e-11,
    -0.1332386735758119579287754420570e-11,
    0.3028046806177132017173697243304e-12,
    -0.6966648814941032588795867588954e-13,
    0.1620854541053922969812893227628e-13,
    -0.3809934465250491999876913057729e-14,
    0.9040487815978831149368971012975e-15,
    -0.2164006195089607347809812047003e-15,
    0.5222102233995854984607980244172e-16,
    -0.1269729602364555336372415527780e-16,
    0.3109145504276197583836227412951e-17,
    -0.7663762920320385524009566714811e-18,
    0.1900819251362745202536929733290e-18,
    -0.4742207279069039545225655999965e-19,
    0.1189649200076528382880683078451e-19,
    -0.3000035590325780256845271313066e-20,
    0.7602993453043246173019385277098e-21,
    -0.1935909447606872881569811049130e-21,
    0.4951399124773337881000042386773e-22,
    -0.1271807481336371879608621989888e-22,
    0.3280049600469513043315841652053e-23,
    -0.8492320176822896568924792422399e-24,
    0.2206917892807560223519879987199e-24,
    -0.5755617245696528498312819507199e-25,
    0.1506191533639234250354144051199e-25,
    -0.3954502959018796953104285695999e-26,
    0.1041529704151500979984645051733e-26,
    -0.2751487795278765079450178901333e-27,
    0.7290058205497557408997703680000e-28,
    -0.1936939645915947804077501098666e-28,
    0.5160357112051487298370054826666e-29,
    -0.1378419322193094099389644800000e-29,
    0.3691326793107069042251093333333e-30,
    -0.9909389590624365420653226666666e-31,
    0.2666491705195388413323946666666e-31,
];
const ERFCCS: [f64; 59] = [
    0.715179310202924774503697709496e-1,
    -0.265324343376067157558893386681e-1,
    0.171115397792085588332699194606e-2,
    -0.163751663458517884163746404749e-3,
    0.198712935005520364995974806758e-4,
    -0.284371241276655508750175183152e-5,
    0.460616130896313036969379968464e-6,
    -0.822775302587920842057766536366e-7,
    0.159214187277090112989358340826e-7,
    -0.329507136225284321486631665072e-8,
    0.722343976040055546581261153890e-9,
    -0.166485581339872959344695966886e-9,
    0.401039258823766482077671768814e-10,
    -0.100481621442573113272170176283e-10,
    0.260827591330033380859341009439e-11,
    -0.699111056040402486557697812476e-12,
    0.192949233326170708624205749803e-12,
    -0.547013118875433106490125085271e-13,
    0.158966330976269744839084032762e-13,
    -0.472689398019755483920369584290e-14,
    0.143587337678498478672873997840e-14,
    -0.444951056181735839417250062829e-15,
    0.140481088476823343737305537466e-15,
    -0.451381838776421089625963281623e-16,
    0.147452154104513307787018713262e-16,
    -0.489262140694577615436841552532e-17,
    0.164761214141064673895301522827e-17,
    -0.562681717632940809299928521323e-18,
    0.194744338223207851429197867821e-18,
    -0.682630564294842072956664144723e-19,
    0.242198888729864924018301125438e-19,
    -0.869341413350307042563800861857e-20,
    0.315518034622808557122363401262e-20,
    -0.115737232404960874261239486742e-20,
    0.428894716160565394623737097442e-21,
    -0.160503074205761685005737770964e-21,
    0.606329875745380264495069923027e-22,
    -0.231140425169795849098840801367e-22,
    0.888877854066188552554702955697e-23,
    -0.344726057665137652230718495566e-23,
    0.134786546020696506827582774181e-23,
    -0.531179407112502173645873201807e-24,
    0.210934105861978316828954734537e-24,
    -0.843836558792378911598133256738e-25,
    0.339998252494520890627359576337e-25,
    -0.137945238807324209002238377110e-25,
    0.563449031183325261513392634811e-26,
    -0.231649043447706544823427752700e-26,
    0.958446284460181015263158381226e-27,
    -0.399072288033010972624224850193e-27,
    0.167212922594447736017228709669e-27,
    -0.704599152276601385638803782587e-28,
    0.297976840286420635412357989444e-28,
    -0.126252246646061929722422632994e-28,
    0.539543870454248793985299653154e-29,
    -0.238099288253145918675346190062e-29,
    0.109905283010276157359726683750e-29,
    -0.486771374164496572732518677435e-30,
    0.152587726411035756763200828211e-30,
];
const PS: [f64; 9] = [
    1.00000000000036828,
    1.87051017604560834,
    1.74642369370058320,
    1.02438464807598001,
    4.07413180167223764e-1,
    1.11870870991098165e-1,
    2.07045775788719818e-2,
    2.37133372752999036e-3,
    1.29992515945788642e-4,
];
const QS: [f64; 10] = [
    1.00000000000000000,
    2.99888934314798253,
    4.13030795287321183,
    3.43830153103630866,
    1.91273588328781533,
    7.40352738163508723e-1,
    2.00387662412610424e-1,
    3.68131014202168126e-2,
    4.20307996290648223e-3,
    2.30405728794132537e-4,
];


