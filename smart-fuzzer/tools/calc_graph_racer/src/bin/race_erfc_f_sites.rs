//! Implied-bit diagnosis of leftover mid rows on x87 NSWC PQR@1.5 / AABB@4.
//! Single-site spills, sign vs z last bit, ulp>=4 list. Heldouts unnamed. No landing.
//!
//!   cargo run --release --bin race_erfc_f_sites -- ../../work/w109/G3-01-dist [out-dir]

use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
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

fn spill(x: Ext80) -> Ext80 {
    ext_from_f64(ext_to_f64(&x, CW_PC64_RN))
}
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

fn pqr_x87(x: f64, t_mode: u8, site: u8) -> f64 {
    let xe = ext_from_f64(x);
    let mut xm = ext_sub(&xe, &ext_from_f64(3.75), CW_PC64_RN);
    let mut xp = ext_add(&xe, &ext_from_f64(3.75), CW_PC64_RN);
    if site & 1 != 0 {
        xm = spill(xm);
    }
    if site & 2 != 0 {
        xp = spill(xp);
    }
    let mut t = match t_mode {
        2 => {
            let r = ext_div(&ext_from_f64(3.75), &xe, CW_PC64_RN);
            ext_div(
                &ext_sub(&ext_from_f64(1.0), &r, CW_PC64_RN),
                &ext_add(&ext_from_f64(1.0), &r, CW_PC64_RN),
                CW_PC64_RN,
            )
        }
        _ => ext_div(&xm, &xp, CW_PC64_RN),
    };
    if site & 4 != 0 {
        t = spill(t);
    }
    let mut u = x87_horner(&P, xe);
    let mut v = x87_horner(&Q, xe);
    if site & 8 != 0 {
        u = spill(u);
    }
    if site & 16 != 0 {
        v = spill(v);
    }
    let mut acc = ext_div(&u, &v, CW_PC64_RN);
    if site & 32 != 0 {
        acc = spill(acc);
    }
    for &r in R.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(r), CW_PC64_RN);
        if site & 64 != 0 {
            acc = spill(acc);
        }
    }
    ext_to_f64(&acc, CW_PC64_RN)
}

fn aabb_x87(x: f64, site: u8) -> f64 {
    let xe = ext_from_f64(x);
    let mut zz = ext_mul(&xe, &xe, CW_PC64_RN);
    if site & 1 != 0 {
        zz = spill(zz);
    }
    let mut den = ext_add(&ext_from_f64(2.5), &zz, CW_PC64_RN);
    if site & 2 != 0 {
        den = spill(den);
    }
    let mut z = ext_div(&ext_from_f64(1.0), &den, CW_PC64_RN);
    if site & 4 != 0 {
        z = spill(z);
    }
    let mut t = ext_sub(
        &ext_mul(&ext_from_f64(13.0), &z, CW_PC64_RN),
        &ext_from_f64(1.0),
        CW_PC64_RN,
    );
    if site & 8 != 0 {
        t = spill(t);
    }
    let mut u = x87_horner(&AA, z);
    let mut v = x87_horner(&BB, z);
    if site & 16 != 0 {
        u = spill(u);
    }
    if site & 32 != 0 {
        v = spill(v);
    }
    let mut r = ext_div(&u, &v, CW_PC64_RN);
    if site & 64 != 0 {
        r = spill(r);
    }
    let mut acc = ext_add(&ext_mul(&r, &t, CW_PC64_RN), &ext_from_f64(E2), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E1), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E0), CW_PC64_RN);
    if site & 128 != 0 {
        acc = spill(acc);
    }
    ext_to_f64(&ext_div(&acc, &xe, CW_PC64_RN), CW_PC64_RN)
}

fn graph(z: f64) -> f64 {
    if z < 1.5 {
        pqr_x87(z, 0, 0)
    } else if z < 4.0 {
        aabb_x87(z, 0)
    } else {
        f::nswc_ccdd_f(z)
    }
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("G3-01-dist (heldout files are not named)");
    assert!(!dir.contains("heldout"));
    let out = PathBuf::from(
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| "../../work/w109/erfc-f-sites".into()),
    );
    fs::create_dir_all(&out).unwrap();
    let tagged = f::load_q_rows_tagged(&dir);
    let mut rf = fs::File::create(out.join("residual-x87-15-4-mid.tsv")).unwrap();
    writeln!(rf, "z\tdirect\tulp\tsign\tzlsb\tf_or\tf_g").unwrap();
    let mut hist = [0usize; 10];
    let mut n_pos = 0usize;
    let mut n_neg = 0usize;
    let mut n_zero = 0usize;
    let mut lsb0_pos = 0usize;
    let mut lsb0_neg = 0usize;
    let mut lsb1_pos = 0usize;
    let mut lsb1_neg = 0usize;
    let mut leftover_pqr: Vec<(f64, u64, bool)> = Vec::new();
    let mut leftover_aabb: Vec<(f64, u64, bool)> = Vec::new();
    println!("## x87 PQR@1.5 / AABB@4 residual (mid)");
    for r in &tagged {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let fg = graph(r.z);
        let d = ulp_distance(fg, fo).unwrap_or(u64::MAX);
        if d > (1 << 20) {
            continue;
        }
        let sign = if d == 0 {
            0
        } else if fg > fo {
            1
        } else {
            -1
        };
        match sign {
            0 => n_zero += 1,
            1 => n_pos += 1,
            _ => n_neg += 1,
        }
        let lsb = r.z.to_bits() & 1;
        if sign > 0 && lsb == 0 {
            lsb0_pos += 1;
        }
        if sign < 0 && lsb == 0 {
            lsb0_neg += 1;
        }
        if sign > 0 && lsb == 1 {
            lsb1_pos += 1;
        }
        if sign < 0 && lsb == 1 {
            lsb1_neg += 1;
        }
        let b = match d {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 4,
            5 => 5,
            6 => 6,
            7 => 7,
            8..=15 => 8,
            _ => 9,
        };
        hist[b] += 1;
        if d >= 4 {
            println!(
                "  ulp>={d} z={:.16e} direct={} sign={sign}",
                r.z, r.direct as u8
            );
        }
        writeln!(
            rf,
            "{:.16e}\t{}\t{d}\t{sign}\t{lsb}\t{}\t{}",
            r.z,
            r.direct as u8,
            fo.to_bits(),
            fg.to_bits()
        )
        .unwrap();
        if d != 0 {
            if r.z < 1.5 {
                leftover_pqr.push((r.z, r.qbits, r.direct));
            } else {
                leftover_aabb.push((r.z, r.qbits, r.direct));
            }
        }
    }
    println!("  exact={n_zero} +={n_pos} -={n_neg} hist={hist:?}");
    println!(
        "  sign vs z lsb: lsb0 +={lsb0_pos} -={lsb0_neg}  lsb1 +={lsb1_pos} -={lsb1_neg}"
    );
    println!(
        "  leftover PQR[0.5,1.5)={}  AABB[1.5,4)={}",
        leftover_pqr.len(),
        leftover_aabb.len()
    );

    println!("\n## single-site PQR spills (how many leftover rows become exact)");
    let site_names = [
        (1u8, "spill x-3.75"),
        (2, "spill x+3.75"),
        (4, "spill t"),
        (8, "spill u=P"),
        (16, "spill v=Q"),
        (32, "spill u/v"),
        (64, "spill each R step"),
    ];
    for t_mode in [0u8, 2] {
        let base_fix = leftover_pqr
            .iter()
            .filter(|&&(z, q, _)| {
                f::f_or(z, q)
                    .and_then(|fo| ulp_distance(pqr_x87(z, t_mode, 0), fo))
                    .unwrap_or(1)
                    == 0
            })
            .count();
        println!("  t_mode={t_mode} site=0 already-exact-on-leftover={base_fix}");
        for (bit, name) in site_names {
            let mut fix = 0usize;
            let mut lose = 0usize;
            let mut better = 0usize;
            for &(z, q, _) in &leftover_pqr {
                let Some(fo) = f::f_or(z, q) else {
                    continue;
                };
                let d0 = ulp_distance(pqr_x87(z, t_mode, 0), fo).unwrap_or(u64::MAX);
                let d1 = ulp_distance(pqr_x87(z, t_mode, bit), fo).unwrap_or(u64::MAX);
                if d1 == 0 {
                    fix += 1;
                }
                if d1 < d0 {
                    better += 1;
                }
                if d1 > d0 {
                    lose += 1;
                }
            }
            println!("  t_mode={t_mode} {name:<22} exact={fix} better={better} worse={lose}");
        }
    }

    println!("\n## single-site AABB spills on leftover [1.5,4)");
    let aabb_names = [
        (1u8, "spill z*z"),
        (2, "spill 2.5+zz"),
        (4, "spill 1/den"),
        (8, "spill t=13z-1"),
        (16, "spill AA"),
        (32, "spill BB"),
        (64, "spill AA/BB"),
        (128, "spill E- Horner"),
    ];
    for (bit, name) in aabb_names {
        let mut fix = 0usize;
        let mut better = 0usize;
        let mut lose = 0usize;
        for &(z, q, _) in &leftover_aabb {
            let Some(fo) = f::f_or(z, q) else {
                continue;
            };
            let d0 = ulp_distance(aabb_x87(z, 0), fo).unwrap_or(u64::MAX);
            let d1 = ulp_distance(aabb_x87(z, bit), fo).unwrap_or(u64::MAX);
            if d1 == 0 {
                fix += 1;
            }
            if d1 < d0 {
                better += 1;
            }
            if d1 > d0 {
                lose += 1;
            }
        }
        println!("  {name:<22} exact={fix} better={better} worse={lose}");
    }

    println!("\n## nextafter: leftover 1-ULP that equal nextafter(fg, ±inf)");
    let mut np = 0usize;
    let mut nn = 0usize;
    let mut n1 = 0usize;
    for r in &tagged {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let fg = graph(r.z);
        let d = ulp_distance(fg, fo).unwrap_or(u64::MAX);
        if d != 1 {
            continue;
        }
        n1 += 1;
        if fo.to_bits() == fg.next_up().to_bits() {
            np += 1;
        }
        if fo.to_bits() == fg.next_down().to_bits() {
            nn += 1;
        }
    }
    println!("  1-ULP rows={n1}  F_or=next_up(fg)={np}  F_or=next_down(fg)={nn}");

    println!("\n## PQR 7-bit store mask on z in [0.5,1.5), AABB baseline");
    let pqr_band: Vec<(f64, u64)> = tagged
        .iter()
        .filter(|r| r.z >= 0.5 && r.z < 1.5)
        .map(|r| (r.z, r.qbits))
        .collect();
    let aabb_band: Vec<(f64, u64)> = tagged
        .iter()
        .filter(|r| r.z >= 1.5 && r.z < 4.0)
        .map(|r| (r.z, r.qbits))
        .collect();
    let (aabb0, _) = f::score_f(&aabb_band, |z| aabb_x87(z, 0));
    let mut best_p = (0u8, 0usize);
    for site in 0u8..=127 {
        let (m, _) = f::score_f(&pqr_band, |z| pqr_x87(z, 0, site));
        if m.exact > best_p.1 {
            best_p = (site, m.exact);
            println!(
                "  PQR site=0x{site:02x} exact={}  +AABB0 {}  mid~{}",
                m.exact,
                aabb0.exact,
                m.exact + aabb0.exact
            );
        }
    }
    println!(
        "  best PQR site=0x{:02x} exact={} combined_mid~{}",
        best_p.0,
        best_p.1,
        best_p.1 + aabb0.exact
    );

    println!("\n## AABB 8-bit store mask on z in [1.5,4), PQR baseline");
    let (pqr0, _) = f::score_f(&pqr_band, |z| pqr_x87(z, 0, 0));
    let mut best_a = (0u8, 0usize);
    for site in 0u16..=255 {
        let s = site as u8;
        let (m, _) = f::score_f(&aabb_band, |z| aabb_x87(z, s));
        if m.exact > best_a.1 {
            best_a = (s, m.exact);
            println!(
                "  AABB site=0x{s:02x} exact={}  +PQR0 {}  mid~{}",
                m.exact,
                pqr0.exact,
                m.exact + pqr0.exact
            );
        }
    }
    println!(
        "  best AABB site=0x{:02x} exact={} combined_mid~{}",
        best_a.0,
        best_a.1,
        best_a.1 + pqr0.exact
    );

    println!("\n## PQR R-Horner per-step 9-bit spill on [0.5,1.5)");
    fn pqr_r_mask(x: f64, mask: u16) -> f64 {
        let xe = ext_from_f64(x);
        let t = ext_div(
            &ext_sub(&xe, &ext_from_f64(3.75), CW_PC64_RN),
            &ext_add(&xe, &ext_from_f64(3.75), CW_PC64_RN),
            CW_PC64_RN,
        );
        let u = x87_horner(&P, xe);
        let v = x87_horner(&Q, xe);
        let mut acc = ext_div(&u, &v, CW_PC64_RN);
        for (i, &r) in R.iter().rev().enumerate() {
            acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(r), CW_PC64_RN);
            if mask & (1 << i) != 0 {
                acc = spill(acc);
            }
        }
        ext_to_f64(&acc, CW_PC64_RN)
    }
    let mut best_r = (0u16, 0usize);
    for mask in 0u16..512 {
        let (m, _) = f::score_f(&pqr_band, |z| pqr_r_mask(z, mask));
        if m.exact > best_r.1 {
            best_r = (mask, m.exact);
            println!(
                "  Rmask=0x{mask:03x} exact={} combined_mid~{}",
                m.exact,
                m.exact + aabb0.exact
            );
        }
    }
    println!(
        "  best Rmask=0x{:03x} exact={} combined_mid~{}",
        best_r.0,
        best_r.1,
        best_r.1 + aabb0.exact
    );
    println!("artifacts in {}", out.display());
}
