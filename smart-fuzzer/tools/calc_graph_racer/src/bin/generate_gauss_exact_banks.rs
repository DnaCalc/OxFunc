//! Deterministic answer-blind GAUSS discovery and held-out bank generator.
//!
//! The two sets use independent fixed seeds and are made disjoint by exact
//! input bits.  They contain no candidate outputs and no oracle answers.  The
//! discovery bank is intended to be captured and scored first; the held-out
//! answers must remain sealed until one coherent discovery graph survives.
//!
//! Usage:
//!   generate_gauss_exact_banks <output-directory>
//!   generate_gauss_exact_banks <output-directory> route

use serde::Serialize;
use std::collections::BTreeMap;

const DISCOVERY_COUNT: usize = 8_192;
const HELDOUT_COUNT: usize = 4_096;
const ROUTE_DISCOVERY_COUNT: usize = 1_024;
const ROUTE_HELDOUT_COUNT: usize = 512;
const DISCOVERY_SEED: u64 = 0x4741_5553_535f_4431;
const HELDOUT_SEED: u64 = 0x4741_5553_535f_4831;
const ROUTE_DISCOVERY_SEED: u64 = 0x4741_5553_535f_5244;
const ROUTE_HELDOUT_SEED: u64 = 0x4741_5553_535f_5248;

#[derive(Serialize)]
struct Probe {
    id: String,
    args: [String; 1],
}

#[derive(Serialize)]
struct ProbeEnvelope {
    probe: Probe,
    distinct_outputs: usize,
    outputs: Vec<String>,
}

#[derive(Serialize)]
struct ProbeBatch {
    function: &'static str,
    row_id: &'static str,
    bank_role: &'static str,
    generator: &'static str,
    seed_hex: String,
    probes: Vec<ProbeEnvelope>,
}

#[derive(Clone, Copy)]
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

fn insert(rows: &mut BTreeMap<u64, String>, value: f64, class: &str) {
    if value.is_finite() {
        rows.entry(value.to_bits())
            .or_insert_with(|| class.to_string());
    }
}

fn next_up(x: f64) -> f64 {
    if x.is_nan() || x == f64::INFINITY {
        x
    } else if x == 0.0 {
        f64::from_bits(1)
    } else if x > 0.0 {
        f64::from_bits(x.to_bits() + 1)
    } else {
        f64::from_bits(x.to_bits() - 1)
    }
}

fn next_down(x: f64) -> f64 {
    if x.is_nan() || x == f64::NEG_INFINITY {
        x
    } else if x == 0.0 {
        -f64::from_bits(1)
    } else if x > 0.0 {
        f64::from_bits(x.to_bits() - 1)
    } else {
        f64::from_bits(x.to_bits() + 1)
    }
}

fn add_window(rows: &mut BTreeMap<u64, String>, center: f64, radius: usize, class: &str) {
    insert(rows, center, class);
    let (mut lo, mut hi) = (center, center);
    for _ in 0..radius {
        lo = next_down(lo);
        hi = next_up(hi);
        insert(rows, lo, class);
        insert(rows, hi, class);
    }
}

fn add_structured(rows: &mut BTreeMap<u64, String>, discovery: bool) {
    // Signed zero, the stable G3-07 witness, and the only non-exact legacy
    // survivor are pinned explicitly.
    insert(rows, 0.0, "zero");
    insert(rows, -0.0, "zero");
    for &v in &[1.0, -1.0, -1.000_954_812_153_151_3] {
        add_window(rows, v, if discovery { 128 } else { 193 }, "unit_boundary");
    }

    // Every representable power-of-two scale from the minimum subnormal to
    // 16, both signs.  This resolves the ultra-tiny route without assuming a
    // threshold and samples every normal exponent relevant before saturation.
    for bit in 0..52 {
        let v = f64::from_bits(1u64 << bit);
        insert(rows, v, "subnormal_power");
        insert(rows, -v, "subnormal_power");
    }
    for exponent_field in 1u64..=1027 {
        let v = f64::from_bits(exponent_field << 52);
        insert(rows, v, "normal_power");
        insert(rows, -v, "normal_power");
    }

    // Public erf/erfc branch landmarks transported through z=x/sqrt(2), plus
    // cancellation and saturation landmarks.  ULP windows distinguish divide
    // from multiply input formation and neighboring branch predicates.
    let root_two = std::f64::consts::SQRT_2;
    let landmarks = [
        f64::EPSILON,
        f64::EPSILON.sqrt(),
        1e-12,
        1e-9,
        1e-6,
        1e-3,
        0.125 * root_two,
        0.5 * root_two,
        0.84375 * root_two,
        1.25 * root_two,
        2.0 * root_two,
        2.857 * root_two,
        4.0 * root_two,
        6.0 * root_two,
        8.0,
        8.125,
        8.25,
    ];
    for &v in &landmarks {
        let radius = if discovery { 64 } else { 111 };
        add_window(rows, v, radius, "transported_landmark");
        add_window(rows, -v, radius, "transported_landmark");
    }
}

fn random_value(rng: &mut SplitMix64, mode: u64) -> f64 {
    match mode % 5 {
        // Uniformly cover binary exponents through the informative GAUSS range.
        0 | 1 => {
            let raw = rng.next();
            let exponent = raw % 1028;
            let mantissa = rng.next() & 0x000f_ffff_ffff_ffff;
            let sign = rng.next() & (1u64 << 63);
            f64::from_bits(sign | (exponent << 52) | mantissa)
        }
        // Dense moderate values, where erfc body and tail branches are visible.
        2 => {
            let unit = (rng.next() >> 11) as f64 * (1.0 / ((1u64 << 53) as f64));
            (unit * 20.0) - 10.0
        }
        // Dense cancellation band around zero.
        3 => {
            let unit = (rng.next() >> 11) as f64 * (1.0 / ((1u64 << 53) as f64));
            let sign = if rng.next() & 1 == 0 { 1.0 } else { -1.0 };
            sign * unit * 1e-3
        }
        // ULP-neighborhoods around the x=+/-1 transported erf branch.
        _ => {
            let signed_offset = (rng.next() % 2_000_001) as i64 - 1_000_000;
            let negative = rng.next() & 1 != 0;
            let base = if negative {
                (-1.0f64).to_bits()
            } else {
                1.0f64.to_bits()
            };
            let distance = signed_offset.unsigned_abs();
            let bits = if (signed_offset >= 0) ^ negative {
                base + distance
            } else {
                base - distance
            };
            f64::from_bits(bits)
        }
    }
}

fn build_set(
    count: usize,
    seed: u64,
    discovery: bool,
    forbidden: Option<&BTreeMap<u64, String>>,
) -> BTreeMap<u64, String> {
    let mut rows = BTreeMap::new();
    add_structured(&mut rows, discovery);
    if let Some(forbidden) = forbidden {
        rows.retain(|bits, _| !forbidden.contains_key(bits));
    }
    let mut rng = SplitMix64(seed);
    let mut attempt = 0u64;
    while rows.len() < count {
        let value = random_value(&mut rng, attempt);
        attempt += 1;
        if !value.is_finite() || value.abs() > 40.0 {
            continue;
        }
        let bits = value.to_bits();
        if forbidden.is_some_and(|set| set.contains_key(&bits)) {
            continue;
        }
        insert(&mut rows, value, "seeded_random");
    }
    while rows.len() > count {
        rows.pop_last();
    }
    rows
}

fn insert_route_pair(
    rows: &mut BTreeMap<u64, String>,
    value: f64,
    class: &str,
    forbidden: &BTreeMap<u64, String>,
    count: usize,
) {
    if rows.len() + 2 > count || !value.is_finite() || value <= 0.0 {
        return;
    }
    let positive = value.to_bits();
    let negative = positive | (1u64 << 63);
    if forbidden.contains_key(&positive)
        || forbidden.contains_key(&negative)
        || rows.contains_key(&positive)
        || rows.contains_key(&negative)
    {
        return;
    }
    rows.insert(positive, class.to_string());
    rows.insert(negative, class.to_string());
}

fn build_route_discovery(forbidden: &BTreeMap<u64, String>) -> BTreeMap<u64, String> {
    let mut rows = BTreeMap::new();
    let center = 1e-15f64;
    let center_bits = center.to_bits();
    for offset in -255i64..=255 {
        let bits = (center_bits as i64 + offset) as u64;
        insert_route_pair(
            &mut rows,
            f64::from_bits(bits),
            "decimal_1e-15_ulp_window",
            forbidden,
            ROUTE_DISCOVERY_COUNT,
        );
    }
    insert_route_pair(
        &mut rows,
        5.0 * f64::EPSILON,
        "five_epsilon_anchor",
        forbidden,
        ROUTE_DISCOVERY_COUNT,
    );

    // This loop is normally unnecessary (the explicit set is 1,024 rows),
    // but keeps the frozen count stable if a future base bank happens to
    // overlap one of the route probes.
    let lo = (4.0 * f64::EPSILON).to_bits();
    let hi = (8.0 * f64::EPSILON).to_bits();
    let mut rng = SplitMix64(ROUTE_DISCOVERY_SEED);
    while rows.len() < ROUTE_DISCOVERY_COUNT {
        let bits = lo + rng.next() % (hi - lo + 1);
        insert_route_pair(
            &mut rows,
            f64::from_bits(bits),
            "seeded_route_fill",
            forbidden,
            ROUTE_DISCOVERY_COUNT,
        );
    }
    rows
}

fn build_route_heldout(forbidden: &BTreeMap<u64, String>) -> BTreeMap<u64, String> {
    let mut rows = BTreeMap::new();
    let lo = (4.0 * f64::EPSILON).to_bits();
    let hi = (8.0 * f64::EPSILON).to_bits();
    let mut rng = SplitMix64(ROUTE_HELDOUT_SEED);
    while rows.len() < ROUTE_HELDOUT_COUNT {
        let bits = lo + rng.next() % (hi - lo + 1);
        insert_route_pair(
            &mut rows,
            f64::from_bits(bits),
            "seeded_route_heldout",
            forbidden,
            ROUTE_HELDOUT_COUNT,
        );
    }
    rows
}

fn batch(rows: &BTreeMap<u64, String>, role: &'static str, seed: u64) -> ProbeBatch {
    let probes = rows
        .iter()
        .enumerate()
        .map(|(index, (&bits, class))| ProbeEnvelope {
            probe: Probe {
                id: format!("gauss-{role}-{index:05}-{class}-0x{bits:016x}"),
                args: [format!("0x{bits:016x}")],
            },
            distinct_outputs: 0,
            outputs: Vec::new(),
        })
        .collect();
    ProbeBatch {
        function: "GAUSS",
        row_id: "G3-07-gauss",
        bank_role: role,
        generator: "generate_gauss_exact_banks/v1",
        seed_hex: format!("0x{seed:016x}"),
        probes,
    }
}

fn write_batch(path: &std::path::Path, batch: &ProbeBatch) {
    let mut text = serde_json::to_string_pretty(batch).expect("serialize batch");
    text.push('\n');
    std::fs::write(path, text)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
}

fn main() {
    let out_dir = std::env::args().nth(1).expect("output directory");
    let mode = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "exact".to_string());
    let out_dir = std::path::Path::new(&out_dir);
    std::fs::create_dir_all(out_dir)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", out_dir.display()));

    let discovery = build_set(DISCOVERY_COUNT, DISCOVERY_SEED, true, None);
    let heldout = build_set(HELDOUT_COUNT, HELDOUT_SEED, false, Some(&discovery));
    assert_eq!(discovery.len(), DISCOVERY_COUNT);
    assert_eq!(heldout.len(), HELDOUT_COUNT);
    assert!(discovery.keys().all(|bits| !heldout.contains_key(bits)));

    if mode == "route" {
        let mut forbidden = discovery.clone();
        forbidden.extend(heldout.clone());
        let route_discovery = build_route_discovery(&forbidden);
        forbidden.extend(route_discovery.clone());
        let route_heldout = build_route_heldout(&forbidden);
        assert_eq!(route_discovery.len(), ROUTE_DISCOVERY_COUNT);
        assert_eq!(route_heldout.len(), ROUTE_HELDOUT_COUNT);
        assert!(
            route_discovery
                .keys()
                .all(|bits| !route_heldout.contains_key(bits))
        );
        let discovery_path = out_dir.join("batch-gauss-route-discovery-v1.json");
        let heldout_path = out_dir.join("batch-gauss-route-heldout-v1.json");
        write_batch(
            &discovery_path,
            &batch(&route_discovery, "route-discovery-v1", ROUTE_DISCOVERY_SEED),
        );
        write_batch(
            &heldout_path,
            &batch(&route_heldout, "route-heldout-v1", ROUTE_HELDOUT_SEED),
        );
        println!(
            "route_discovery={} {}\nroute_heldout={} {}\ndisjoint_from_base_and_each_other=true; answers absent",
            route_discovery.len(),
            discovery_path.display(),
            route_heldout.len(),
            heldout_path.display()
        );
        return;
    }
    assert_eq!(mode, "exact", "mode must be exact|route");

    let discovery_path = out_dir.join("batch-gauss-exact-discovery-v1.json");
    let heldout_path = out_dir.join("batch-gauss-exact-heldout-v1.json");
    write_batch(
        &discovery_path,
        &batch(&discovery, "discovery", DISCOVERY_SEED),
    );
    write_batch(&heldout_path, &batch(&heldout, "heldout", HELDOUT_SEED));
    println!(
        "discovery={} {}\nheldout={} {}\ndisjoint=true; answers absent",
        discovery.len(),
        discovery_path.display(),
        heldout.len(),
        heldout_path.display()
    );
}
