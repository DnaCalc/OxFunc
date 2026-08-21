//! W109 offline racer: ACOTH(x) for |x|>1, mirroring the ATANH two-regime
//! identification (check_atanh_unified.rs). Candidate forms:
//!   1. ratio_x87       : signed, r=(x+1)/(x-1), 0.5*ln(r) via x87 fyl2x
//!   2. uni_x87_signed   : signed, r=2/(x-1), 0.5*ln1p(r) via x87 fyl2xp1 (no abs)
//!   3. uni_x87_abs      : r=2/(|x|-1), 0.5*ln1p(r) via x87 fyl2xp1, then copysign(x)
//!   4. uni_sse_abs      : same arg, but production SSE2 excel_log1p, then copysign(x)
//!   5. platform         : current production baseline (platform ln_1p), copysign(x)
//!   6. pair_1overx      : ATANH(1/x) pair form via x87 fyl2xp1 pair
//! All scoring offline against the ACOTH corpus. No production code touched.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

const CW: u16 = 0x133F;
fn e(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}
fn tof(v: &rx::Ext80) -> f64 {
    rx::ext_to_f64(v, CW)
}

// 1. near-1 ratio form, signed directly (mirrors ATANH region-C ratio).
fn ratio_x87(x: f64) -> f64 {
    let r = (x + 1.0) / (x - 1.0);
    let l = rx::ext_fyl2x(&rx::ext_ln2(), &e(r), CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}

// 2. large-|x| log1p form, signed directly (no abs/copysign) — arg is
//    naturally negative for x>1 (since x-1>0 => r>0... wait signed test below).
fn uni_x87_signed(x: f64) -> f64 {
    let r = 2.0 / (x - 1.0);
    let l = rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}

// 3. large-|x| log1p form via abs + copysign (mirrors production structure,
//    x87 fyl2xp1 instead of platform ln_1p).
fn uni_x87_abs(x: f64) -> f64 {
    let r = 2.0 / (x.abs() - 1.0);
    let l = rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW)).copysign(x)
}

// 4. same arg, production SSE2 double-double log1p (rx::excel_log1p).
fn uni_sse_abs(x: f64) -> f64 {
    (0.5 * rx::excel_log1p(2.0 / (x.abs() - 1.0))).copysign(x)
}

// 5. current production baseline: platform ln_1p.
fn platform(x: f64) -> f64 {
    (0.5 * (2.0 / (x.abs() - 1.0)).ln_1p()).copysign(x)
}

// 6. ATANH(1/x) pair form via x87 fyl2xp1, expect ~worse (region mismatch).
fn pair_1overx(x: f64) -> f64 {
    let t = 1.0 / x;
    let ln2 = rx::ext_ln2();
    let t1 = rx::ext_fyl2xp1(&ln2, &e(t), CW);
    let t2 = rx::ext_fyl2xp1(&ln2, &e(-t), CW);
    let d = rx::ext_sub(&t1, &t2, CW);
    tof(&rx::ext_mul(&d, &e(0.5), CW))
}

// --- reciprocal-staging variants of the large-|x| pair form (ACOTH = ATANH(1/x)) ---
// t formed 3 ways; pair difference + halve in x87 extended, single final store.
fn pair_recip_bin64(x: f64) -> f64 {
    // t = 1/x in binary64 (agent baseline)
    let t = 1.0 / x;
    let ln2 = rx::ext_ln2();
    let a = rx::ext_fyl2xp1(&ln2, &e(t), CW);
    let b = rx::ext_fyl2xp1(&ln2, &e(-t), CW);
    tof(&rx::ext_mul(&rx::ext_sub(&a, &b, CW), &e(0.5), CW))
}
fn pair_recip_x87(x: f64) -> f64 {
    // t = RN53(RN64(1/x)) — Excel's POWER x87 reciprocal
    let t = rx::x87_recip(x);
    let ln2 = rx::ext_ln2();
    let a = rx::ext_fyl2xp1(&ln2, &e(t), CW);
    let b = rx::ext_fyl2xp1(&ln2, &e(-t), CW);
    tof(&rx::ext_mul(&rx::ext_sub(&a, &b, CW), &e(0.5), CW))
}
fn pair_recip_extkeep(x: f64) -> f64 {
    // 1/x in x87 extended, kept extended (never stored)
    let ln2 = rx::ext_ln2();
    let t = rx::ext_div(&rx::ext_one(), &e(x), CW); // extended reciprocal
    let nt = rx::ext_div(&rx::ext_one(), &e(-x), CW);
    let a = rx::ext_fyl2xp1(&ln2, &t, CW);
    let b = rx::ext_fyl2xp1(&ln2, &nt, CW);
    tof(&rx::ext_mul(&rx::ext_sub(&a, &b, CW), &e(0.5), CW))
}
// ratio via the production excel_log (=excel_ln), to confirm it equals raw fyl2x staging.
fn ratio_excel_ln(x: f64) -> f64 {
    0.5 * rx::excel_ln((x + 1.0) / (x - 1.0))
}

fn load(paths: &[&str]) -> Vec<(f64, u64)> {
    let mut m: std::collections::BTreeMap<u64, u64> = Default::default();
    for p in paths {
        let ws: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(p).expect("read")).expect("parse");
        for w in &ws.witnesses {
            if !w.expected_bits.starts_with("0x") {
                continue;
            }
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            if let Some(v) = parse_bits_hex(&w.expected_bits) {
                m.insert(x.to_bits(), v.to_bits());
            }
        }
    }
    m.into_iter().map(|(k, v)| (f64::from_bits(k), v)).collect()
}

const REGIONS: [(f64, f64); 6] = [
    (1.0, 1.5),
    (1.5, 3.0),
    (3.0, 10.0),
    (10.0, 1e3),
    (1e3, 1e8),
    (1e8, f64::INFINITY),
];

fn score_all(rows: &[(f64, u64)], f: &dyn Fn(f64) -> f64) -> u32 {
    let mut ok = 0u32;
    for (x, want) in rows {
        if f(*x).to_bits() == *want {
            ok += 1;
        }
    }
    ok
}

fn region_counts(rows: &[(f64, u64)], f: &dyn Fn(f64) -> f64) -> Vec<(u32, u32)> {
    REGIONS
        .iter()
        .map(|&(lo, hi)| {
            let mut ok = 0u32;
            let mut n = 0u32;
            for (x, want) in rows {
                let a = x.abs();
                if a < lo || a >= hi {
                    continue;
                }
                n += 1;
                if f(*x).to_bits() == *want {
                    ok += 1;
                }
            }
            (ok, n)
        })
        .collect()
}

fn main() {
    let rows = load(&["../../work/w109/G4-hyp-answers-acoth.json"]);
    let total = rows.len();
    let n_lt15 = rows.iter().filter(|(x, _)| x.abs() < 1.5).count();
    let n_ge15 = total - n_lt15;
    println!("ACOTH corpus: {total} rows total  (|x|<1.5: {n_lt15}, |x|>=1.5: {n_ge15})\n");

    let forms: [(&str, fn(f64) -> f64); 6] = [
        ("ratio_x87", ratio_x87),
        ("uni_x87_signed", uni_x87_signed),
        ("uni_x87_abs", uni_x87_abs),
        ("uni_sse_abs", uni_sse_abs),
        ("platform", platform),
        ("pair_1overx", pair_1overx),
    ];

    println!("=== per-form scores ===");
    for (name, f) in forms {
        let ok = score_all(&rows, &f);
        let regions = region_counts(&rows, &f);
        print!("{name:16} total {ok:3}/{total}   ");
        for (i, (ro, rn)) in regions.iter().enumerate() {
            let (lo, hi) = REGIONS[i];
            if hi.is_finite() {
                print!("[{lo:.1e},{hi:.1e}):{ro}/{rn}  ");
            } else {
                print!("[{lo:.1e},inf):{ro}/{rn}  ");
            }
        }
        println!();
    }

    // Best piecewise: uni_x87_abs for |x|>=T, else ratio_x87.
    println!("\n=== piecewise ratio_x87(|x|<T) | uni_x87_abs(|x|>=T) ===");
    let mut best = (0u32, 0f64);
    for &t in &[1.2f64, 1.5, 2.0, 3.0, 5.0, 10.0, 100.0, 1e3] {
        let mut ok = 0u32;
        for (x, want) in &rows {
            let v = if x.abs() < t {
                ratio_x87(*x)
            } else {
                uni_x87_abs(*x)
            };
            if v.to_bits() == *want {
                ok += 1;
            }
        }
        if ok > best.0 {
            best = (ok, t);
        }
        println!("  T={t:.4}: {ok}/{total}");
    }
    println!("BEST piecewise: {}/{} at T={:.4}", best.0, total, best.1);

    // Boundary bracket table, |x| ascending.
    println!("\n=== boundary bracket, |x| ascending: ratio_x87 / uni_x87_abs / uni_sse_abs ===");
    let mut sorted = rows.clone();
    sorted.sort_by(|a, b| a.0.abs().partial_cmp(&b.0.abs()).unwrap());
    for (x, want) in &sorted {
        let re = ratio_x87(*x).to_bits() == *want;
        let ue = uni_x87_abs(*x).to_bits() == *want;
        let se = uni_sse_abs(*x).to_bits() == *want;
        println!(
            "   x={:+.8e}  |x|={:.8e}  ratio_x87 {}  uni_x87_abs {}  uni_sse_abs {}",
            x,
            x.abs(),
            if re { "OK " } else { "..." },
            if ue { "OK " } else { "..." },
            if se { "OK " } else { "..." },
        );
    }

    // Detail on rows where the best piecewise still misses.
    println!("\n=== misses under best piecewise (T={:.4}) ===", best.1);
    let t = best.1;
    for (x, want) in &sorted {
        let v = if x.abs() < t {
            ratio_x87(*x)
        } else {
            uni_x87_abs(*x)
        };
        if v.to_bits() != *want {
            let d = v.to_bits() as i64 - *want as i64;
            println!("   x={:+.8e}  |x|={:.8e}  got-want {:+} ulp", x, x.abs(), d);
        }
    }

    // BONUS (not requested, but pair_1overx aces [10,1e3) where nothing else
    // scores at all): 3-way piecewise ratio_x87(near) | uni_x87_abs(mid) | pair_1overx(far).
    println!(
        "\n=== BONUS: 3-way piecewise ratio_x87(<T1) | uni_x87_abs([T1,T2)) | pair_1overx(>=T2) ==="
    );
    let mut best3 = (0u32, 0f64, 0f64);
    let cuts = [1.2f64, 1.5, 2.0, 3.0, 5.0, 8.0, 10.0];
    for &t1 in &cuts {
        for &t2 in &cuts {
            if t2 < t1 {
                continue;
            }
            let mut ok = 0u32;
            for (x, want) in &rows {
                let a = x.abs();
                let v = if a < t1 {
                    ratio_x87(*x)
                } else if a < t2 {
                    uni_x87_abs(*x)
                } else {
                    pair_1overx(*x)
                };
                if v.to_bits() == *want {
                    ok += 1;
                }
            }
            if ok > best3.0 {
                best3 = (ok, t1, t2);
            }
        }
    }
    println!(
        "BEST 3-way: {}/{} at T1={:.4} T2={:.4}",
        best3.0, total, best3.1, best3.2
    );
    println!(
        "\n--- misses under best 3-way (T1={:.4} T2={:.4}) ---",
        best3.1, best3.2
    );
    for (x, want) in &sorted {
        let a = x.abs();
        let v = if a < best3.1 {
            ratio_x87(*x)
        } else if a < best3.2 {
            uni_x87_abs(*x)
        } else {
            pair_1overx(*x)
        };
        if v.to_bits() != *want {
            let d = v.to_bits() as i64 - *want as i64;
            println!("   x={:+.8e}  |x|={:.8e}  got-want {:+} ulp", x, x.abs(), d);
        }
    }

    // Also show pair_1overx across full |x| ascending for context in [3,inf).
    println!("\n--- pair_1overx exactness, |x| ascending (|x|>=3 only) ---");
    for (x, want) in &sorted {
        if x.abs() < 3.0 {
            continue;
        }
        let pe = pair_1overx(*x).to_bits() == *want;
        println!(
            "   x={:+.8e}  |x|={:.8e}  pair_1overx {}",
            x,
            x.abs(),
            if pe { "OK " } else { "..." }
        );
    }

    // ===== TIMEBOXED reciprocal-staging round =====
    // ratio via excel_ln vs raw fyl2x (should match), and 3 reciprocal stagings
    // for the large-|x| pair. Then best 2-way ratio(<T) | <pair-variant>(>=T).
    println!("\n===== reciprocal-staging round =====");
    println!(
        "ratio_x87 total {}/{}   ratio_excel_ln total {}/{}",
        score_all(&rows, &ratio_x87),
        total,
        score_all(&rows, &ratio_excel_ln),
        total
    );
    let pairs: [(&str, fn(f64) -> f64); 3] = [
        ("pair_recip_bin64", pair_recip_bin64),
        ("pair_recip_x87  ", pair_recip_x87),
        ("pair_recip_extkeep", pair_recip_extkeep),
    ];
    for (name, f) in pairs {
        let r = region_counts(&rows, &f);
        print!("{name}  total {}/{}   ", score_all(&rows, &f), total);
        for (i, (ro, rn)) in r.iter().enumerate() {
            let (lo, hi) = REGIONS[i];
            print!(
                "[{lo:.0e},{:.0e}):{ro}/{rn} ",
                if hi.is_finite() { hi } else { f64::MAX }
            );
        }
        println!();
    }
    println!("\n-- best 2-way ratio_x87(<T) | <pair-variant>(>=T) --");
    let mut best2 = (0u32, 0f64, "");
    for (pname, pf) in pairs {
        for &t in &[2.0f64, 2.5, 3.0, 3.5, 4.0, 5.0] {
            let mut ok = 0u32;
            for (x, want) in &rows {
                let v = if x.abs() < t { ratio_x87(*x) } else { pf(*x) };
                if v.to_bits() == *want {
                    ok += 1;
                }
            }
            if ok > best2.0 {
                best2 = (ok, t, pname);
            }
        }
    }
    println!(
        "BEST 2-way: {}/{} at T={:.2} with {}",
        best2.0,
        total,
        best2.1,
        best2.2.trim()
    );
    // Miss detail for the winner.
    let bpf: fn(f64) -> f64 = match best2.2.trim() {
        "pair_recip_x87" => pair_recip_x87,
        "pair_recip_extkeep" => pair_recip_extkeep,
        _ => pair_recip_bin64,
    };
    println!("-- misses under best 2-way --");
    for (x, want) in &sorted {
        let v = if x.abs() < best2.1 {
            ratio_x87(*x)
        } else {
            bpf(*x)
        };
        if v.to_bits() != *want {
            println!(
                "   x={:+.8e}  |x|={:.6e}  got-want {:+} ulp",
                x,
                x.abs(),
                v.to_bits() as i64 - *want as i64
            );
        }
    }

    // ===== ODDNESS + switch sweep: compute on |x|, copysign at the end =====
    // Hypothesis: ACOTH is odd (unlike ATANH region C) — copysign(ACOTH(|x|), x).
    println!("\n===== odd form: copysign(acoth(|x|), x), ratio(<T)|pair(>=T) =====");
    let ratio_abs = |a: f64| {
        let r = (a + 1.0) / (a - 1.0);
        tof(&rx::ext_mul(
            &rx::ext_fyl2x(&rx::ext_ln2(), &e(r), CW),
            &e(0.5),
            CW,
        ))
    };
    let pair_abs = |a: f64| {
        let ln2 = rx::ext_ln2();
        let t = 1.0 / a;
        let p = rx::ext_fyl2xp1(&ln2, &e(t), CW);
        let m = rx::ext_fyl2xp1(&ln2, &e(-t), CW);
        tof(&rx::ext_mul(&rx::ext_sub(&p, &m, CW), &e(0.5), CW))
    };
    let mut bestodd = (0u32, 0f64);
    for &t in &[2.5f64, 3.0, 3.5, 4.0, 5.0, 6.0, 8.0, 10.0, 100.0] {
        let mut ok = 0u32;
        for (x, want) in &rows {
            let a = x.abs();
            let m = if a < t { ratio_abs(a) } else { pair_abs(a) };
            if m.copysign(*x).to_bits() == *want {
                ok += 1;
            }
        }
        println!("  T={t:.1}: {ok}/{total}");
        if ok > bestodd.0 {
            bestodd = (ok, t);
        }
    }
    println!("BEST odd: {}/{} at T={:.1}", bestodd.0, total, bestodd.1);
    let t = bestodd.1;
    println!("-- misses under best odd --");
    for (x, want) in &sorted {
        let a = x.abs();
        let m = if a < t { ratio_abs(a) } else { pair_abs(a) };
        let v = m.copysign(*x);
        if v.to_bits() != *want {
            println!(
                "   x={:+.8e}  |x|={:.6e}  got-want {:+} ulp",
                x,
                a,
                v.to_bits() as i64 - *want as i64
            );
        }
    }

    // ===== no-regression check: odd-form (T=3.5) vs current platform =====
    let oddform = |x: f64| {
        let a = x.abs();
        let m = if a < 3.5 { ratio_abs(a) } else { pair_abs(a) };
        m.copysign(x)
    };
    let (mut reg, mut gain, mut both_miss) = (0u32, 0u32, 0u32);
    for (x, want) in &rows {
        let p_ok = platform(*x).to_bits() == *want;
        let o_ok = oddform(*x).to_bits() == *want;
        if p_ok && !o_ok {
            reg += 1;
        }
        if o_ok && !p_ok {
            gain += 1;
        }
        if !p_ok && !o_ok {
            both_miss += 1;
        }
    }
    println!(
        "\n===== odd-form(T=3.5) vs platform: regressions={reg}  gains={gain}  both_miss={both_miss} ====="
    );
    println!("(strict improvement iff regressions==0)");
}
