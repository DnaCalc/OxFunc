use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN, Ext80, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x,
    ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sub, ext_to_f64,
};
use std::collections::{BTreeMap, BTreeSet};

const CW: u16 = CW_PC64_RN;

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

fn load_gauss(root: &str, answer: &str, batch: &str, expected_count: usize) -> BTreeMap<u64, u64> {
    let path = format!("{root}/smart-fuzzer/work/w109/G3-07-gauss/{answer}");
    let batch_path = format!("{root}/smart-fuzzer/work/w109/G3-07-gauss/{batch}");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    let document: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("failed to parse {path}: {error}"));
    let batch_text = std::fs::read_to_string(&batch_path)
        .unwrap_or_else(|error| panic!("failed to read {batch_path}: {error}"));
    let batch_document: serde_json::Value = serde_json::from_str(&batch_text)
        .unwrap_or_else(|error| panic!("failed to parse {batch_path}: {error}"));
    assert_eq!(batch_document["function"].as_str(), Some("GAUSS"));
    assert_eq!(batch_document["row_id"].as_str(), Some("G3-07-gauss"));
    let answer_rows = document["witnesses"]
        .as_array()
        .unwrap_or_else(|| panic!("missing witnesses in {path}"));
    let batch_rows = batch_document["probes"]
        .as_array()
        .unwrap_or_else(|| panic!("missing probes in {batch_path}"));
    assert_eq!(
        answer_rows.len(),
        expected_count,
        "answer count drift in {path}"
    );
    assert_eq!(
        batch_rows.len(),
        expected_count,
        "batch count drift in {batch_path}"
    );
    for (answer_row, batch_row) in answer_rows.iter().zip(batch_rows) {
        let probe = &batch_row["probe"];
        assert_eq!(answer_row["id"], probe["id"], "batch/answer ID drift");
        assert_eq!(
            answer_row["args"], probe["args"],
            "batch/answer argument drift"
        );
    }
    assert_eq!(
        document["capture_provenance"]["schema_version"].as_str(),
        Some("w109-capture-provenance-v1")
    );
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
        let input = match &witness.args[0] {
            WitnessArg::Scalar(bits) => parse_bits_hex(bits).expect("scalar bits"),
            WitnessArg::Array(_) => panic!("array argument in scalar GAUSS bank"),
        };
        let expected = parse_bits_hex(&witness.expected_bits).expect("numeric GAUSS answer");
        assert!(
            rows.insert(input.to_bits(), expected.to_bits()).is_none(),
            "duplicate GAUSS input in {path}"
        );
    }
    rows
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
    let product = match cfg.assoc {
        Assoc::WgThenInner => ext_mul(&stage(ext_mul(&w, &g, CW)), &inner, CW),
        Assoc::WThenGInner => ext_mul(&w, &stage(ext_mul(&g, &inner, CW)), CW),
        Assoc::WInnerThenG => ext_mul(&stage(ext_mul(&w, &inner, CW)), &g, CW),
    };
    match site {
        HalfSite::StoredReturn => 0.5 * store(&product),
        HalfSite::ExtendedReturn => store(&ext_mul(&product, &ef(0.5), CW)),
        HalfSite::GFactor | HalfSite::WFactor | HalfSite::InnerFactor => store(&product),
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
        0.0
    } else {
        value
    }
}
