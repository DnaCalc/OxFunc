//! Last-bit PRICE accumulator graphs not in the 2026-08-09 residual racer:
//! Kahan / Neumaier / pairwise / accrue-first / two-sum remainder.
//! Uses already-captured discovery+companion banks. No Excel. Heldout unnamed.

use oxfunc_core::excel_numeric::research::excel_pow_chain;
use oxfunc_core::functions::bond_core_family::price_kernel;
use oxfunc_core::locale_format::{
    WorkbookDateSystem, excel_serial_from_ymd, ymd_from_excel_serial,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

const DISCOVERY: &str =
    "../../work/w109/G6-price-duration-exact/answers-price-residual-graph-discovery-20260809.json";
const COMPANION: &str = "../../work/w109/G6-price-duration-exact/answers-price-residual-companion-discovery-20260809.json";

#[derive(Deserialize)]
struct AnswerSet {
    function: String,
    witnesses: Vec<Witness>,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    args: Vec<String>,
    expected_bits: String,
}

struct Row {
    id: String,
    settlement: i64,
    maturity: i64,
    rate: f64,
    yld: f64,
    redemption: f64,
    frequency: i64,
    basis: i64,
    want: u64,
}

fn bits(text: &str) -> u64 {
    u64::from_str_radix(text.strip_prefix("0x").expect("0x"), 16).expect("hex")
}

fn ordered(value: u64) -> u64 {
    if value >> 63 == 0 {
        value | (1_u64 << 63)
    } else {
        !value
    }
}

fn signed_ulp(got: u64, want: u64) -> i64 {
    let got = i128::from(ordered(got));
    let want = i128::from(ordered(want));
    (got - want).clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

fn leap(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap(year) => 29,
        2 => 28,
        _ => panic!("month"),
    }
}

fn add_months(serial: i64, months: i64) -> i64 {
    let (year, month, day) =
        ymd_from_excel_serial(WorkbookDateSystem::System1900, serial as f64).expect("serial");
    let index = year * 12 + month - 1 + months;
    let target_year = index.div_euclid(12);
    let target_month = index.rem_euclid(12) + 1;
    let target_day = day.min(days_in_month(target_year, target_month));
    excel_serial_from_ymd(
        WorkbookDateSystem::System1900,
        target_year,
        target_month,
        target_day,
    )
    .expect("shift") as i64
}

struct Period {
    prev: i64,
    n: i64,
}

fn period(settlement: i64, maturity: i64, frequency: i64) -> Period {
    let months = 12 / frequency;
    let mut next = maturity;
    let mut n = 1_i64;
    loop {
        let prev = add_months(next, -months);
        if prev <= settlement {
            return Period { prev, n };
        }
        next = prev;
        n += 1;
    }
}

fn schedule(settlement: i64, maturity: i64, frequency: i64, basis: i64) -> (Period, f64, f64, f64) {
    assert!(matches!(basis, 2 | 3));
    let p = period(settlement, maturity, frequency);
    let e = if basis == 2 { 360.0 } else { 365.0 } / frequency as f64;
    let a = (settlement - p.prev) as f64;
    let off = (e - a) / e;
    (p, e, a, off)
}

fn binexp(base: f64, exponent: f64) -> f64 {
    let mut n = exponent as u64;
    let mut result = 1.0;
    let mut factor = base;
    while n > 0 {
        if n & 1 == 1 {
            result *= factor;
        }
        n >>= 1;
        if n > 0 {
            factor *= factor;
        }
    }
    result
}

fn disc(base: f64, exponent: f64) -> f64 {
    if exponent >= 0.0 && exponent < 1024.0 && exponent.fract() == 0.0 {
        binexp(base, exponent)
    } else {
        excel_pow_chain(base, exponent)
    }
}

fn load(path: &str) -> Vec<Row> {
    let text = std::fs::read_to_string(Path::new(path)).expect(path);
    let set: AnswerSet = serde_json::from_str(&text).expect("json");
    assert_eq!(set.function, "PRICE");
    set.witnesses
        .into_iter()
        .map(|w| {
            let args: Vec<f64> = w.args.iter().map(|a| f64::from_bits(bits(a))).collect();
            Row {
                id: w.id,
                settlement: args[0] as i64,
                maturity: args[1] as i64,
                rate: args[2],
                yld: args[3],
                redemption: args[4],
                frequency: args[5] as i64,
                basis: args[6] as i64,
                want: bits(&w.expected_bits),
            }
        })
        .filter(|r| matches!(r.basis, 2 | 3))
        .collect()
}

fn kahan_sum(xs: &[f64]) -> f64 {
    let mut s = 0.0;
    let mut c = 0.0;
    for &x in xs {
        let y = x - c;
        let t = s + y;
        c = (t - s) - y;
        s = t;
    }
    s
}

fn neumaier_sum(xs: &[f64]) -> f64 {
    let mut s = 0.0;
    let mut c = 0.0;
    for &x in xs {
        let t = s + x;
        if s.abs() >= x.abs() {
            c += (s - t) + x;
        } else {
            c += (x - t) + s;
        }
        s = t;
    }
    s + c
}

fn pairwise_sum(xs: &[f64]) -> f64 {
    match xs.len() {
        0 => 0.0,
        1 => xs[0],
        2 => xs[0] + xs[1],
        n => {
            let m = n / 2;
            pairwise_sum(&xs[..m]) + pairwise_sum(&xs[m..])
        }
    }
}

fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let v = s - a;
    let e = (a - (s - v)) + (b - v);
    (s, e)
}

fn terms(row: &Row) -> Option<(Vec<f64>, f64)> {
    let (p, e, a, off) = schedule(row.settlement, row.maturity, row.frequency, row.basis);
    if p.n <= 1 {
        return None;
    }
    let f = row.frequency as f64;
    let coup = 100.0 * row.rate / f;
    let base = 1.0 + row.yld / f;
    let mut ts = Vec::with_capacity((p.n + 1) as usize);
    for k in 0..p.n {
        ts.push(coup / disc(base, off + k as f64));
    }
    ts.push(row.redemption / disc(base, off + (p.n - 1) as f64));
    let accr = coup * (a / e);
    Some((ts, accr))
}

struct Score {
    exact: usize,
    total: usize,
    max_ulp: u64,
    sum_ulp: u128,
    signed: BTreeMap<i64, usize>,
}

impl Score {
    fn new() -> Self {
        Self {
            exact: 0,
            total: 0,
            max_ulp: 0,
            sum_ulp: 0,
            signed: BTreeMap::new(),
        }
    }
    fn add(&mut self, got: f64, want: u64) {
        let s = signed_ulp(got.to_bits(), want);
        let d = s.unsigned_abs();
        self.total += 1;
        if d == 0 {
            self.exact += 1;
        }
        self.max_ulp = self.max_ulp.max(d);
        self.sum_ulp += u128::from(d);
        *self.signed.entry(s.clamp(-8, 8)).or_default() += 1;
    }
    fn show(&self, label: &str) {
        println!(
            "{label:<42} {:>3}/{:<3} max={} sum={} signed={:?}",
            self.exact, self.total, self.max_ulp, self.sum_ulp, self.signed
        );
    }
}

fn race(rows: &[Row], label: &str, eval: impl Fn(&Row) -> Option<f64>) {
    let mut sc = Score::new();
    for row in rows {
        if let Some(got) = eval(row) {
            sc.add(got, row.want);
        }
    }
    sc.show(label);
}

fn main() {
    let mut rows = load(DISCOVERY);
    rows.extend(load(COMPANION));
    println!("rows {}", rows.len());

    race(&rows, "production price_kernel", |r| {
        price_kernel(
            r.settlement as f64,
            r.maturity as f64,
            r.rate,
            r.yld,
            r.redemption,
            r.frequency as f64,
            Some(r.basis as f64),
        )
        .ok()
    });

    race(&rows, "native loop (pcomp_disc replica)", |r| {
        let (ts, accr) = terms(r)?;
        Some(ts.iter().fold(0.0, |s, &t| s + t) - accr)
    });

    race(&rows, "kahan then -accr", |r| {
        let (ts, accr) = terms(r)?;
        Some(kahan_sum(&ts) - accr)
    });
    race(&rows, "neumaier then -accr", |r| {
        let (ts, accr) = terms(r)?;
        Some(neumaier_sum(&ts) - accr)
    });
    race(&rows, "pairwise then -accr", |r| {
        let (ts, accr) = terms(r)?;
        Some(pairwise_sum(&ts) - accr)
    });
    race(&rows, "kahan including -accr term", |r| {
        let (mut ts, accr) = terms(r)?;
        ts.push(-accr);
        Some(kahan_sum(&ts))
    });
    race(&rows, "neumaier including -accr term", |r| {
        let (mut ts, accr) = terms(r)?;
        ts.push(-accr);
        Some(neumaier_sum(&ts))
    });
    race(&rows, "reverse native then -accr", |r| {
        let (ts, accr) = terms(r)?;
        Some(ts.iter().rev().fold(0.0, |s, &t| s + t) - accr)
    });
    race(&rows, "two-sum chain then -accr", |r| {
        let (ts, accr) = terms(r)?;
        let mut s = 0.0;
        let mut e = 0.0;
        for &t in &ts {
            let (ns, ne) = two_sum(s, t);
            s = ns;
            e += ne;
        }
        Some((s + e) - accr)
    });
    race(&rows, "accr-first native", |r| {
        let (ts, accr) = terms(r)?;
        Some(ts.iter().fold(-accr, |s, &t| s + t))
    });
    race(&rows, "kahan accr-first", |r| {
        let (ts, accr) = terms(r)?;
        let mut xs = vec![-accr];
        xs.extend_from_slice(&ts);
        Some(kahan_sum(&xs))
    });
    race(&rows, "native (sum-accr) via two_sum final", |r| {
        let (ts, accr) = terms(r)?;
        let s = ts.iter().fold(0.0, |s, &t| s + t);
        let (q, e) = two_sum(s, -accr);
        Some(q + e)
    });
    // redemption last vs already in terms; drop last and add with two_sum
    race(&rows, "coupons kahan + redemption two_sum -accr", |r| {
        let (ts, accr) = terms(r)?;
        let n = ts.len();
        let coup_sum = kahan_sum(&ts[..n - 1]);
        let (s, e) = two_sum(coup_sum, ts[n - 1]);
        Some((s + e) - accr)
    });
    fn horner(row: &Row, fold_red: bool, divide_off_last: bool, accr_ratio: bool) -> Option<f64> {
        let (p, e, a, off) = schedule(row.settlement, row.maturity, row.frequency, row.basis);
        if p.n <= 1 {
            return None;
        }
        let f = row.frequency as f64;
        let coup = 100.0 * row.rate / f;
        let base = 1.0 + row.yld / f;
        let mut pv = if fold_red {
            row.redemption + coup
        } else {
            row.redemption
        };
        let loops = if fold_red { p.n - 1 } else { p.n };
        for _ in 0..loops {
            pv = coup + pv / base;
        }
        if !fold_red {
            pv += coup;
        }
        let scale = if divide_off_last {
            disc(base, off)
        } else {
            1.0
        };
        let dirty = if divide_off_last {
            pv / scale
        } else {
            pv / disc(base, off)
        };
        let accr = if accr_ratio {
            coup * (a / e)
        } else {
            coup * a / e
        };
        Some(dirty - accr)
    }
    race(&rows, "horner fold-red /base^off last ratio-accr", |r| {
        horner(r, true, true, true)
    });
    race(&rows, "horner fold-red /base^off last prod-accr", |r| {
        horner(r, true, true, false)
    });
    race(&rows, "horner nofold /base^off last ratio-accr", |r| {
        horner(r, false, true, true)
    });
    fn geom(row: &Row, pow_n: bool, inv_first: bool) -> Option<f64> {
        let (p, e, a, off) = schedule(row.settlement, row.maturity, row.frequency, row.basis);
        if p.n <= 1 {
            return None;
        }
        let f = row.frequency as f64;
        let coup = 100.0 * row.rate / f;
        let base = 1.0 + row.yld / f;
        let n = p.n as f64;
        let r = if inv_first {
            1.0 / base
        } else {
            disc(base, -1.0)
        };
        let rn = if pow_n {
            disc(base, -n)
        } else {
            r.powi(p.n as i32)
        };
        let geom = (1.0 - rn) / (1.0 - r);
        let head = disc(base, off);
        let coupon_pv = coup * geom / head;
        let red_pv = row.redemption / disc(base, off + (p.n - 1) as f64);
        Some(coupon_pv + red_pv - coup * (a / e))
    }
    race(&rows, "geom pow_chain(-n) 1/base ratio-accr", |r| {
        geom(r, true, true)
    });
    race(&rows, "geom r.powi(n) 1/base ratio-accr", |r| {
        geom(r, false, true)
    });
    race(&rows, "geom pow_chain(-n) disc(base,-1)", |r| {
        geom(r, true, false)
    });

    race(&rows, "native (coup*a)/e like production accr", |r| {
        let (p, e, a, off) = schedule(r.settlement, r.maturity, r.frequency, r.basis);
        if p.n <= 1 {
            return None;
        }
        let f = r.frequency as f64;
        let coup = 100.0 * r.rate / f;
        let base = 1.0 + r.yld / f;
        let mut s = 0.0;
        for k in 0..p.n {
            s += coup / disc(base, off + k as f64);
        }
        s += r.redemption / disc(base, off + (p.n - 1) as f64);
        Some(s - (coup * a / e))
    });

    println!("\n## replica misses (native loop coup*(a/e))");
    let mut miss_rows = Vec::new();
    for r in &rows {
        let Some((ts, accr)) = terms(r) else { continue };
        let got = ts.iter().fold(0.0, |s, &t| s + t) - accr;
        let s = signed_ulp(got.to_bits(), r.want);
        if s != 0 {
            let (p, e, a, off) = schedule(r.settlement, r.maturity, r.frequency, r.basis);
            println!(
                "  {} sulp={s} n={} red={} rate={} yld={} f={} basis={} a={} e={} off={off}",
                r.id, p.n, r.redemption, r.rate, r.yld, r.frequency, r.basis, a, e
            );
            miss_rows.push(r);
        }
    }
    println!("misses {}", miss_rows.len());
    let dump = "../../work/w109/lastbit-12h-20260912/price-lastbit/replica-misses.json";
    let _ = std::fs::create_dir_all("../../work/w109/lastbit-12h-20260912/price-lastbit");
    #[derive(serde::Serialize)]
    struct DumpRow {
        id: String,
        settlement: i64,
        maturity: i64,
        rate_bits: String,
        yld_bits: String,
        redemption_bits: String,
        frequency: i64,
        basis: i64,
        want: String,
        n: i64,
        a: f64,
        e: f64,
        off_bits: String,
        coup_bits: String,
        base_bits: String,
        terms: Vec<String>,
        accr_bits: String,
        got_bits: String,
    }
    let dumps: Vec<DumpRow> = miss_rows
        .iter()
        .map(|r| {
            let (p, e, a, off) = schedule(r.settlement, r.maturity, r.frequency, r.basis);
            let f = r.frequency as f64;
            let coup = 100.0 * r.rate / f;
            let base = 1.0 + r.yld / f;
            let (ts, accr) = terms(r).unwrap();
            let got = ts.iter().fold(0.0, |s, &t| s + t) - accr;
            DumpRow {
                id: r.id.clone(),
                settlement: r.settlement,
                maturity: r.maturity,
                rate_bits: format!("0x{:016x}", r.rate.to_bits()),
                yld_bits: format!("0x{:016x}", r.yld.to_bits()),
                redemption_bits: format!("0x{:016x}", r.redemption.to_bits()),
                frequency: r.frequency,
                basis: r.basis,
                want: format!("0x{:016x}", r.want),
                n: p.n,
                a,
                e,
                off_bits: format!("0x{:016x}", off.to_bits()),
                coup_bits: format!("0x{:016x}", coup.to_bits()),
                base_bits: format!("0x{:016x}", base.to_bits()),
                terms: ts
                    .iter()
                    .map(|t| format!("0x{:016x}", t.to_bits()))
                    .collect(),
                accr_bits: format!("0x{:016x}", accr.to_bits()),
                got_bits: format!("0x{:016x}", got.to_bits()),
            }
        })
        .collect();
    std::fs::write(dump, serde_json::to_string_pretty(&dumps).unwrap()).unwrap();
    println!("wrote {dump}");
}
