//! Discovery-only ERF.PRECISE / ERFC.PRECISE / GAUSS route-and-store audit.
//!
//! Clean-room evidence only: published TOMS 654 / CDFLIB branch-190 arithmetic
//! and explicitly named reproducible discovery answers.  Neither GAUSS heldout
//! answer path nor the historical ERF heldout is named by this program.
//!
//! Usage:
//!   audit_erf_gauss_route_store <OxFunc-root>

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN, Ext80, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x,
    ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};
use std::collections::{BTreeMap, BTreeSet};

const CW: u16 = CW_PC64_RN;

const ERF_DISCOVERY: [&str; 7] = [
    "answers-b9train.json",
    "answers-erfp.json",
    "answers-erfm.json",
    "answers-b8erf.json",
    "answers-b7erf.json",
    "answers-b11.json",
    "answers-b10.json",
];

const ERFC_DISCOVERY: [&str; 5] = [
    "answers-erfcp.json",
    "answers-erfcm.json",
    "answers-b8erfc.json",
    "answers-b7erfc.json",
    "answers-b11c.json",
];

const GP: [f64; 7] = [
    0.577215664901533e+00,
    -0.409078193005776e+00,
    -0.230975380857675e+00,
    0.597275330452234e-01,
    0.766968181649490e-02,
    -0.514889771323592e-02,
    0.589597428611429e-03,
];

const GQ: [f64; 5] = [
    1.0,
    0.427569613095214e+00,
    0.158451672430138e+00,
    0.261132021441447e-01,
    0.423244297896961e-02,
];

fn ef(value: f64) -> Ext80 {
    ext_from_f64(value)
}

fn store(value: &Ext80) -> f64 {
    ext_to_f64(value, CW)
}

fn spill(value: Ext80, yes: bool) -> Ext80 {
    if yes { ef(store(&value)) } else { value }
}

fn ln_ext(value: &Ext80) -> Ext80 {
    ext_fyl2x(&ext_ln2(), value, CW)
}

fn exp_ext(value: &Ext80) -> Ext80 {
    let t = ext_mul(value, &ext_l2e(), CW);
    let k = ext_rndint(&t, CW);
    let f = ext_sub(&t, &k, CW);
    let negative = store(&f) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, CW), CW);
    let mut m = ext_add(&w, &ext_one(), CW);
    if negative {
        m = ext_div(&ext_one(), &m, CW);
    }
    ext_scale(&m, &k, CW)
}

fn ext_le(left: &Ext80, right: &Ext80) -> bool {
    store(&ext_sub(left, right, CW)) <= 0.0
}

fn load_map(root: &str, names: &[&str], expected_function: &str) -> BTreeMap<u64, u64> {
    let mut rows = BTreeMap::new();
    for name in names {
        let path = format!("{root}/smart-fuzzer/work/w109/G3-01-dist/{name}");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
        let bank: WitnessSet = serde_json::from_str(&text)
            .unwrap_or_else(|error| panic!("failed to parse {path}: {error}"));
        assert_eq!(bank.function, expected_function, "function drift in {path}");
        let mut ids = BTreeSet::new();
        for witness in bank.witnesses {
            let id = witness
                .id
                .as_deref()
                .filter(|id| !id.is_empty())
                .unwrap_or_else(|| panic!("missing witness id in {path}"));
            assert!(
                ids.insert(id.to_string()),
                "duplicate witness id {id} in {path}"
            );
            assert_eq!(witness.args.len(), 1, "non-scalar arity in {path}/{id}");
            let x = match &witness.args[0] {
                WitnessArg::Scalar(bits) => parse_bits_hex(bits).expect("scalar bits"),
                WitnessArg::Array(_) => continue,
            };
            let Some(expected) = parse_bits_hex(&witness.expected_bits) else {
                continue;
            };
            if let Some(previous) = rows.insert(x.to_bits(), expected.to_bits()) {
                assert_eq!(
                    previous,
                    expected.to_bits(),
                    "conflicting duplicate at x=0x{:016x}",
                    x.to_bits()
                );
            }
        }
    }
    rows
}

fn load_gauss(root: &str, answer: &str, expected_count: usize) -> BTreeMap<u64, u64> {
    let path = format!("{root}/smart-fuzzer/work/w109/G3-07-gauss/{answer}");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    let document: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("failed to parse {path}: {error}"));
    let environment = &document["capture_provenance"]["environment"];
    let cache = &document["capture_provenance"]["oracle_cache"];
    assert_eq!(environment["excel_version"].as_str(), Some("16.0"));
    assert_eq!(environment["excel_build"].as_str(), Some("20228"));
    assert_eq!(environment["excel_bitness"].as_str(), Some("64-bit"));
    assert_eq!(environment["workbook_compatibility"].as_str(), Some("2"));
    assert_eq!(
        environment["excel_input_plumbing"].as_str(),
        Some("cell_value2_bulk")
    );
    assert_eq!(cache["mode"].as_str(), Some("no_cache"));
    assert_eq!(cache["hits"].as_u64(), Some(0));
    assert_eq!(cache["misses"].as_u64(), Some(0));
    let bank: WitnessSet = serde_json::from_value(document)
        .unwrap_or_else(|error| panic!("failed to parse {path}: {error}"));
    assert_eq!(bank.function, "GAUSS", "function drift in {path}");
    assert_eq!(
        bank.witnesses.len(),
        expected_count,
        "count drift in {path}"
    );
    let mut rows = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for witness in bank.witnesses {
        let id = witness
            .id
            .as_deref()
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| panic!("missing GAUSS id in {path}"));
        assert!(
            ids.insert(id.to_string()),
            "duplicate GAUSS id {id} in {path}"
        );
        assert_eq!(
            witness.args.len(),
            1,
            "non-scalar GAUSS arity in {path}/{id}"
        );
        let x = match &witness.args[0] {
            WitnessArg::Scalar(bits) => parse_bits_hex(bits).expect("scalar bits"),
            WitnessArg::Array(_) => panic!("array argument in scalar GAUSS bank"),
        };
        let expected = parse_bits_hex(&witness.expected_bits).expect("numeric GAUSS answer");
        assert!(rows.insert(x.to_bits(), expected.to_bits()).is_none());
    }
    rows
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PairBin {
    BelowHalf,
    Half,
    HalfToOneThreeEighths,
    MidTail,
    SixOrMore,
}

impl PairBin {
    fn for_x(x: f64) -> Self {
        if x < 0.5 {
            Self::BelowHalf
        } else if x == 0.5 {
            Self::Half
        } else if x < 1.375 {
            Self::HalfToOneThreeEighths
        } else if x < 6.0 {
            Self::MidTail
        } else {
            Self::SixOrMore
        }
    }
}

#[derive(Default)]
struct PairScore {
    rows: usize,
    q_from_p_direct: usize,
    p_from_q_direct: usize,
    q_from_p_compensated: usize,
    p_from_q_compensated: usize,
}

fn score_pairs(root: &str) {
    let p = load_map(root, &ERF_DISCOVERY, "ERF.PRECISE");
    let q = load_map(root, &ERFC_DISCOVERY, "ERFC.PRECISE");
    let mut bins: BTreeMap<PairBin, PairScore> = BTreeMap::new();
    let mut overlaps = 0usize;
    for (&x_bits, &p_bits) in &p {
        let Some(&q_bits) = q.get(&x_bits) else {
            continue;
        };
        let x = f64::from_bits(x_bits);
        if !(x.is_finite() && x >= 0.0) {
            continue;
        }
        overlaps += 1;
        let p_value = f64::from_bits(p_bits);
        let q_value = f64::from_bits(q_bits);
        let score = bins.entry(PairBin::for_x(x)).or_default();
        score.rows += 1;
        score.q_from_p_direct += usize::from((1.0 - p_value).to_bits() == q_bits);
        score.p_from_q_direct += usize::from((1.0 - q_value).to_bits() == p_bits);
        score.q_from_p_compensated += usize::from((0.5 + (0.5 - p_value)).to_bits() == q_bits);
        score.p_from_q_compensated += usize::from((0.5 + (0.5 - q_value)).to_bits() == p_bits);
    }
    println!(
        "ERF/ERFC paired discovery: ERF={} ERFC={} overlaps={overlaps}",
        p.len(),
        q.len()
    );
    for (bin, score) in bins {
        println!(
            "  {bin:?}: rows={} q=1-stored-p {}/{} p=1-stored-q {}/{} q=comp-p {}/{} p=comp-q {}/{}",
            score.rows,
            score.q_from_p_direct,
            score.rows,
            score.p_from_q_direct,
            score.rows,
            score.q_from_p_compensated,
            score.rows,
            score.p_from_q_compensated,
            score.rows
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum XMode {
    Extended,
    Stored,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GamMode {
    Binary64,
    Extended,
    ExtendedReturn53,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InnerMode {
    ExtendedCompensated,
    ExtendedDirect,
    Binary64Compensated,
    Binary64Direct,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Assoc {
    WgThenInner,
    WThenGInner,
    WInnerThenG,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WMode {
    X87Continuous,
    InputZ,
}

#[derive(Clone, Copy, Debug)]
struct BodyCfg {
    x: XMode,
    series_53: bool,
    j_53: bool,
    gam: GamMode,
    g_53: bool,
    inner: InnerMode,
    assoc: Assoc,
    first_product_53: bool,
    w: WMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HalfSite {
    StoredReturn,
    ExtendedReturn,
    GFactor,
    WFactor,
    InnerFactor,
}

fn gam1_half(mode: GamMode) -> Ext80 {
    let per_op_53 = mode == GamMode::Binary64;
    let stage = |value: Ext80| spill(value, per_op_53);
    let t = ef(0.5);
    let mut top = ef(GP[6]);
    for &coefficient in GP[..6].iter().rev() {
        top = stage(ext_add(&ext_mul(&top, &t, CW), &ef(coefficient), CW));
    }
    let mut bottom = ef(GQ[4]);
    for &coefficient in GQ[..4].iter().rev() {
        bottom = stage(ext_add(&ext_mul(&bottom, &t, CW), &ef(coefficient), CW));
    }
    let ratio = stage(ext_div(&top, &bottom, CW));
    let h = stage(ext_mul(&ef(0.5), &ratio, CW));
    spill(h, mode == GamMode::ExtendedReturn53)
}

fn series_j(x: &Ext80, series_53: bool, j_53: bool) -> Ext80 {
    let a = ef(0.5);
    let stage = |value: Ext80| spill(value, series_53);
    let mut an = ef(3.0);
    let mut c = *x;
    let mut sum = stage(ext_div(x, &ext_add(&a, &ef(3.0), CW), CW));
    let tolerance = ext_div(
        &ext_mul(&ef(3.0), &ef(5e-15), CW),
        &ext_add(&a, &ext_one(), CW),
        CW,
    );
    for _ in 0..200 {
        an = stage(ext_add(&an, &ext_one(), CW));
        c = stage(ext_chs(&ext_mul(&c, &ext_div(x, &an, CW), CW), CW));
        let term = stage(ext_div(&c, &ext_add(&a, &an, CW), CW));
        sum = stage(ext_add(&sum, &term, CW));
        if ext_le(&ext_abs(&term, CW), &tolerance) {
            break;
        }
    }
    let inner_poly = ext_add(
        &ext_mul(
            &ext_sub(
                &ext_div(&sum, &ef(6.0), CW),
                &ext_div(&ef(0.5), &ext_add(&a, &ef(2.0), CW), CW),
                CW,
            ),
            x,
            CW,
        ),
        &ext_div(&ext_one(), &ext_add(&a, &ext_one(), CW), CW),
        CW,
    );
    spill(ext_mul(&ext_mul(&a, x, CW), &inner_poly, CW), j_53)
}

fn inner_value(j: &Ext80, mode: InnerMode) -> Ext80 {
    match mode {
        InnerMode::ExtendedCompensated => ext_add(&ef(0.5), &ext_sub(&ef(0.5), j, CW), CW),
        InnerMode::ExtendedDirect => ext_sub(&ext_one(), j, CW),
        InnerMode::Binary64Compensated => {
            let j = store(j);
            ef(0.5 + (0.5 - j))
        }
        InnerMode::Binary64Direct => ef(1.0 - store(j)),
    }
}

fn body_parts(z: f64, cfg: BodyCfg) -> Option<(Ext80, Ext80, Ext80)> {
    let mut x = ext_mul(&ef(z), &ef(z), CW);
    if cfg.x == XMode::Stored {
        x = ef(store(&x));
    }
    // The public branch consumes x=z^2 as an Ext80 temporary.  Testing the
    // binary64 publication of x here would incorrectly zero the enormous
    // interval where z^2 underflows binary64 but remains representable in
    // Ext80.  Only an actual Ext80 zero terminates the path.
    let x_mantissa = u64::from_le_bytes(x.0[..8].try_into().unwrap());
    if x_mantissa == 0 {
        return None;
    }
    let j = series_j(&x, cfg.series_53, cfg.j_53);
    let mut g = ext_add(&ext_one(), &gam1_half(cfg.gam), CW);
    if cfg.g_53 {
        g = ef(store(&g));
    }
    let w = match cfg.w {
        WMode::X87Continuous => {
            let argument = ext_mul(&ef(0.5), &ln_ext(&x), CW);
            exp_ext(&argument)
        }
        WMode::InputZ => ef(z),
    };
    Some((w, g, inner_value(&j, cfg.inner)))
}

fn combine(mut w: Ext80, mut g: Ext80, mut inner: Ext80, cfg: BodyCfg, site: HalfSite) -> f64 {
    match site {
        HalfSite::GFactor => g = ext_mul(&g, &ef(0.5), CW),
        HalfSite::WFactor => w = ext_mul(&w, &ef(0.5), CW),
        HalfSite::InnerFactor => inner = ext_mul(&inner, &ef(0.5), CW),
        HalfSite::StoredReturn | HalfSite::ExtendedReturn => {}
    }
    let stage = |value: Ext80| spill(value, cfg.first_product_53);
    let p = match cfg.assoc {
        Assoc::WgThenInner => ext_mul(&stage(ext_mul(&w, &g, CW)), &inner, CW),
        Assoc::WThenGInner => ext_mul(&w, &stage(ext_mul(&g, &inner, CW)), CW),
        Assoc::WInnerThenG => ext_mul(&stage(ext_mul(&w, &inner, CW)), &g, CW),
    };
    match site {
        HalfSite::StoredReturn => 0.5 * store(&p),
        HalfSite::ExtendedReturn => store(&ext_mul(&p, &ef(0.5), CW)),
        HalfSite::GFactor | HalfSite::WFactor | HalfSite::InnerFactor => store(&p),
    }
}

fn ordered(bits: u64) -> i128 {
    let signed = bits as i64;
    if signed < 0 {
        (!signed) as i128
    } else {
        signed as i128
    }
}

fn distance(left: u64, right: u64) -> u64 {
    ordered(left).abs_diff(ordered(right)) as u64
}

fn flush_subnormal(value: f64) -> f64 {
    if value.abs() < f64::MIN_POSITIVE {
        // The current Excel capture canonicalizes either sign to +0 here.
        0.0
    } else {
        value
    }
}

#[derive(Clone, Copy, Debug)]
struct TinyScore {
    exact: usize,
    max_ulp: u64,
    sum_ulp: u64,
    cfg: BodyCfg,
    site: HalfSite,
}

fn score_gauss_tiny(root: &str) {
    let exact = load_gauss(root, "answers-gauss-exact-discovery-v1.json", 8_192);
    let route = load_gauss(root, "answers-gauss-route-discovery-v1.json", 1_024);
    let mut all = exact;
    for (x, expected) in route {
        assert!(all.insert(x, expected).is_none(), "GAUSS discovery overlap");
    }
    let tiny: Vec<_> = all
        .iter()
        .filter_map(|(&x_bits, &expected)| {
            let x = f64::from_bits(x_bits);
            (x.abs() <= 1e-15).then_some((x, expected))
        })
        .collect();

    let magnitudes: BTreeSet<u64> = tiny
        .iter()
        .filter(|(x, _)| *x > 0.0)
        .map(|(x, _)| x.to_bits())
        .collect();
    let mut paired = 0usize;
    let mut odd = 0usize;
    let mut both_positive_zero = 0usize;
    for magnitude in magnitudes {
        let positive = all[&magnitude];
        let negative_key = magnitude | (1u64 << 63);
        let Some(&negative) = all.get(&negative_key) else {
            continue;
        };
        paired += 1;
        odd += usize::from(negative == (positive ^ (1u64 << 63)));
        both_positive_zero += usize::from(positive == 0 && negative == 0);
    }
    let nonzero_pairs = paired - both_positive_zero;
    println!(
        "GAUSS direct-tiny symmetry: rows={} signed_pairs={paired} nonzero-bit-exact-odd={odd}/{nonzero_pairs} canonical-+zero-flush-pairs={both_positive_zero}",
        tiny.len(),
    );

    let x_modes = [XMode::Extended, XMode::Stored];
    let gam_modes = [
        GamMode::Binary64,
        GamMode::Extended,
        GamMode::ExtendedReturn53,
    ];
    let inner_modes = [
        InnerMode::ExtendedCompensated,
        InnerMode::ExtendedDirect,
        InnerMode::Binary64Compensated,
        InnerMode::Binary64Direct,
    ];
    let associations = [Assoc::WgThenInner, Assoc::WThenGInner, Assoc::WInnerThenG];
    let w_modes = [WMode::X87Continuous, WMode::InputZ];
    let sites = [
        HalfSite::StoredReturn,
        HalfSite::ExtendedReturn,
        HalfSite::GFactor,
        HalfSite::WFactor,
        HalfSite::InnerFactor,
    ];
    let mut scores = Vec::new();
    for x in x_modes {
        for series_53 in [false, true] {
            for j_53 in [false, true] {
                for gam in gam_modes {
                    for g_53 in [false, true] {
                        for inner in inner_modes {
                            for assoc in associations {
                                for first_product_53 in [false, true] {
                                    for w in w_modes {
                                        let cfg = BodyCfg {
                                            x,
                                            series_53,
                                            j_53,
                                            gam,
                                            g_53,
                                            inner,
                                            assoc,
                                            first_product_53,
                                            w,
                                        };
                                        for site in sites {
                                            let (mut exact, mut max_ulp, mut sum_ulp) =
                                                (0usize, 0u64, 0u64);
                                            for &(input, expected) in &tiny {
                                                let z =
                                                    input.abs() * std::f64::consts::FRAC_1_SQRT_2;
                                                let mut got = body_parts(z, cfg)
                                                    .map(|(w, g, inner)| {
                                                        combine(w, g, inner, cfg, site)
                                                    })
                                                    .unwrap_or(0.0);
                                                if input.is_sign_negative() {
                                                    got = -got;
                                                }
                                                got = flush_subnormal(got);
                                                let delta = distance(got.to_bits(), expected);
                                                exact += usize::from(delta == 0);
                                                max_ulp = max_ulp.max(delta);
                                                sum_ulp = sum_ulp.saturating_add(delta);
                                            }
                                            scores.push(TinyScore {
                                                exact,
                                                max_ulp,
                                                sum_ulp,
                                                cfg,
                                                site,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    scores.sort_by_key(|score| (usize::MAX - score.exact, score.max_ulp, score.sum_ulp));
    let best_key = (scores[0].exact, scores[0].max_ulp, scores[0].sum_ulp);
    let best_ties = scores
        .iter()
        .filter(|score| (score.exact, score.max_ulp, score.sum_ulp) == best_key)
        .count();
    println!(
        "GAUSS direct-tiny public branch-190/store race: {} graphs x {} rows",
        scores.len(),
        tiny.len()
    );
    println!(
        "  best aggregate exact={}/{} max={} sum={} tied-graphs={best_ties}",
        best_key.0,
        tiny.len(),
        best_key.1,
        best_key.2,
    );
    for w in w_modes {
        let score = scores
            .iter()
            .filter(|score| score.cfg.w == w)
            .next()
            .expect("W-mode score");
        println!(
            "  best w={w:?}: exact={}/{} max={} sum={}",
            score.exact,
            tiny.len(),
            score.max_ulp,
            score.sum_ulp,
        );
    }
    for gam in gam_modes {
        let score = scores
            .iter()
            .filter(|score| score.cfg.gam == gam)
            .next()
            .expect("GAM1-mode score");
        println!(
            "  best gam={gam:?}: exact={}/{} max={} sum={}",
            score.exact,
            tiny.len(),
            score.max_ulp,
            score.sum_ulp,
        );
    }
    for site in sites {
        let score = scores
            .iter()
            .filter(|score| score.site == site)
            .next()
            .expect("half-site score");
        println!(
            "  best half={site:?}: exact={}/{} max={} sum={}",
            score.exact,
            tiny.len(),
            score.max_ulp,
            score.sum_ulp,
        );
    }
    for (rank, score) in scores.iter().take(24).enumerate() {
        println!(
            "  #{:02} exact={}/{} max={} sum={} half={:?} {:?}",
            rank + 1,
            score.exact,
            tiny.len(),
            score.max_ulp,
            score.sum_ulp,
            score.site,
            score.cfg
        );
    }

    let best = scores[0];
    let mut delta_hist: BTreeMap<i128, usize> = BTreeMap::new();
    let mut by_source = [(0usize, 0usize), (0usize, 0usize)];
    for &(input, expected) in &tiny {
        let z = input.abs() * std::f64::consts::FRAC_1_SQRT_2;
        let mut got = body_parts(z, best.cfg)
            .map(|(w, g, inner)| combine(w, g, inner, best.cfg, best.site))
            .unwrap_or(0.0);
        if input.is_sign_negative() {
            got = -got;
        }
        got = flush_subnormal(got);
        let signed_delta = ordered(expected) - ordered(got.to_bits());
        *delta_hist.entry(signed_delta).or_default() += 1;
        let source = usize::from(input.abs() >= 4.0 * f64::EPSILON);
        by_source[source].1 += 1;
        by_source[source].0 += usize::from(signed_delta == 0);
    }
    println!("  best signed expected-minus-model ULP histogram: {delta_hist:?}");
    println!(
        "  best by magnitude: abs(x)<4eps {}/{}; abs(x)>=4eps {}/{}",
        by_source[0].0, by_source[0].1, by_source[1].0, by_source[1].1
    );
}

fn main() {
    let root = std::env::args().nth(1).expect("OxFunc root");
    score_pairs(&root);
    score_gauss_tiny(&root);
}
