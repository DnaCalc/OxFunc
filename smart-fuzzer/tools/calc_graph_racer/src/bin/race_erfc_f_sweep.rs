//! Inverse-problem F sweep: winner map, CF n*, Lentz/even-odd/stop,
//! NSWC internal cuts, t-association, x87 F, named-site stores, 3-piece.
//! Heldouts unnamed. Does not land kernels.
//!
//!   cargo run --release --bin race_erfc_f_sweep -- ../../work/w109/G3-01-dist [out-dir]

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

fn nswc_pqr_x87(x: f64, t_mode: u8, site: u8) -> f64 {
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
        1 => {
            let q = ext_div(&ext_from_f64(7.5), &xp, CW_PC64_RN);
            ext_sub(&ext_from_f64(1.0), &q, CW_PC64_RN)
        }
        2 => {
            let r = ext_div(&ext_from_f64(3.75), &xe, CW_PC64_RN);
            let n = ext_sub(&ext_from_f64(1.0), &r, CW_PC64_RN);
            let d = ext_add(&ext_from_f64(1.0), &r, CW_PC64_RN);
            ext_div(&n, &d, CW_PC64_RN)
        }
        3 => {
            let u = ext_div(&xe, &ext_from_f64(3.75), CW_PC64_RN);
            let n = ext_sub(&u, &ext_from_f64(1.0), CW_PC64_RN);
            let d = ext_add(&u, &ext_from_f64(1.0), CW_PC64_RN);
            ext_div(&n, &d, CW_PC64_RN)
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

fn nswc_aabb_x87(x: f64) -> f64 {
    let xe = ext_from_f64(x);
    let zz = ext_mul(&xe, &xe, CW_PC64_RN);
    let den = ext_add(&ext_from_f64(2.5), &zz, CW_PC64_RN);
    let z = ext_div(&ext_from_f64(1.0), &den, CW_PC64_RN);
    let t = ext_sub(&ext_mul(&ext_from_f64(13.0), &z, CW_PC64_RN), &ext_from_f64(1.0), CW_PC64_RN);
    let u = x87_horner(&AA, z);
    let v = x87_horner(&BB, z);
    let r = ext_div(&u, &v, CW_PC64_RN);
    let mut acc = ext_add(&ext_mul(&r, &t, CW_PC64_RN), &ext_from_f64(E2), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E1), CW_PC64_RN);
    acc = ext_add(&ext_mul(&acc, &t, CW_PC64_RN), &ext_from_f64(E0), CW_PC64_RN);
    ext_to_f64(&ext_div(&acc, &xe, CW_PC64_RN), CW_PC64_RN)
}

fn nswc_derfc0_x87(x: f64) -> f64 {
    if x <= 2.0 {
        nswc_pqr_x87(x, 0, 0)
    } else if x <= 4.0 {
        nswc_aabb_x87(x)
    } else {
        f::nswc_ccdd_f(x)
    }
}

fn nswc_cut(x: f64, mid: f64, far: f64) -> f64 {
    if x < mid {
        f::nswc_pqr_f(x)
    } else if x < far {
        // AA/BB body of DERFC0 without the published x<=2 gate
        let z = 1.0 / (2.5 + x * x);
        let t = 13.0 * z - 1.0;
        let acc = ((f::horner(&AA, z) / f::horner(&BB, z) * t + E2) * t + E1) * t + E0;
        acc / x
    } else {
        f::nswc_ccdd_f(x)
    }
}

fn report(label: &str, rows: &[(f64, u64)], eval: impl Fn(f64) -> f64) {
    let (m, t) = f::score_f(rows, eval);
    println!("  {label:<28} mid {}  tail {}", f::fmt_acc(&m), f::fmt_acc(&t));
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("G3-01-dist (heldout files are not named)");
    assert!(!dir.contains("heldout"));
    let out = PathBuf::from(
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| "../../work/w109/erfc-f-sweep".into()),
    );
    fs::create_dir_all(&out).unwrap();
    let tagged = f::load_q_rows_tagged(&dir);
    let rows: Vec<(f64, u64)> = tagged.iter().map(|r| (r.z, r.qbits)).collect();
    let direct: Vec<(f64, u64)> = tagged
        .iter()
        .filter(|r| r.direct)
        .map(|r| (r.z, r.qbits))
        .collect();
    let implied: Vec<(f64, u64)> = tagged
        .iter()
        .filter(|r| !r.direct)
        .map(|r| (r.z, r.qbits))
        .collect();
    let n_direct = tagged.iter().filter(|r| r.direct).count();
    println!(
        "rows={} direct={} implied={}  F_or w_rn53; heldout absent",
        rows.len(),
        n_direct,
        tagged.len() - n_direct
    );

    println!("\n## corpora split (NSWC DERFC0)");
    report("merged", &rows, f::nswc_derfc0);
    report("direct-only", &direct, f::nswc_derfc0);
    report("implied-only", &implied, f::nswc_derfc0);

    println!("\n## named recurrences (merged)");
    for (name, eval) in [
        ("back_as714_n21", (|z| f::cf_as714_n(z, 21)) as fn(f64) -> f64),
        ("back_gaut_n21", |z| f::cf_gautschi_n(z, 21)),
        ("lentz_as714_n21", |z| f::cf_lentz_as714_n(z, 21)),
        ("lentz_gaut_n21", |z| f::cf_lentz_gaut_n(z, 21)),
        ("mlentz_as714_n21", |z| f::cf_mlentz_as714_n(z, 21)),
        ("mlentz_gaut_n21", |z| f::cf_mlentz_gaut_n(z, 21)),
        ("lentz_as714_n80", |z| f::cf_lentz_as714_n(z, 80)),
        ("lentz_gaut_n80", |z| f::cf_lentz_gaut_n(z, 80)),
        ("mlentz_as714_stop80", |z| f::cf_lentz_as714_stop(z, 80)),
        ("mlentz_gaut_stop80", |z| f::cf_lentz_gaut_stop(z, 80)),
        ("evenodd_as714_n12", |z| f::cf_evenodd_as714_n(z, 12)),
        ("evenodd_as714_n21", |z| f::cf_evenodd_as714_n(z, 21)),
        ("evenodd_as714_n40", |z| f::cf_evenodd_as714_n(z, 40)),
    ] {
        report(name, &rows, eval);
    }

    println!("\n## x87 NSWC F vs native F");
    report("nswc_derfc0_native", &rows, f::nswc_derfc0);
    report("nswc_pqr_native", &rows, f::nswc_pqr_f);
    report("nswc_pqr_x87", &rows, |z| nswc_pqr_x87(z, 0, 0));
    report("nswc_derfc0_x87", &rows, nswc_derfc0_x87);
    report("nswc_aabb_x87_allz", &rows, nswc_aabb_x87);

    println!("\n## NSWC t-association (PQR on all z)");
    for (name, tm) in [
        ("t=published", f::nswc_t_published as fn(f64) -> f64),
        ("t=1-7.5/(x+3.75)", f::nswc_t_oneminus),
        ("t=(1-r)/(1+r)", f::nswc_t_divfirst),
        ("t=(x/3.75-1)/(+1)", f::nswc_t_scaled),
    ] {
        report(name, &rows, |z| f::nswc_pqr_t(z, tm(z)));
    }

    println!("\n## NSWC internal cuts (PQR / AA-BB / CC-DD)");
    let mut best_cut = (0.0, 0.0, 0usize);
    let mut cut_md = String::from("mid\tfar\tmid_exact\ttail_exact\tall\n");
    for mi in 0..=40 {
        let midc = 0.5 + mi as f64 * 0.1;
        for fi in 0..=40 {
            let farc = 1.5 + fi as f64 * 0.2;
            if farc <= midc || farc > 10.0 {
                continue;
            }
            let (m, t) = f::score_f(&rows, |z| nswc_cut(z, midc, farc));
            let all = m.exact + t.exact;
            if all > best_cut.2 {
                best_cut = (midc, farc, all);
            }
            if all >= 3632 || (mi % 5 == 0 && fi % 5 == 0) {
                println!(
                    "  midc={midc:.1} farc={farc:.1} mid {} tail {} all={}",
                    f::fmt_acc(&m),
                    f::fmt_acc(&t),
                    all
                );
            }
            if all >= 3600 {
                cut_md.push_str(&format!(
                    "{midc:.1}\t{farc:.1}\t{}\t{}\t{all}\n",
                    m.exact, t.exact
                ));
            }
        }
    }
    println!(
        "  best internal-cut midc={:.1} farc={:.1} all_exact={}",
        best_cut.0, best_cut.1, best_cut.2
    );
    fs::write(out.join("nswc-internal-cuts.tsv"), cut_md).unwrap();

    println!("\n## x87 NSWC F with retuned PQR/AABB/CCDD cuts");
    fn nswc_cut_x87(x: f64, mid: f64, far: f64) -> f64 {
        if x < mid {
            nswc_pqr_x87(x, 0, 0)
        } else if x < far {
            nswc_aabb_x87(x)
        } else {
            f::nswc_ccdd_f(x)
        }
    }
    let mut best_x = (0.0, 0.0, 0usize);
    for mi in 10..=20 {
        let midc = mi as f64 * 0.1;
        for fi in 20..=40 {
            let farc = fi as f64 * 0.1;
            if farc <= midc {
                continue;
            }
            let (m, t) = f::score_f(&rows, |z| nswc_cut_x87(z, midc, farc));
            let all = m.exact + t.exact;
            if all > best_x.2 {
                best_x = (midc, farc, all);
                println!(
                    "  x87 midc={midc:.1} farc={farc:.1} mid {} tail {} all={all}",
                    f::fmt_acc(&m),
                    f::fmt_acc(&t)
                );
            }
        }
    }
    println!(
        "  best x87-cut midc={:.1} farc={:.1} all_exact={}",
        best_x.0, best_x.1, best_x.2
    );

    println!("\n## 3-piece NSWC / Cody / CF-as714-n80");
    let fams: [(&str, fn(f64) -> f64); 3] = [
        ("nswc", f::nswc_derfc0),
        ("cody", f::cody_erfcx_f),
        ("cf", f::cf_as714_f),
    ];
    let cuts1 = [0.8, 1.0, 1.2, 1.4, 1.5, 1.6, 1.8, 2.0, 2.5, 3.0, 3.5];
    let cuts2 = [2.0, 2.5, 3.0, 3.5, 4.0, 5.0, 5.6, 6.0, 8.0];
    let mut best3 = (0usize, String::new());
    let mut tsv3 = String::from("lo\thi\tfam0\tfam1\tfam2\tmid\ttail\tall\n");
    for &c1 in &cuts1 {
        for &c2 in &cuts2 {
            if c2 <= c1 {
                continue;
            }
            for i0 in 0..3 {
                for i1 in 0..3 {
                    for i2 in 0..3 {
                        if i0 == i1 && i1 == i2 {
                            continue;
                        }
                        let (m, t) = f::score_f(&rows, |z| {
                            if z < c1 {
                                fams[i0].1(z)
                            } else if z < c2 {
                                fams[i1].1(z)
                            } else {
                                fams[i2].1(z)
                            }
                        });
                        let all = m.exact + t.exact;
                        if all > best3.0 {
                            best3 = (
                                all,
                                format!(
                                    "{}<{c1:.1} {}<{c2:.1} {}  mid {} tail {}",
                                    fams[i0].0,
                                    fams[i1].0,
                                    fams[i2].0,
                                    f::fmt_acc(&m),
                                    f::fmt_acc(&t)
                                ),
                            );
                        }
                        if all >= 3699 {
                            println!(
                                "  {}<{c1:.1} {}<{c2:.1} {} mid {} tail {} all={all}",
                                fams[i0].0,
                                fams[i1].0,
                                fams[i2].0,
                                f::fmt_acc(&m),
                                f::fmt_acc(&t)
                            );
                            tsv3.push_str(&format!(
                                "{c1:.1}\t{c2:.1}\t{}\t{}\t{}\t{}\t{}\t{all}\n",
                                fams[i0].0,
                                fams[i1].0,
                                fams[i2].0,
                                m.exact,
                                t.exact
                            ));
                        }
                    }
                }
            }
        }
    }
    println!("  best 3-piece all_exact={} {}", best3.0, best3.1);
    fs::write(out.join("piece3.tsv"), tsv3).unwrap();

    println!("\n## x87 PQR named-site mask (8-bit) on mid, t_mode=0");
    let mid_rows: Vec<(f64, u64)> = rows
        .iter()
        .copied()
        .filter(|(z, _)| *z >= 0.5 && *z < 4.0)
        .collect();
    let mut best_site = (0u8, 0usize);
    for site in 0u8..=127 {
        let (m, _) = f::score_f(&mid_rows, |z| nswc_pqr_x87(z, 0, site));
        if m.exact > best_site.1 {
            best_site = (site, m.exact);
            println!("  site=0x{site:02x} mid {}", f::fmt_acc(&m));
        }
    }
    println!(
        "  best named-site 0x{:02x} mid_exact={}",
        best_site.0, best_site.1
    );
    for t_mode in 1..=3 {
        let mut best = (0u8, 0usize);
        for site in [0u8, 4, 32, 64, 7, 56, 127] {
            let (m, _) = f::score_f(&mid_rows, |z| nswc_pqr_x87(z, t_mode, site));
            if m.exact > best.1 {
                best = (site, m.exact);
            }
        }
        println!("  t_mode={t_mode} best_probe site=0x{:02x} exact={}", best.0, best.1);
    }

    println!("\n## winner map + CF n* (merged z>=0.5)");
    let named: [(&str, fn(f64) -> f64); 6] = [
        ("nswc", f::nswc_derfc0),
        ("cody", f::cody_erfcx_f),
        ("cephes", f::cephes_f),
        ("cdflib", f::cdflib_erfc1_f),
        ("cf_as714", f::cf_as714_f),
        ("cf_gaut", f::cf_gautschi_f),
    ];
    let win_path = out.join("winner-map.tsv");
    let mut wf = fs::File::create(&win_path).unwrap();
    writeln!(wf, "z\tdirect\tnswc_ulp\twinner\twin_ulp\tnstar_as\tnstar_gaut").unwrap();
    let mut nstar_as = [0usize; 42];
    let mut nstar_g = [0usize; 42];
    let mut none = 0usize;
    let mut nswc_only = 0usize;
    let mut cf_only = 0usize;
    let mut both = 0usize;
    for r in &tagged {
        if r.z < 0.5 {
            continue;
        }
        let Some(fo) = f::f_or(r.z, r.qbits) else {
            continue;
        };
        let mut best_name = "none";
        let mut best_d = u64::MAX;
        let mut nswc_d = u64::MAX;
        let mut cf_d = u64::MAX;
        for (name, eval) in named {
            let fg = eval(r.z);
            if !fg.is_finite() {
                continue;
            }
            let d = ulp_distance(fg, fo).unwrap_or(u64::MAX);
            if name == "nswc" {
                nswc_d = d;
            }
            if name == "cf_as714" {
                cf_d = d;
            }
            if d < best_d {
                best_d = d;
                best_name = name;
            }
        }
        if nswc_d == 0 && cf_d == 0 {
            both += 1;
        } else if nswc_d == 0 {
            nswc_only += 1;
        } else if cf_d == 0 {
            cf_only += 1;
        } else {
            none += 1;
        }
        let mut ns = 0u32;
        let mut ng = 0u32;
        for n in 1u32..=40 {
            if ns == 0 {
                let d = ulp_distance(f::cf_as714_n(r.z, n), fo).unwrap_or(u64::MAX);
                if d == 0 {
                    ns = n;
                }
            }
            if ng == 0 {
                let d = ulp_distance(f::cf_gautschi_n(r.z, n), fo).unwrap_or(u64::MAX);
                if d == 0 {
                    ng = n;
                }
            }
            if ns != 0 && ng != 0 {
                break;
            }
        }
        nstar_as[ns as usize] += 1;
        nstar_g[ng as usize] += 1;
        writeln!(
            wf,
            "{:.16e}\t{}\t{}\t{best_name}\t{best_d}\t{ns}\t{ng}",
            r.z,
            r.direct as u8,
            nswc_d
        )
        .unwrap();
    }
    println!(
        "  exact-set nswc_only={nswc_only} cf_as714_only={cf_only} both={both} neither={none}"
    );
    println!("  n* as714 hist n=0 means never-exact:");
    print!("   ");
    for n in 0..=40 {
        if nstar_as[n] > 0 {
            print!(" n{n}={}", nstar_as[n]);
        }
    }
    println!();
    print!("  n* gaut  hist");
    for n in 0..=40 {
        if nstar_g[n] > 0 {
            print!(" n{n}={}", nstar_g[n]);
        }
    }
    println!();
    println!("  dumped {}", win_path.display());

    let mut nstar_f = fs::File::create(out.join("cf-nstar-hist.tsv")).unwrap();
    writeln!(nstar_f, "n\tas714\tgaut").unwrap();
    for n in 0..=40 {
        writeln!(nstar_f, "{n}\t{}\t{}", nstar_as[n], nstar_g[n]).unwrap();
    }

    println!("\n## pins");
    for &z in &f::PIN_Z {
        let Some(r) = tagged.iter().find(|rr| rr.z == z) else {
            continue;
        };
        let Some(fo) = f::f_or(z, r.qbits) else {
            continue;
        };
        print!("  z={z} direct={}:", r.direct);
        for (name, eval) in [
            ("nswc", f::nswc_derfc0 as fn(f64) -> f64),
            ("pqr_x87", |x| nswc_pqr_x87(x, 0, 0)),
            ("lentz21", |x| f::cf_lentz_as714_n(x, 21)),
            ("stop80", |x| f::cf_lentz_as714_stop(x, 80)),
            ("cdflib", f::cdflib_erfc1_f),
            ("cephes", f::cephes_f),
        ] {
            print!(
                " {name}={}",
                ulp_distance(eval(z), fo).unwrap_or(u64::MAX)
            );
        }
        println!();
    }
    println!("\nartifacts in {}", out.display());
}
