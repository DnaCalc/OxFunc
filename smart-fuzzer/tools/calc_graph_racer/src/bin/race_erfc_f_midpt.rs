//! Final-store midpoint / rounding-mode probe on the unspilled-x87 PQR 2939 graph.
//! Also Estrin R, fma Horner, AABB z-from-t. Heldouts unnamed. No landing.
//!
//!   cargo run --release --bin race_erfc_f_midpt -- ../../work/w109/G3-01-dist [out-dir]

use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN,
};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const P: [f64; 8] = [
    0.16506148041280876191828601e-03,
    0.15471455377139313353998665e-03,
    0.44852548090298868465196794e-04,
    -0.49177280017226285450486205e-05,
    -0.69353602078656412367801676e-05,
    -0.20508667787746282746857743e-05,
    -0.28982842617824971177267380e-06,
    -0.17272433544836633301127174e-07,
];
const Q: [f64; 8] = [
    1.0,
    0.16272656776533322859856317e+01,
    0.12040996037066026106794322e+01,
    0.52400246352158386907601472e+00,
    0.14497345252798672362384241e+00,
    0.25592517111042546492590736e-01,
    0.26869088293991371028123158e-02,
    0.13133767840925681614496481e-03,
];
const R: [f64; 9] = [
    0.145589721275038539045668824025,
    -0.273421931495426482902320421863,
    0.226008066916621506788789064272,
    -0.163571895523923805648814425592,
    0.102604312032193978662297299832,
    -0.548023266949835519254211506880e-01,
    0.241432239725390106956523668160e-01,
    -0.822062115403915116036874169600e-02,
    0.180296241564687154310619200000e-02,
];
const AA: [f64; 9] = [
    -0.45894433406309678202825375e-03,
    -0.12281298722544724287816236e-01,
    -0.91144359512342900801764781e-01,
    -0.28412489223839285652511367e-01,
    0.14083827189977123530129812e+01,
    0.11532175281537044570477189e+01,
    -0.72170903389442152112483632e+01,
    -0.19685597805218214001309225e+01,
    0.93846891504541841150916038e+01,
];
const BB: [f64; 12] = [
    1.0,
    0.25136329960926527692263725e+02,
    0.15349442087145759184067981e+03,
    -0.29971215958498680905476402e+03,
    -0.33876477506888115226730368e+04,
    0.28301829314924804988873701e+04,
    0.22979620942196507068034887e+05,
    -0.24280681522998071562462041e+05,
    -0.36680620673264731899504580e+05,
    0.42278731622295627627042436e+05,
    0.28834257644413614344549790e+03,
    0.70226293775648358646587341e+03,
];
const E0: f64 = 0.540464821348814822409610122136;
const E1: f64 = -0.261515522487415653487049835220e-01;
const E2: f64 = -0.288573438386338758794591212600e-02;
const CW_RM: u16 = CW_PC64_RN | 0x0400;
const CW_RP: u16 = CW_PC64_RN | 0x0800;
const CW_RZ: u16 = CW_PC64_RN | 0x0C00;

fn x87_horner(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ext_from_f64(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(
            &ext_mul(&acc, &x, CW_PC64_RN),
            &ext_from_f64(c),
            CW_PC64_RN,
        );
    }
    acc
}

fn pqr_acc(x: f64) -> Ext80 {
    let xe = ext_from_f64(x);
    let t = ext_div(
        &ext_sub(&xe, &ext_from_f64(3.75), CW_PC64_RN),
        &ext_add(&xe, &ext_from_f64(3.75), CW_PC64_RN),
        CW_PC64_RN,
    );
    let mut acc = ext_div(&x87_horner(&P, xe), &x87_horner(&Q, xe), CW_PC64_RN);
    for &r in R.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(r), CW_PC64_RN);
    }
    acc
}

fn aabb_acc(x: f64) -> Ext80 {
    let xe = ext_from_f64(x);
    let zz = ext_mul(&xe, &xe, CW_PC64_RN);
    let z = ext_div(
        &ext_from_f64(1.0),
        &ext_add(&ext_from_f64(2.5), &zz, CW_PC64_RN),
        CW_PC64_RN,
    );
    let t = ext_sub(
        &ext_mul(&ext_from_f64(13.0), &z, CW_PC64_RN),
        &ext_from_f64(1.0),
        CW_PC64_RN,
    );
    let r = ext_div(
        &x87_horner(&AA, z),
        &x87_horner(&BB, z),
        CW_PC64_RN,
    );
    let mut acc = ext_add(&ext_mul(&r, &t, CW_PC64_RN), &ext_from_f64(E2), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E1), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E0), CW_PC64_RN);
    ext_div(&acc, &xe, CW_PC64_RN)
}

fn sig64(e: &Ext80) -> u64 {
    u64::from_le_bytes(e.0[0..8].try_into().unwrap())
}

/// low 11 bits of the 64-bit significand: round bit = bit 10, sticky = [9:0].
fn round_bits(e: &Ext80) -> (u64, bool, bool) {
    let s = sig64(e);
    let low11 = s & 0x7ff;
    let tie = low11 == 0x400;
    let exact53 = low11 == 0;
    (low11, tie, exact53)
}

fn store(e: &Ext80, cw: u16) -> f64 {
    ext_to_f64(e, cw)
}

fn round_to_odd(e: &Ext80) -> f64 {
    let rn = store(e, CW_PC64_RN);
    let (_, tie, _) = round_bits(e);
    if !tie {
        return rn;
    }
    if rn.to_bits() & 1 == 1 {
        rn
    } else {
        // tie and even: pick the odd neighbor on the round-up side of RN-even.
        let rp = store(e, CW_RP);
        let rm = store(e, CW_RM);
        if rp.to_bits() & 1 == 1 {
            rp
        } else {
            rm
        }
    }
}

fn sticky_away(e: &Ext80) -> f64 {
    let (_, _, exact) = round_bits(e);
    if exact {
        return store(e, CW_PC64_RN);
    }
    if e.0[9] & 0x80 != 0 {
        store(e, CW_RM)
    } else {
        store(e, CW_RP)
    }
}

fn estrin_r_x87(t: Ext80) -> Ext80 {
    // R degree 8, coeffs R[0]..R[8].
    let t2 = ext_mul(&t, &t, CW_PC64_RN);
    let t4 = ext_mul(&t2, &t2, CW_PC64_RN);
    let c = |i: usize| ext_from_f64(R[i]);
    let e0 = ext_add(&c(0), &ext_mul(&c(2), &t2, CW_PC64_RN), CW_PC64_RN);
    let e1 = ext_add(&c(4), &ext_mul(&c(6), &t2, CW_PC64_RN), CW_PC64_RN);
    let even = ext_add(
        &ext_add(&e0, &ext_mul(&e1, &t4, CW_PC64_RN), CW_PC64_RN),
        &ext_mul(&c(8), &ext_mul(&t4, &t4, CW_PC64_RN), CW_PC64_RN),
        CW_PC64_RN,
    );
    let o0 = ext_add(&c(1), &ext_mul(&c(3), &t2, CW_PC64_RN), CW_PC64_RN);
    let o1 = ext_add(&c(5), &ext_mul(&c(7), &t2, CW_PC64_RN), CW_PC64_RN);
    let odd = ext_add(&o0, &ext_mul(&o1, &t4, CW_PC64_RN), CW_PC64_RN);
    ext_add(&even, &ext_mul(&odd, &t, CW_PC64_RN), CW_PC64_RN)
}

fn pqr_estrin(x: f64) -> f64 {
    let xe = ext_from_f64(x);
    let t = ext_div(
        &ext_sub(&xe, &ext_from_f64(3.75), CW_PC64_RN),
        &ext_add(&xe, &ext_from_f64(3.75), CW_PC64_RN),
        CW_PC64_RN,
    );
    let uv = ext_div(&x87_horner(&P, xe), &x87_horner(&Q, xe), CW_PC64_RN);
    let t9 = {
        let t2 = ext_mul(&t, &t, CW_PC64_RN);
        let t4 = ext_mul(&t2, &t2, CW_PC64_RN);
        ext_mul(&ext_mul(&t4, &t4, CW_PC64_RN), &t, CW_PC64_RN)
    };
    let rpart = estrin_r_x87(t);
    ext_to_f64(
        &ext_add(&rpart, &ext_mul(&uv, &t9, CW_PC64_RN), CW_PC64_RN),
        CW_PC64_RN,
    )
}

fn pqr_fma_native(x: f64) -> f64 {
    let t = (x - 3.75) / (x + 3.75);
    let uv = f::horner(&P, x) / f::horner(&Q, x);
    let mut acc = uv;
    for &r in R.iter().rev() {
        acc = acc.mul_add(t, r);
    }
    acc
}

fn aabb_x87(x: f64) -> f64 {
    store(&aabb_acc(x), CW_PC64_RN)
}

fn mid_eval(z: f64, pqr: impl Fn(f64) -> f64) -> f64 {
    if z < 1.5 {
        pqr(z)
    } else if z < 4.0 {
        aabb_x87(z)
    } else {
        f::nswc_ccdd_f(z)
    }
}

fn report(label: &str, rows: &[(f64, u64)], eval: impl Fn(f64) -> f64) {
    let (m, t) = f::score_f(rows, eval);
    println!("  {label:<42} mid {}  tail {}", f::fmt_acc(&m), f::fmt_acc(&t));
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("G3-01-dist (heldout files are not named)");
    assert!(!dir.contains("heldout"));
    let out = PathBuf::from(
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| "../../work/w109/erfc-f-midpt".into()),
    );
    fs::create_dir_all(&out).unwrap();
    let tagged = f::load_q_rows_tagged(&dir);
    let rows: Vec<(f64, u64)> = tagged.iter().map(|r| (r.z, r.qbits)).collect();

    println!("## final-store rounding of unspilled x87 PQR cut 1.5 + AABB");
    report("RN (bar)", &rows, |z| mid_eval(z, |x| store(&pqr_acc(x), CW_PC64_RN)));
    report("RM", &rows, |z| mid_eval(z, |x| store(&pqr_acc(x), CW_RM)));
    report("RP", &rows, |z| mid_eval(z, |x| store(&pqr_acc(x), CW_RP)));
    report("RZ", &rows, |z| mid_eval(z, |x| store(&pqr_acc(x), CW_RZ)));
    report("round-to-odd on tie else RN", &rows, |z| {
        mid_eval(z, |x| round_to_odd(&pqr_acc(x)))
    });
    report("sticky-away (RM/RP if inexact)", &rows, |z| {
        mid_eval(z, |x| sticky_away(&pqr_acc(x)))
    });
    report("Estrin R + uv*t^8 x87", &rows, |z| mid_eval(z, pqr_estrin));
    report("native fma Horner R", &rows, |z| mid_eval(z, pqr_fma_native));

    println!("\n## leftover vs RN unspilled PQR [0.5,1.5): midpoint diagnostic");
    let mut n = 0usize;
    let mut n1 = 0usize;
    let mut n_tie = 0usize;
    let mut n_exact53 = 0usize;
    let mut n1_tie = 0usize;
    let mut n1_exact = 0usize;
    let mut n1_roundbit = 0usize;
    let mut n_tie_exact_fo = 0usize;
    let mut n1_odd_hits = 0usize;
    let mut n1_rm_hits = 0usize;
    let mut n1_rp_hits = 0usize;
    let mut n1_rz_hits = 0usize;
    let mut low11_hist = [0usize; 8];
    let mut tf = fs::File::create(out.join("midpt-pqr.tsv")).unwrap();
    writeln!(tf, "z\tdirect\tulp_rn\tlow11\ttie\texact53\tfo_bits\trn_bits").unwrap();
    for r in &tagged {
        if r.z < 0.5 || r.z >= 1.5 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let acc = pqr_acc(r.z);
        let rn = store(&acc, CW_PC64_RN);
        let d = ulp_distance(rn, fo).unwrap_or(u64::MAX);
        let (low11, tie, exact53) = round_bits(&acc);
        n += 1;
        if tie {
            n_tie += 1;
            if d == 0 {
                n_tie_exact_fo += 1;
            }
        }
        if exact53 {
            n_exact53 += 1;
        }
        let bucket = match low11 {
            0 => 0,
            0x400 => 1,
            1..=0x3ff => 2,
            0x401..=0x5ff => 3,
            0x600..=0x7ff => 4,
            _ => 7,
        };
        low11_hist[bucket] += 1;
        if d == 1 {
            n1 += 1;
            if tie {
                n1_tie += 1;
            }
            if exact53 {
                n1_exact += 1;
            }
            if low11 & 0x400 != 0 {
                n1_roundbit += 1;
            }
            if ulp_distance(round_to_odd(&acc), fo).unwrap_or(1) == 0 {
                n1_odd_hits += 1;
            }
            if ulp_distance(store(&acc, CW_RM), fo).unwrap_or(1) == 0 {
                n1_rm_hits += 1;
            }
            if ulp_distance(store(&acc, CW_RP), fo).unwrap_or(1) == 0 {
                n1_rp_hits += 1;
            }
            if ulp_distance(store(&acc, CW_RZ), fo).unwrap_or(1) == 0 {
                n1_rz_hits += 1;
            }
            writeln!(
                tf,
                "{:.16e}\t{}\t{d}\t{low11}\t{}\t{}\t{}\t{}",
                r.z,
                r.direct as u8,
                tie as u8,
                exact53 as u8,
                fo.to_bits(),
                rn.to_bits()
            )
            .unwrap();
        }
    }
    println!("  PQR-band n={n} leftover-1ulp n1={n1} ties={n_tie} exact53={n_exact53}");
    println!("  ties that already match F_or under RN: {n_tie_exact_fo}");
    println!("  among 1-ULP leftover: tie={n1_tie} exact53={n1_exact} roundbit_set={n1_roundbit}");
    println!(
        "  1-ULP leftover rescued by r2odd={n1_odd_hits} RM={n1_rm_hits} RP={n1_rp_hits} RZ={n1_rz_hits}"
    );
    println!(
        "  low11 buckets (0, tie 0x400, <mid, mid.., high, unused): {low11_hist:?}"
    );

    println!("\n## AABB leftover implied-z [1.5,4) vs z=1/(2.5+x^2)");
    let mut an = 0usize;
    let mut a1 = 0usize;
    let mut a_hist = [0usize; 8];
    let mut a_tie = 0usize;
    let mut a1_tie = 0usize;
    for r in &tagged {
        if r.z < 1.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let acc = aabb_acc(r.z);
        let rn = store(&acc, CW_PC64_RN);
        let d = ulp_distance(rn, fo).unwrap_or(u64::MAX);
        if d == 0 {
            continue;
        }
        an += 1;
        let (_, tie, _) = round_bits(&acc);
        if tie {
            a_tie += 1;
        }
        let zpub = 1.0 / (2.5 + r.z * r.z);
        // invert t from native poly then z=(t+1)/13
        let rr = f::horner(&AA, zpub) / f::horner(&BB, zpub);
        let tpub = 13.0 * zpub - 1.0;
        let mut t = tpub;
        for _ in 0..12 {
            let accp = ((rr * t + E2) * t + E1) * t + E0;
            let dacc = (3.0 * rr * t + 2.0 * E2) * t + E1;
            if dacc == 0.0 {
                break;
            }
            t -= (accp - fo * r.z) / dacc;
        }
        let zimp = (t + 1.0) / 13.0;
        let dz = ulp_distance(zimp, zpub).unwrap_or(u64::MAX);
        let bucket = match dz {
            0 => 0,
            1 => 1,
            2 => 2,
            3..=7 => 3,
            8..=63 => 4,
            64..=1023 => 5,
            1024..=1048576 => 6,
            _ => 7,
        };
        a_hist[bucket] += 1;
        if d == 1 {
            a1 += 1;
            if tie {
                a1_tie += 1;
            }
        }
    }
    println!("  AABB leftover n={an} of which 1-ULP F n1={a1} ties_among_leftover={a_tie} ties_among_1ulp={a1_tie}");
    println!("  ulp(z_imp,z_pub) hist 0,1,2,3-7,8-63,64-1k,1k-2^20,big: {a_hist:?}");
    println!("artifacts in {}", out.display());
}
