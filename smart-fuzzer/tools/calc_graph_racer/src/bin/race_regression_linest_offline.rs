//! Offline-only G3-03 LINEST/TREND regression schedule audit.
//!
//! This research binary consumes only the banked July 2026 oracle answers. It
//! deliberately performs no Excel/COM work.  It replays the current production
//! TREND and FORECAST surfaces and races clean-room single-predictor least-
//! squares graphs against the twelve banked TREND cells.

#[path = "regression_research/common.rs"]
mod common;

use common::{Arithmetic, InterceptForm, Order, regression_variants, sum};
use oxfunc_core::functions::regression_forecast_family::{
    eval_forecast_surface, eval_linest_surface, eval_trend_surface,
    map_regression_forecast_error_to_ws,
};
use oxfunc_core::resolver::ReferenceSystemProvider;
use oxfunc_core::value::{CalcArray, CalcValue, CoreValue};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

const TREND_ANSWERS: &str = "../../work/w109/G3-03-answers-trend.json";
const FORECAST_ANSWERS: &str = "../../work/w109/G3-03-answers-forecast.json";
const FORECAST_ADV_ANSWERS: &str = "../../work/w109/G3-03-answers-fc-adv.json";
const TREND_BATCH: &str = "../../work/w109/G3-03-batch-trend.json";
const FORECAST_BATCH: &str = "../../work/w109/G3-03-batch-forecast.json";
const FORECAST_ADV_BATCH: &str = "../../work/w109/G3-03-batch-fc-adv.json";

#[derive(Debug, Deserialize)]
struct AnswerBank {
    function: String,
    witnesses: Vec<Witness>,
}

#[derive(Debug, Deserialize)]
struct ProbeBatch {
    function: String,
    probes: Vec<ProbeEnvelope>,
}

#[derive(Debug, Deserialize)]
struct ProbeEnvelope {
    probe: Probe,
}

#[derive(Debug, Deserialize)]
struct Probe {
    id: String,
    args: Vec<Value>,
}

#[derive(Clone, Debug, Deserialize)]
struct Witness {
    id: String,
    args: Vec<Value>,
    expected_bits: String,
}

#[derive(Clone, Debug)]
struct TrendCase {
    id: String,
    y: Vec<f64>,
    x: Vec<f64>,
    new_x: f64,
    expected: f64,
}

#[derive(Clone, Debug)]
struct ForecastCase {
    id: String,
    new_x: f64,
    y: Vec<f64>,
    x: Vec<f64>,
    expected: Expected,
}

#[derive(Clone, Debug)]
enum Expected {
    Number(f64),
    Error(String),
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
struct Score {
    exact: usize,
    sum_ulp: u128,
    max_ulp: u64,
}

#[derive(Clone, Debug)]
struct CandidateResult {
    score: Score,
    id: String,
    output_bits: Vec<u64>,
}

#[derive(Clone, Copy, Debug)]
enum Publication {
    InterceptF64,
    InterceptX87Stored,
    PointSlopeF64(Order),
    PointSlopeX87Stored(Order),
}

impl Publication {
    const ALL: [Self; 8] = [
        Self::InterceptF64,
        Self::InterceptX87Stored,
        Self::PointSlopeF64(Order::Forward),
        Self::PointSlopeF64(Order::Reverse),
        Self::PointSlopeF64(Order::Pairwise),
        Self::PointSlopeX87Stored(Order::Forward),
        Self::PointSlopeX87Stored(Order::Reverse),
        Self::PointSlopeX87Stored(Order::Pairwise),
    ];

    fn tag(self) -> &'static str {
        match self {
            Self::InterceptF64 => "a+b*x-f64",
            Self::InterceptX87Stored => "a+b*x-x87dr",
            Self::PointSlopeF64(Order::Forward) => "my+b*(x-mx)-f64-mean-fwd",
            Self::PointSlopeF64(Order::Reverse) => "my+b*(x-mx)-f64-mean-rev",
            Self::PointSlopeF64(Order::Pairwise) => "my+b*(x-mx)-f64-mean-pair",
            Self::PointSlopeX87Stored(Order::Forward) => "my+b*(x-mx)-x87dr-mean-fwd",
            Self::PointSlopeX87Stored(Order::Reverse) => "my+b*(x-mx)-x87dr-mean-rev",
            Self::PointSlopeX87Stored(Order::Pairwise) => "my+b*(x-mx)-x87dr-mean-pair",
        }
    }
}

struct EmptyResolver;
impl ReferenceSystemProvider for EmptyResolver {}

fn parse_hex(text: &str) -> f64 {
    let digits = text.strip_prefix("0x").expect("0x-prefixed binary64");
    f64::from_bits(u64::from_str_radix(digits, 16).expect("binary64 hex"))
}

fn parse_expected(text: &str) -> f64 {
    parse_hex(text)
}

fn parse_expected_value(text: &str) -> Expected {
    if let Some(error) = text.strip_prefix("error:") {
        Expected::Error(error.to_string())
    } else {
        Expected::Number(parse_expected(text))
    }
}

fn parse_hex_scalar(value: &Value) -> f64 {
    parse_hex(value.as_str().expect("hex scalar"))
}

fn parse_hex_vector(value: &Value) -> Vec<f64> {
    value
        .as_array()
        .expect("hex vector")
        .iter()
        .map(parse_hex_scalar)
        .collect()
}

fn load_bank(path: &str) -> AnswerBank {
    serde_json::from_str(&fs::read_to_string(path).expect("read answer bank"))
        .expect("parse answer bank")
}

fn load_verified_bank(
    answer_path: &str,
    batch_path: &str,
    expected_function: &str,
    expected_count: usize,
) -> AnswerBank {
    let answers = load_bank(answer_path);
    let batch: ProbeBatch =
        serde_json::from_str(&fs::read_to_string(batch_path).expect("read probe batch"))
            .expect("parse probe batch");
    assert_eq!(
        answers.function, expected_function,
        "answer function drift in {answer_path}"
    );
    assert_eq!(
        batch.function, expected_function,
        "batch function drift in {batch_path}"
    );
    assert_eq!(
        answers.witnesses.len(),
        batch.probes.len(),
        "batch/answer count drift for {expected_function}"
    );
    assert_eq!(
        answers.witnesses.len(),
        expected_count,
        "frozen row-count drift for {expected_function} bank {answer_path}"
    );

    let mut answer_ids = BTreeSet::new();
    let mut batch_ids = BTreeSet::new();
    for (index, (witness, envelope)) in answers
        .witnesses
        .iter()
        .zip(batch.probes.iter())
        .enumerate()
    {
        assert!(
            answer_ids.insert(witness.id.as_str()),
            "duplicate answer ID {} in {answer_path}",
            witness.id
        );
        assert!(
            batch_ids.insert(envelope.probe.id.as_str()),
            "duplicate batch ID {} in {batch_path}",
            envelope.probe.id
        );
        assert_eq!(
            witness.id, envelope.probe.id,
            "ordered ID drift at row {index} for {expected_function}"
        );
        assert_eq!(
            witness.args, envelope.probe.args,
            "ordered argument drift at row {index} for {expected_function}"
        );
    }
    answers
}

fn load_trend() -> Vec<TrendCase> {
    let bank = load_verified_bank(TREND_ANSWERS, TREND_BATCH, "TREND", 12);
    bank.witnesses
        .into_iter()
        .map(|w| {
            assert_eq!(w.args.len(), 3, "{} arg count", w.id);
            TrendCase {
                id: w.id,
                y: parse_hex_vector(&w.args[0]),
                x: parse_hex_vector(&w.args[1]),
                new_x: parse_hex_scalar(&w.args[2]),
                expected: parse_expected(&w.expected_bits),
            }
        })
        .collect()
}

fn load_forecast(answer_path: &str, batch_path: &str) -> Vec<ForecastCase> {
    let bank = load_verified_bank(answer_path, batch_path, "FORECAST", 35);
    bank.witnesses
        .into_iter()
        .map(|w| {
            assert_eq!(w.args.len(), 3, "{} arg count", w.id);
            ForecastCase {
                id: w.id,
                new_x: parse_hex_scalar(&w.args[0]),
                y: parse_hex_vector(&w.args[1]),
                x: parse_hex_vector(&w.args[2]),
                expected: parse_expected_value(&w.expected_bits),
            }
        })
        .collect()
}

fn row(values: &[f64]) -> CalcValue {
    CalcValue::array(
        CalcArray::from_rows(vec![
            values.iter().copied().map(CalcValue::number).collect(),
        ])
        .expect("non-empty row"),
    )
}

fn scalar(value: CalcValue) -> f64 {
    match value.core() {
        CoreValue::Number(number) => *number,
        other => panic!("expected scalar, got {other:?}"),
    }
}

fn numeric_row(value: CalcValue) -> Vec<f64> {
    match value.core() {
        CoreValue::Array(array) => array
            .iter_row_major()
            .map(|cell| match cell.core() {
                CoreValue::Number(number) => *number,
                other => panic!("expected numeric coefficient, got {other:?}"),
            })
            .collect(),
        other => panic!("expected coefficient row, got {other:?}"),
    }
}

fn production_trend(case: &TrendCase) -> Result<f64, String> {
    eval_trend_surface(
        &[row(&case.y), row(&case.x), CalcValue::number(case.new_x)],
        &EmptyResolver,
    )
    .map(scalar)
    .map_err(|error| format!("{:?}", map_regression_forecast_error_to_ws(&error)))
}

fn production_linest(case: &TrendCase) -> Result<(f64, f64), String> {
    let coefficients = eval_linest_surface(&[row(&case.y), row(&case.x)], &EmptyResolver)
        .map(numeric_row)
        .map_err(|error| format!("{:?}", map_regression_forecast_error_to_ws(&error)))?;
    assert_eq!(coefficients.len(), 2);
    Ok((coefficients[0], coefficients[1]))
}

fn production_forecast(case: &ForecastCase) -> Result<f64, String> {
    eval_forecast_surface(
        &[CalcValue::number(case.new_x), row(&case.y), row(&case.x)],
        &EmptyResolver,
    )
    .map(scalar)
    .map_err(|error| format!("{:?}", map_regression_forecast_error_to_ws(&error)))
}

fn ordered_key(value: f64) -> u64 {
    let bits = value.to_bits();
    if bits >> 63 == 0 {
        bits | (1_u64 << 63)
    } else {
        !bits
    }
}

fn ulp(a: f64, b: f64) -> u64 {
    if a.is_nan() || b.is_nan() {
        u64::MAX
    } else {
        ordered_key(a).abs_diff(ordered_key(b))
    }
}

fn update_score(score: &mut Score, got: f64, expected: f64) {
    let distance = ulp(got, expected);
    score.exact += usize::from(distance == 0);
    score.sum_ulp = score.sum_ulp.saturating_add(distance as u128);
    score.max_ulp = score.max_ulp.max(distance);
}

fn mean(values: &[f64], order: Order, arithmetic: Arithmetic) -> f64 {
    arithmetic.div(sum(values, order, arithmetic), values.len() as f64)
}

fn publish(
    case: &TrendCase,
    slope: f64,
    intercept: f64,
    arithmetic: Arithmetic,
    publication: Publication,
) -> f64 {
    match publication {
        Publication::InterceptF64 => intercept + slope * case.new_x,
        Publication::InterceptX87Stored => {
            Arithmetic::X87Stored.add(intercept, Arithmetic::X87Stored.mul(slope, case.new_x))
        }
        Publication::PointSlopeF64(mean_order) => {
            let mx = mean(&case.x, mean_order, arithmetic);
            let my = mean(&case.y, mean_order, arithmetic);
            my + slope * (case.new_x - mx)
        }
        Publication::PointSlopeX87Stored(mean_order) => {
            let a = Arithmetic::X87Stored;
            let mx = mean(&case.x, mean_order, arithmetic);
            let my = mean(&case.y, mean_order, arithmetic);
            a.add(my, a.mul(slope, a.sub(case.new_x, mx)))
        }
    }
}

fn centered_qr(
    x: &[f64],
    y: &[f64],
    arithmetic: Arithmetic,
    mean_order: Order,
    moment_order: Order,
    intercept_form: InterceptForm,
) -> (f64, f64) {
    let mx = mean(x, mean_order, arithmetic);
    let my = mean(y, mean_order, arithmetic);
    let dx = x
        .iter()
        .copied()
        .map(|value| arithmetic.sub(value, mx))
        .collect::<Vec<_>>();
    let dy = y
        .iter()
        .copied()
        .map(|value| arithmetic.sub(value, my))
        .collect::<Vec<_>>();
    let squares = dx
        .iter()
        .copied()
        .map(|value| arithmetic.mul(value, value))
        .collect::<Vec<_>>();
    let norm = sum(&squares, moment_order, arithmetic).sqrt();
    let q = dx
        .iter()
        .copied()
        .map(|value| arithmetic.div(value, norm))
        .collect::<Vec<_>>();
    let qy = q
        .iter()
        .copied()
        .zip(dy.iter().copied())
        .map(|(qv, dyv)| arithmetic.mul(qv, dyv))
        .collect::<Vec<_>>();
    let slope = arithmetic.div(sum(&qy, moment_order, arithmetic), norm);
    let intercept = match intercept_form {
        InterceptForm::MeanMinusSlopeMean => arithmetic.sub(my, arithmetic.mul(slope, mx)),
        InterceptForm::SumResidualOverN => {
            let sx = sum(x, mean_order, arithmetic);
            let sy = sum(y, mean_order, arithmetic);
            arithmetic.div(
                arithmetic.sub(sy, arithmetic.mul(slope, sx)),
                x.len() as f64,
            )
        }
        InterceptForm::MeanResidual => {
            let residuals = x
                .iter()
                .copied()
                .zip(y.iter().copied())
                .map(|(xv, yv)| arithmetic.sub(yv, arithmetic.mul(slope, xv)))
                .collect::<Vec<_>>();
            arithmetic.div(
                sum(&residuals, mean_order, arithmetic),
                residuals.len() as f64,
            )
        }
        InterceptForm::RawDeterminant | InterceptForm::SolverCoefficient => {
            unreachable!("special centered graph received a non-post-slope intercept")
        }
    };
    (slope, intercept)
}

fn scaled_centered(
    x: &[f64],
    y: &[f64],
    arithmetic: Arithmetic,
    mean_order: Order,
    moment_order: Order,
    intercept_form: InterceptForm,
    qr: bool,
) -> (f64, f64) {
    let mx = mean(x, mean_order, arithmetic);
    let my = mean(y, mean_order, arithmetic);
    let dx = x
        .iter()
        .copied()
        .map(|value| arithmetic.sub(value, mx))
        .collect::<Vec<_>>();
    let dy = y
        .iter()
        .copied()
        .map(|value| arithmetic.sub(value, my))
        .collect::<Vec<_>>();
    let scale = dx.iter().copied().map(f64::abs).fold(0.0, f64::max);
    let z = dx
        .iter()
        .copied()
        .map(|value| arithmetic.div(value, scale))
        .collect::<Vec<_>>();
    let z_squares = z
        .iter()
        .copied()
        .map(|value| arithmetic.mul(value, value))
        .collect::<Vec<_>>();
    let zdy = z
        .iter()
        .copied()
        .zip(dy.iter().copied())
        .map(|(zv, dyv)| arithmetic.mul(zv, dyv))
        .collect::<Vec<_>>();
    let scaled_slope = if qr {
        let norm = sum(&z_squares, moment_order, arithmetic).sqrt();
        let q = z
            .iter()
            .copied()
            .map(|value| arithmetic.div(value, norm))
            .collect::<Vec<_>>();
        let qy = q
            .iter()
            .copied()
            .zip(dy.iter().copied())
            .map(|(qv, dyv)| arithmetic.mul(qv, dyv))
            .collect::<Vec<_>>();
        arithmetic.div(sum(&qy, moment_order, arithmetic), norm)
    } else {
        arithmetic.div(
            sum(&zdy, moment_order, arithmetic),
            sum(&z_squares, moment_order, arithmetic),
        )
    };
    let slope = arithmetic.div(scaled_slope, scale);
    let intercept = match intercept_form {
        InterceptForm::MeanMinusSlopeMean => arithmetic.sub(my, arithmetic.mul(slope, mx)),
        InterceptForm::SumResidualOverN => {
            let sx = sum(x, mean_order, arithmetic);
            let sy = sum(y, mean_order, arithmetic);
            arithmetic.div(
                arithmetic.sub(sy, arithmetic.mul(slope, sx)),
                x.len() as f64,
            )
        }
        InterceptForm::MeanResidual => {
            let residuals = x
                .iter()
                .copied()
                .zip(y.iter().copied())
                .map(|(xv, yv)| arithmetic.sub(yv, arithmetic.mul(slope, xv)))
                .collect::<Vec<_>>();
            arithmetic.div(
                sum(&residuals, mean_order, arithmetic),
                residuals.len() as f64,
            )
        }
        InterceptForm::RawDeterminant | InterceptForm::SolverCoefficient => {
            unreachable!("special scaled graph received a non-post-slope intercept")
        }
    };
    (slope, intercept)
}

fn score_common(cases: &[TrendCase]) -> Vec<CandidateResult> {
    let mut results = Vec::new();
    for variant in regression_variants() {
        for publication in Publication::ALL {
            let mut score = Score::default();
            let mut output_bits = Vec::with_capacity(cases.len());
            for case in cases {
                let coefficients = common::regress(&case.x, &case.y, variant);
                let got = publish(
                    case,
                    coefficients.slope,
                    coefficients.intercept,
                    variant.arith,
                    publication,
                );
                update_score(&mut score, got, case.expected);
                output_bits.push(got.to_bits());
            }
            results.push(CandidateResult {
                score,
                id: format!("common:{}:{}", variant.id(), publication.tag()),
                output_bits,
            });
        }
    }
    results
}

fn score_special(cases: &[TrendCase]) -> Vec<CandidateResult> {
    let mut results = Vec::new();
    for arithmetic in [Arithmetic::F64, Arithmetic::X87Stored] {
        for mean_order in [Order::Forward, Order::Reverse, Order::Pairwise] {
            for moment_order in [Order::Forward, Order::Reverse, Order::Pairwise] {
                for intercept in [
                    InterceptForm::MeanMinusSlopeMean,
                    InterceptForm::SumResidualOverN,
                    InterceptForm::MeanResidual,
                ] {
                    for publication in Publication::ALL {
                        for (family, scaled, qr) in [
                            ("centered-qr", false, true),
                            ("scaled-centered", true, false),
                            ("scaled-centered-qr", true, true),
                        ] {
                            let mut score = Score::default();
                            let mut output_bits = Vec::with_capacity(cases.len());
                            for case in cases {
                                let (slope, intercept_value) = if scaled {
                                    scaled_centered(
                                        &case.x,
                                        &case.y,
                                        arithmetic,
                                        mean_order,
                                        moment_order,
                                        intercept,
                                        qr,
                                    )
                                } else {
                                    centered_qr(
                                        &case.x,
                                        &case.y,
                                        arithmetic,
                                        mean_order,
                                        moment_order,
                                        intercept,
                                    )
                                };
                                let got =
                                    publish(case, slope, intercept_value, arithmetic, publication);
                                update_score(&mut score, got, case.expected);
                                output_bits.push(got.to_bits());
                            }
                            results.push(CandidateResult {
                                score,
                                id: format!(
                                    "{family}:{}:mean-{}:mom-{}:{}:{}",
                                    arithmetic.tag(),
                                    mean_order.tag(),
                                    moment_order.tag(),
                                    intercept.tag(),
                                    publication.tag()
                                ),
                                output_bits,
                            });
                        }
                    }
                }
            }
        }
    }
    results
}

fn sort_results(results: &mut [CandidateResult]) {
    results.sort_by(|left, right| {
        right
            .score
            .exact
            .cmp(&left.score.exact)
            .then_with(|| left.score.max_ulp.cmp(&right.score.max_ulp))
            .then_with(|| left.score.sum_ulp.cmp(&right.score.sum_ulp))
            .then_with(|| left.id.cmp(&right.id))
    });
}

fn main() {
    let trend = load_trend();
    assert_eq!(trend.len(), 12);
    let mut forecast = load_forecast(FORECAST_ANSWERS, FORECAST_BATCH);
    forecast.extend(load_forecast(FORECAST_ADV_ANSWERS, FORECAST_ADV_BATCH));
    assert_eq!(forecast.len(), 70); // 65 numeric cells plus five #DIV/0! controls.

    let mut prod_trend = Score::default();
    let mut prod_trend_errors = 0usize;
    for case in &trend {
        match production_trend(case) {
            Ok(got) => update_score(&mut prod_trend, got, case.expected),
            Err(_) => prod_trend_errors += 1,
        }
    }
    let mut prod_forecast = Score::default();
    let mut prod_forecast_exact = 0usize;
    let mut prod_forecast_error_exact = 0usize;
    let mut prod_forecast_misses = Vec::new();
    for case in &forecast {
        match (&case.expected, production_forecast(case)) {
            (Expected::Number(expected), Ok(got)) => {
                update_score(&mut prod_forecast, got, *expected);
                if got.to_bits() == expected.to_bits() {
                    prod_forecast_exact += 1;
                } else {
                    prod_forecast_misses.push(format!(
                        "{} got=0x{:016x} want=0x{:016x}",
                        case.id,
                        got.to_bits(),
                        expected.to_bits()
                    ));
                }
            }
            (Expected::Error(expected), Err(got)) if got == *expected => {
                prod_forecast_exact += 1;
                prod_forecast_error_exact += 1;
            }
            (expected, got) => {
                prod_forecast_misses.push(format!("{} got={got:?} want={expected:?}", case.id));
            }
        }
    }
    println!(
        "production TREND exact={}/{} errors={} max_ulp={} sum_ulp={}",
        prod_trend.exact,
        trend.len(),
        prod_trend_errors,
        prod_trend.max_ulp,
        prod_trend.sum_ulp
    );
    println!(
        "production FORECAST exact={}/{} max_ulp={} sum_ulp={}",
        prod_forecast_exact,
        forecast.len(),
        prod_forecast.max_ulp,
        prod_forecast.sum_ulp
    );
    println!(
        "production FORECAST numeric_exact={} error_exact={} misses={}",
        prod_forecast.exact,
        prod_forecast_error_exact,
        prod_forecast_misses.len()
    );
    for miss in &prod_forecast_misses {
        println!("FORECAST miss {miss}");
    }

    let mut by_dataset: BTreeMap<Vec<u64>, Vec<&TrendCase>> = BTreeMap::new();
    for case in &trend {
        let mut key = case
            .x
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>();
        key.extend(case.y.iter().map(|value| value.to_bits()));
        by_dataset.entry(key).or_default().push(case);
    }
    println!("trend datasets={}", by_dataset.len());
    for cases in by_dataset.values() {
        let representative = cases[0];
        let coefficients = production_linest(representative);
        let exact = cases
            .iter()
            .filter(|case| {
                production_trend(case).is_ok_and(|got| got.to_bits() == case.expected.to_bits())
            })
            .count();
        match coefficients {
            Ok((slope, intercept)) => println!(
                "dataset={} n={} production_LINEST slope=0x{:016x} intercept=0x{:016x} TREND_exact={}/{}",
                representative.id,
                representative.x.len(),
                slope.to_bits(),
                intercept.to_bits(),
                exact,
                cases.len()
            ),
            Err(error) => println!(
                "dataset={} n={} production_LINEST error={} TREND_exact={}/{}",
                representative.id,
                representative.x.len(),
                error,
                exact,
                cases.len()
            ),
        }
    }

    let mut results = score_common(&trend);
    let common_graphs = results.len();
    let special = score_special(&trend);
    let special_graphs = special.len();
    results.extend(special);
    assert_eq!(common_graphs, 1_968, "common graph-space drift");
    assert_eq!(special_graphs, 1_296, "special graph-space drift");
    let unique_ids = results
        .iter()
        .map(|result| result.id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        unique_ids.len(),
        results.len(),
        "duplicate candidate graph IDs"
    );
    let distinct_output_vectors = results
        .iter()
        .map(|result| result.output_bits.as_slice())
        .collect::<BTreeSet<_>>()
        .len();
    sort_results(&mut results);
    let leader = &results[0];
    let leader_score_ties = results
        .iter()
        .filter(|result| result.score == leader.score)
        .count();
    let leader_bit_identical_ties = results
        .iter()
        .filter(|result| result.output_bits == leader.output_bits)
        .count();
    println!(
        "candidate_graphs={} common_graphs={common_graphs} special_graphs={special_graphs} distinct_output_vectors={distinct_output_vectors}",
        results.len()
    );
    println!(
        "leader aggregate_score_ties={leader_score_ties} bit_identical_output_ties={leader_bit_identical_ties}"
    );
    for (rank, result) in results.iter().take(50).enumerate() {
        println!(
            "{:02} exact={}/{} max_ulp={} sum_ulp={} {}",
            rank + 1,
            result.score.exact,
            trend.len(),
            result.score.max_ulp,
            result.score.sum_ulp,
            result.id
        );
    }

    println!("production_misses:");
    for case in &trend {
        match production_trend(case) {
            Ok(got) if got.to_bits() != case.expected.to_bits() => println!(
                "{} got=0x{:016x} want=0x{:016x} ulp={}",
                case.id,
                got.to_bits(),
                case.expected.to_bits(),
                ulp(got, case.expected)
            ),
            Err(error) => println!(
                "{} got=error:{} want=0x{:016x}",
                case.id,
                error,
                case.expected.to_bits()
            ),
            _ => {}
        }
    }
}
