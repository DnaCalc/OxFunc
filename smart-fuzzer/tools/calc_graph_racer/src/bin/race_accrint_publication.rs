//! W109 G6-02: race ACCRINT's final coupon/publication staging on the
//! current-build b43 rate ladder.
//!
//! The calendar/day-count graph is already identified.  Calling the production
//! kernel with `rate=1` and `par=frequency` publishes that graph's accrual
//! fraction unchanged; this tool then races only mathematically equivalent
//! final graphs, including register-continuous x87 variants.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::bond_core_family::accrint_kernel;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Deserialize)]
struct WitnessSet {
    function: String,
    witnesses: Vec<Witness>,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    args: Vec<String>,
    expected_bits: String,
}

#[derive(Deserialize)]
struct ProbeBatch {
    function: String,
    probes: Vec<ProbeEnvelope>,
}

#[derive(Deserialize)]
struct ProbeEnvelope {
    probe: Probe,
}

#[derive(Deserialize)]
struct Probe {
    id: String,
    args: Vec<String>,
}

fn h(s: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).expect("hex f64"))
}

fn expected(s: &str) -> u64 {
    u64::from_str_radix(s.trim_start_matches("0x"), 16).expect("expected hex")
}

fn spill(e: &rx::Ext80, cw: u16) -> rx::Ext80 {
    rx::ext_from_f64(rx::ext_to_f64(e, cw))
}

/// Recover the small calendar rational represented by a published `f64`.
/// All W109 ACCRINT terms have denominators derived from coupon lengths, so a
/// one-million denominator cap is deliberately generous.
fn limit_denominator(value: f64, max_den: i128) -> (i128, i128) {
    if value == 0.0 {
        return (0, 1);
    }
    let sign = if value < 0.0 { -1 } else { 1 };
    let target = value.abs();
    let mut x = target;
    let (mut p0, mut q0) = (0_i128, 1_i128);
    let (mut p1, mut q1) = (1_i128, 0_i128);
    loop {
        let a = x.floor() as i128;
        let p2 = p0 + a * p1;
        let q2 = q0 + a * q1;
        if q2 > max_den {
            let k = (max_den - q0) / q1;
            let b1 = (p0 + k * p1, q0 + k * q1);
            let b2 = (p1, q1);
            let e1 = ((b1.0 as f64 / b1.1 as f64) - target).abs();
            let e2 = ((b2.0 as f64 / b2.1 as f64) - target).abs();
            let (p, q) = if e2 <= e1 { b2 } else { b1 };
            return (sign * p, q);
        }
        (p0, q0, p1, q1) = (p1, q1, p2, q2);
        let frac = x - a as f64;
        if frac == 0.0 {
            return (sign * p1, q1);
        }
        x = 1.0 / frac;
    }
}

fn ext_continuous(par: f64, rate: f64, frequency: f64, a: f64, cw: u16) -> f64 {
    let p = rx::ext_mul(&rx::ext_from_f64(par), &rx::ext_from_f64(rate), cw);
    let c = rx::ext_div(&p, &rx::ext_from_f64(frequency), cw);
    let r = rx::ext_mul(&c, &rx::ext_from_f64(a), cw);
    rx::ext_to_f64(&r, cw)
}

fn ext_division_continuous(
    par: f64,
    rate: f64,
    frequency: f64,
    num: i128,
    den: i128,
    cw: u16,
) -> f64 {
    let p = rx::ext_mul(&rx::ext_from_f64(par), &rx::ext_from_f64(rate), cw);
    let c = rx::ext_div(&p, &rx::ext_from_f64(frequency), cw);
    let n = rx::ext_mul(&c, &rx::ext_from_f64(num as f64), cw);
    let r = rx::ext_div(&n, &rx::ext_from_f64(den as f64), cw);
    rx::ext_to_f64(&r, cw)
}

fn ext_fraction_first(
    par: f64,
    rate: f64,
    frequency: f64,
    num: i128,
    den: i128,
    cw: u16,
    store_fraction: bool,
    store_coupon: bool,
) -> f64 {
    let mut a = rx::ext_div(
        &rx::ext_from_f64(num as f64),
        &rx::ext_from_f64(den as f64),
        cw,
    );
    if store_fraction {
        a = spill(&a, cw);
    }
    let p = rx::ext_mul(&rx::ext_from_f64(par), &rx::ext_from_f64(rate), cw);
    let mut c = rx::ext_div(&p, &rx::ext_from_f64(frequency), cw);
    if store_coupon {
        c = spill(&c, cw);
    }
    rx::ext_to_f64(&rx::ext_mul(&c, &a, cw), cw)
}

fn validate_heldout_companions(path: &std::path::Path, doc: &WitnessSet) -> BTreeMap<String, u64> {
    let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
        return BTreeMap::new();
    };
    if name != "answers-accrint-publication-heldout-20260809.json" {
        return BTreeMap::new();
    }
    let dir = path.parent().expect("answer parent");
    let batch_path = dir.join("batch-accrint-publication-heldout-20260809.json");
    let meta_path = dir.join("meta-accrint-publication-heldout-20260809.csv");
    let batch: ProbeBatch =
        serde_json::from_slice(&std::fs::read(&batch_path).expect("read held-out batch companion"))
            .expect("held-out batch json");
    assert_eq!(batch.function, doc.function);
    assert_eq!(batch.probes.len(), doc.witnesses.len());
    for (probe, witness) in batch.probes.iter().zip(&doc.witnesses) {
        assert_eq!(probe.probe.id, witness.id, "held-out id alignment");
        assert_eq!(
            probe.probe.args, witness.args,
            "{} arg alignment",
            witness.id
        );
    }

    let text = std::fs::read_to_string(&meta_path).expect("read held-out meta companion");
    let mut predictions = BTreeMap::new();
    for (line_no, line) in text.lines().enumerate() {
        if line_no == 0 {
            assert_eq!(
                line,
                "id,class,regime,basis,frequency,calc_method,plain_bits,candidate_bits"
            );
            continue;
        }
        let columns: Vec<&str> = line.split(',').collect();
        assert_eq!(columns.len(), 8, "meta line {}", line_no + 1);
        let candidate = expected(columns[7]);
        assert!(
            predictions
                .insert(columns[0].to_string(), candidate)
                .is_none(),
            "duplicate meta id {}",
            columns[0]
        );
    }
    assert_eq!(predictions.len(), doc.witnesses.len());
    predictions
}

fn main() {
    let default_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../work/w109/G6-b2b3/answers-b43-accrint-build20228-20260809.json");
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or(default_path);
    let doc: WitnessSet =
        serde_json::from_slice(&std::fs::read(&path).expect("read witness set")).expect("json");
    assert_eq!(doc.function, "ACCRINT");
    let heldout_predictions = validate_heldout_companions(&path, &doc);

    let mut scores: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut production_misses = Vec::new();
    for w in &doc.witnesses {
        assert_eq!(w.args.len(), 8, "{} arity", w.id);
        let a: Vec<f64> = w.args.iter().map(|v| h(v)).collect();
        let want = expected(&w.expected_bits);
        let got = accrint_kernel(
            a[0],
            a[1],
            a[2],
            a[3],
            Some(a[4]),
            a[5],
            Some(a[6]),
            Some(a[7] != 0.0),
        )
        .expect("valid ACCRINT row");
        let af = accrint_kernel(
            a[0],
            a[1],
            a[2],
            1.0,
            Some(a[5]),
            a[5],
            Some(a[6]),
            Some(a[7] != 0.0),
        )
        .expect("accrual fraction");
        let (num, den) = limit_denominator(af, 1_000_000);
        let rational_recovery_exact = af == num as f64 / den as f64;

        let candidates = [
            ("production", got),
            ("plain reassoc p*(r*a)/f", a[4] * (a[3] * af) / a[5]),
            ("plain reassoc (p*a)*r/f", (a[4] * af) * a[3] / a[5]),
            (
                "x87 PC64 stored-a continuous",
                ext_continuous(a[4], a[3], a[5], af, rx::CW_PC64_RN),
            ),
            (
                "x87 final DR stored coupon/a",
                rx::x87_mul(a[4] * a[3] / a[5], af),
            ),
            (
                "x87 PC53 stored-a continuous",
                ext_continuous(a[4], a[3], a[5], af, rx::CW_PC53_RN),
            ),
            (
                "x87 PC64 rational-a coupon-first",
                ext_division_continuous(a[4], a[3], a[5], num, den, rx::CW_PC64_RN),
            ),
            (
                "x87 PC53 rational-a coupon-first",
                ext_division_continuous(a[4], a[3], a[5], num, den, rx::CW_PC53_RN),
            ),
            (
                "x87 PC64 rational-a fraction-first",
                ext_fraction_first(a[4], a[3], a[5], num, den, rx::CW_PC64_RN, false, false),
            ),
            (
                "x87 PC64 fraction stored",
                ext_fraction_first(a[4], a[3], a[5], num, den, rx::CW_PC64_RN, true, false),
            ),
            (
                "x87 PC64 coupon stored",
                ext_fraction_first(a[4], a[3], a[5], num, den, rx::CW_PC64_RN, false, true),
            ),
            (
                "x87 PC64 both stored",
                ext_fraction_first(a[4], a[3], a[5], num, den, rx::CW_PC64_RN, true, true),
            ),
        ];
        if let Some(predicted) = heldout_predictions.get(&w.id) {
            assert_eq!(
                rx::x87_mul(a[4] * a[3] / a[5], af).to_bits(),
                *predicted,
                "{} frozen candidate prediction",
                w.id
            );
        }
        for (name, value) in candidates {
            if value.to_bits() == want {
                *scores.entry(name).or_default() += 1;
            }
        }
        if got.to_bits() != want {
            production_misses.push((
                w.id.clone(),
                want,
                got.to_bits(),
                num,
                den,
                rational_recovery_exact,
                candidates,
            ));
        }
    }

    println!("{} rows from {}", doc.witnesses.len(), path.display());
    for (name, score) in scores.iter().rev() {
        println!("{score:4}/{} {name}", doc.witnesses.len());
    }
    println!("\nproduction misses {}", production_misses.len());
    for (id, want, got, num, den, rational_recovery_exact, candidates) in production_misses {
        println!(
            "\n{id}: want={want:016x} got={got:016x} a~={num}/{den} rational_exact={rational_recovery_exact}"
        );
        let mut groups: BTreeMap<u64, Vec<&str>> = BTreeMap::new();
        for (name, value) in candidates {
            groups.entry(value.to_bits()).or_default().push(name);
        }
        for (bits, names) in groups {
            let mark = if bits == want { " MATCH" } else { "" };
            println!("  {bits:016x}{mark}: {}", names.join(", "));
        }
    }
}
