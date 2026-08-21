//! agent-P: per-case decode of Excel's internal annuity em = expm1(-n*log1p(r)).
//! Loads the PINNED em oracle (pmt_em_pinned.json, key "rhex|n" -> em f64 hex or null)
//! and, for each pinned case, computes em candidates with REAL x87 ops, then reports
//! the SIGNED ulp residual (candidate - excel) distribution + per-case dumps.
//! Focus first: n=1, where t = -log1p(r) is a SINGLE transcendental (negate is exact).
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

const CW: u16 = rx::CW_PC64_RN;

fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn hb(x: f64) -> String {
    format!("0x{:016x}", x.to_bits())
}
fn ulp(a: f64, b: f64) -> i64 {
    // signed ulp difference a-b in the shared binade (both finite, same sign here)
    a.to_bits() as i64 - b.to_bits() as i64
}

// ---- log1p deliveries ----
fn log1p_port(r: f64) -> f64 {
    rx::excel_log1p(r)
}
// x87 natural log1p via fyl2xp1(ln2, r), stored to f64 (RN53)
fn log1p_fyl2xp1_store(r: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_fyl2xp1(&rx::ext_ln2(), &rx::ext_from_f64(r), CW),
        CW,
    )
}
// x87 ln(1+r): form 1+r in double, then x87 ln
fn ln1pr(r: f64) -> f64 {
    rx::excel_ln(1.0 + r)
}

// The internal-Kahan expm1 (f64 arithmetic), reproduced so we can inspect terms.
struct Kahan {
    u: f64,
    lnu: f64,
    em: f64,
    branch: u8, // 0: u==1 -> t ; 1: |t|<1 kahan ; 2: plain u-1
}
fn kahan_expm1(t: f64) -> Kahan {
    let u = rx::excel_exp(t);
    if u == 1.0 {
        return Kahan {
            u,
            lnu: 0.0,
            em: t,
            branch: 0,
        };
    }
    if t.abs() < 1.0 {
        let lnu = rx::excel_ln(u);
        let em = (u - 1.0) * t / lnu;
        Kahan {
            u,
            lnu,
            em,
            branch: 1,
        }
    } else {
        Kahan {
            u,
            lnu: 0.0,
            em: u - 1.0,
            branch: 2,
        }
    }
}

fn load_pinned(path: &str) -> Vec<(f64, i64, f64)> {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("json");
    let mut out = Vec::new();
    for (k, val) in v.as_object().unwrap() {
        if val.is_null() {
            continue;
        }
        let (rh, nh) = k.split_once('|').unwrap();
        let r = fb(rh);
        let n: i64 = nh.parse().unwrap();
        let em = fb(val.as_str().unwrap());
        out.push((r, n, em));
    }
    out
}

fn hist(name: &str, v: &[i64]) {
    let mut m: BTreeMap<i64, u32> = BTreeMap::new();
    for &x in v {
        *m.entry(x).or_default() += 1;
    }
    let n = v.len();
    let exact = *m.get(&0).unwrap_or(&0);
    print!(
        "  {:32} exact {:4}/{:4} ({:5.1}%)  hist ",
        name,
        exact,
        n,
        100.0 * exact as f64 / n as f64
    );
    for (k, c) in &m {
        print!("{}:{} ", k, c);
    }
    println!();
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../../work/w109/G6-solvers/pmt_em_pinned.json".into());
    let rows = load_pinned(&path);
    let n1: Vec<_> = rows.iter().filter(|(_, n, _)| *n == 1).cloned().collect();
    println!("total pinned: {}  n=1 pinned: {}", rows.len(), n1.len());

    // --- n=1: t = -log1p(r), a single transcendental. Test log1p x expm1 grid ---
    println!("\n=== n=1  em candidate residuals (candidate - excel, in ulp) ===");
    let logs: [(&str, fn(f64) -> f64); 3] = [
        ("log1p_port", log1p_port),
        ("log1p_fyl2xp1_store", log1p_fyl2xp1_store),
        ("ln(1+r)_x87", ln1pr),
    ];
    for (lname, lf) in logs {
        let mut res = Vec::new();
        for (r, _, em_x) in &n1 {
            let t = -lf(*r);
            let em = kahan_expm1(t).em;
            res.push(ulp(em, *em_x));
        }
        hist(&format!("kahan / {}", lname), &res);
    }

    // branch distribution for n=1 (which expm1 branch fires)
    let mut br = [0u32; 3];
    for (r, _, _) in &n1 {
        let t = -log1p_port(*r);
        br[kahan_expm1(t).branch as usize] += 1;
    }
    println!(
        "\n  n=1 kahan branch counts: u==1(ret t):{}  |t|<1 kahan:{}  plain u-1:{}",
        br[0], br[1], br[2]
    );

    // --- Per-case dump of the DISAGREEING n=1 cases (kahan/log1p_port) ---
    // For each mismatch: show r, t, u, lnu, em_cand, em_excel, ulp, and whether
    // an alternate final-store rounding of the SAME extended quotient would match.
    println!("\n=== n=1 per-case DISAGREEMENTS (kahan / log1p_port) ===");
    println!("  columns: r  du(cand-excel)  em_cand  em_excel  u  lnu  branch");
    let mut shown = 0;
    for (r, _, em_x) in &n1 {
        let t = -log1p_port(*r);
        let k = kahan_expm1(t);
        let du = ulp(k.em, *em_x);
        if du != 0 && shown < 40 {
            println!(
                "  r={:+.8e} du={:+} cand={} exc={} u={} lnu={} br={}",
                r,
                du,
                hb(k.em),
                hb(*em_x),
                hb(k.u),
                hb(k.lnu),
                k.branch
            );
            shown += 1;
        }
    }
    println!("  (showing {} of the disagreements)", shown);
}
