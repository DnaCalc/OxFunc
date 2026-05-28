// RAND() characterization study.
//
// NOT a conformance / blessing test. The goal is to characterise Excel's
// RAND() against a few candidate Rust RNG implementations and ask: can a
// statistical battery DISTINGUISH them? (Excel modern RAND is documented as
// Mersenne-Twister-based; older Excel used Wichmann-Hill.) Whether OxFunc's
// RAND stays host-supplied or moves in-crate is undecided — this only
// characterises the situation.
//
// One identical battery is computed for every source (Excel sample read from
// a file + each Rust RNG generated here), plus a two-sample KS distance of
// each Rust RNG against the Excel sample.
//
// usage: rand_characterize --excel <samples.txt> --out <report.json> [--n <count>]
//   <samples.txt>: one f64 per line (Excel RAND draws). --n overrides the
//   per-RNG generated sample size (default: match the Excel sample count).

use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;

// ----------------------------------------------------------------------------
// Candidate RNGs — each yields a unit draw in [0,1). Fixed seeds (deterministic).
// The output transform (grid resolution) is part of what we characterise.
// ----------------------------------------------------------------------------

/// 64-bit LCG (PCG-style multiplier), top 32 bits -> [0,1) on a 2^-32 grid.
struct Lcg {
    state: u64,
}
impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_unit(&mut self) -> f64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let top = (self.state >> 32) as u32;
        top as f64 / 4294967296.0 // 2^32
    }
}

/// xorshift128+ -> 53-bit mantissa / 2^53 (fine grid).
struct Xorshift128p {
    s0: u64,
    s1: u64,
}
impl Xorshift128p {
    fn new(seed: u64) -> Self {
        // seed the two words via splitmix64 so neither is zero
        let mut sm = Splitmix64 { state: seed };
        Self {
            s0: sm.next_u64() | 1,
            s1: sm.next_u64() | 1,
        }
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.s0;
        let y = self.s1;
        self.s0 = y;
        x ^= x << 23;
        self.s1 = x ^ y ^ (x >> 17) ^ (y >> 26);
        self.s1.wrapping_add(y)
    }
    fn next_unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / 9007199254740992.0 // 2^53
    }
}

/// splitmix64 -> 53-bit / 2^53 (fine grid).
struct Splitmix64 {
    state: u64,
}
impl Splitmix64 {
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    fn next_unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / 9007199254740992.0
    }
}

/// MT19937 (32-bit) with genrand_res53 -> 2^-53 grid. Excel's documented family.
struct Mt19937 {
    mt: [u32; 624],
    index: usize,
}
impl Mt19937 {
    fn new(seed: u32) -> Self {
        let mut mt = [0u32; 624];
        mt[0] = seed;
        for i in 1..624 {
            mt[i] = 1812433253u32
                .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        Self { mt, index: 624 }
    }
    fn generate(&mut self) {
        for i in 0..624 {
            let y = (self.mt[i] & 0x80000000) | (self.mt[(i + 1) % 624] & 0x7fffffff);
            let mut next = self.mt[(i + 397) % 624] ^ (y >> 1);
            if y & 1 != 0 {
                next ^= 0x9908b0df;
            }
            self.mt[i] = next;
        }
        self.index = 0;
    }
    fn next_u32(&mut self) -> u32 {
        if self.index >= 624 {
            self.generate();
        }
        let mut y = self.mt[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c5680;
        y ^= (y << 15) & 0xefc60000;
        y ^= y >> 18;
        y
    }
    fn next_unit(&mut self) -> f64 {
        let a = (self.next_u32() >> 5) as f64; // 27 bits
        let b = (self.next_u32() >> 6) as f64; // 26 bits
        (a * 67108864.0 + b) / 9007199254740992.0 // (a*2^26 + b)/2^53
    }
}

/// Wichmann-Hill (1982) — Excel pre-2010 RNG. Sum of three fractions mod 1.
struct WichmannHill {
    s1: i64,
    s2: i64,
    s3: i64,
}
impl WichmannHill {
    fn new(seed: u32) -> Self {
        Self {
            s1: ((seed % 30000) + 1) as i64,
            s2: (((seed / 30000) % 30000) + 1) as i64,
            s3: (((seed / 900000000) % 30000) + 1) as i64,
        }
    }
    fn next_unit(&mut self) -> f64 {
        self.s1 = (171 * self.s1) % 30269;
        self.s2 = (172 * self.s2) % 30307;
        self.s3 = (170 * self.s3) % 30323;
        let v = self.s1 as f64 / 30269.0 + self.s2 as f64 / 30307.0 + self.s3 as f64 / 30323.0;
        v - v.floor()
    }
}

// ----------------------------------------------------------------------------
// Statistics battery — computed identically for every source.
// ----------------------------------------------------------------------------
fn profile(samples: &[f64]) -> serde_json::Value {
    let n = samples.len();
    let nf = n as f64;
    let mean = samples.iter().sum::<f64>() / nf;
    let mut m2 = 0.0;
    let mut m3 = 0.0;
    let mut m4 = 0.0;
    for &x in samples {
        let d = x - mean;
        m2 += d * d;
        m3 += d * d * d;
        m4 += d * d * d * d;
    }
    let var = m2 / nf;
    let std = var.sqrt();
    let skew = (m3 / nf) / std.powi(3);
    let kurt_excess = (m4 / nf) / (var * var) - 3.0;

    // chi-square uniformity over K bins.
    let k = 64usize;
    let mut bins = vec![0usize; k];
    for &x in samples {
        let mut idx = (x * k as f64).floor() as isize;
        if idx < 0 {
            idx = 0;
        }
        if idx >= k as isize {
            idx = k as isize - 1;
        }
        bins[idx as usize] += 1;
    }
    let expected = nf / k as f64;
    let chi2 = bins
        .iter()
        .map(|&o| {
            let d = o as f64 - expected;
            d * d / expected
        })
        .sum::<f64>();

    // one-sample KS vs U(0,1): F(x) = x.
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut ks = 0.0f64;
    for (i, &x) in sorted.iter().enumerate() {
        let f_lo = i as f64 / nf;
        let f_hi = (i + 1) as f64 / nf;
        ks = ks.max((f_hi - x).abs()).max((x - f_lo).abs());
    }

    // lag-1 autocorrelation.
    let mut cov = 0.0;
    for w in samples.windows(2) {
        cov += (w[0] - mean) * (w[1] - mean);
    }
    let autocorr1 = (cov / (n - 1) as f64) / var;

    // granularity: fraction of draws lying exactly on the 2^-32 grid.
    // A coarse 32-bit RNG -> ~1.0; a 53-bit-mantissa RNG -> ~0.
    let two32 = 4294967296.0f64;
    let on_grid_2_32 = samples.iter().filter(|&&x| (x * two32).fract() == 0.0).count() as f64 / nf;

    // distinct ratio + smallest positive gap among sorted uniques.
    let mut distinct = 1usize;
    let mut min_gap = f64::INFINITY;
    for w in sorted.windows(2) {
        let g = w[1] - w[0];
        if g > 0.0 {
            distinct += 1;
            if g < min_gap {
                min_gap = g;
            }
        }
    }
    let min_positive = sorted.iter().copied().find(|&x| x > 0.0).unwrap_or(0.0);

    json!({
        "n": n,
        "mean": mean,
        "variance": var,
        "std": std,
        "skewness": skew,
        "excess_kurtosis": kurt_excess,
        "min": sorted.first().copied().unwrap_or(0.0),
        "max": sorted.last().copied().unwrap_or(0.0),
        "chi2_uniform_64": chi2,
        "chi2_df": k - 1,
        "ks_vs_uniform": ks,
        "lag1_autocorr": autocorr1,
        "frac_on_2pow32_grid": on_grid_2_32,
        "distinct_ratio": distinct as f64 / nf,
        "min_positive": min_positive,
        "min_gap": if min_gap.is_finite() { min_gap } else { 0.0 },
    })
}

/// Two-sample KS distance between two samples (max |F_a - F_b|).
fn ks_two_sample(a: &[f64], b: &[f64]) -> f64 {
    let mut sa = a.to_vec();
    let mut sb = b.to_vec();
    sa.sort_by(|x, y| x.partial_cmp(y).unwrap());
    sb.sort_by(|x, y| x.partial_cmp(y).unwrap());
    let (na, nb) = (sa.len() as f64, sb.len() as f64);
    let (mut i, mut j) = (0usize, 0usize);
    let mut d = 0.0f64;
    while i < sa.len() && j < sb.len() {
        let x = sa[i].min(sb[j]);
        while i < sa.len() && sa[i] <= x {
            i += 1;
        }
        while j < sb.len() && sb[j] <= x {
            j += 1;
        }
        let fa = i as f64 / na;
        let fb = j as f64 / nb;
        d = d.max((fa - fb).abs());
    }
    d
}

fn generate(name: &str, n: usize) -> Vec<f64> {
    let mut out = Vec::with_capacity(n);
    match name {
        "lcg64" => {
            let mut r = Lcg::new(0x0123_4567_89AB_CDEF);
            for _ in 0..n {
                out.push(r.next_unit());
            }
        }
        "xorshift128plus" => {
            let mut r = Xorshift128p::new(0x9E37_79B9_7F4A_7C15);
            for _ in 0..n {
                out.push(r.next_unit());
            }
        }
        "splitmix64" => {
            let mut r = Splitmix64 { state: 0xDEAD_BEEF_CAFE_F00D };
            for _ in 0..n {
                out.push(r.next_unit());
            }
        }
        "mt19937_res53" => {
            let mut r = Mt19937::new(5489);
            for _ in 0..n {
                out.push(r.next_unit());
            }
        }
        "wichmann_hill" => {
            let mut r = WichmannHill::new(123_456_789);
            for _ in 0..n {
                out.push(r.next_unit());
            }
        }
        _ => unreachable!(),
    }
    out
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mut excel_path: Option<PathBuf> = None;
    let mut out_path: Option<PathBuf> = None;
    let mut n_override: Option<usize> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--excel" => {
                i += 1;
                excel_path = args.get(i).map(PathBuf::from);
            }
            "--out" => {
                i += 1;
                out_path = args.get(i).map(PathBuf::from);
            }
            "--n" => {
                i += 1;
                n_override = args.get(i).and_then(|s| s.parse().ok());
            }
            other => return Err(format!("unknown arg: {other}")),
        }
        i += 1;
    }
    let excel_path = excel_path.ok_or("missing --excel")?;
    let out_path = out_path.ok_or("missing --out")?;

    let excel_text = fs::read_to_string(&excel_path).map_err(|e| format!("read excel: {e}"))?;
    let excel: Vec<f64> = excel_text
        .lines()
        .filter_map(|l| {
            let t = l.trim();
            if t.is_empty() {
                None
            } else {
                t.parse::<f64>().ok()
            }
        })
        .collect();
    if excel.is_empty() {
        return Err("excel sample file had no parseable f64 values".to_string());
    }
    let n = n_override.unwrap_or(excel.len());

    let rng_names = [
        "lcg64",
        "xorshift128plus",
        "splitmix64",
        "mt19937_res53",
        "wichmann_hill",
    ];

    let mut sources = serde_json::Map::new();
    sources.insert("excel_rand".to_string(), profile(&excel));
    let mut ks_vs_excel = serde_json::Map::new();
    for name in rng_names {
        let s = generate(name, n);
        ks_vs_excel.insert(name.to_string(), json!(ks_two_sample(&excel, &s)));
        sources.insert(name.to_string(), profile(&s));
    }

    let report = json!({
        "schema_version": "oxfunc.rand_characterization.v0",
        "purpose": "characterise Excel RAND() vs candidate Rust RNGs; assess statistical distinguishability (not a conformance/blessing test)",
        "excel_sample_count": excel.len(),
        "rng_sample_count": n,
        "theoretical_uniform": {"mean": 0.5, "variance": 1.0/12.0, "skewness": 0.0, "excess_kurtosis": -1.2},
        "sources": sources,
        "two_sample_ks_vs_excel": ks_vs_excel,
    });
    fs::write(&out_path, serde_json::to_string_pretty(&report).unwrap())
        .map_err(|e| format!("write out: {e}"))?;
    println!("rand characterization written: {}", out_path.display());
    Ok(())
}
