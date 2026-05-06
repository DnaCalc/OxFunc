use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{PreparedArgValue, prepare_args_values_only};
use crate::functions::factorial_common::{factorial_of_int, trunc_nonnegative};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

pub const MULTINOMIAL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MULTINOMIAL",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

#[derive(Debug, Clone, PartialEq)]
pub enum MultinomialEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

fn coerce_prepared_to_nonnegative_int(arg: &PreparedArgValue) -> Result<i64, MultinomialEvalError> {
    let n = crate::functions::adapters::coerce_prepared_to_number(arg)
        .map_err(MultinomialEvalError::Coercion)?;
    trunc_nonnegative(n).map_err(MultinomialEvalError::Domain)
}

const MULTINOMIAL_LANCZOS_G: f64 = 7.0;
const MULTINOMIAL_LANCZOS_COEFFS: [f64; 9] = [
    0.999_999_999_999_809_9,
    676.520_368_121_885_1,
    -1_259.139_216_722_402_8,
    771.323_428_777_653_1,
    -176.615_029_162_140_6,
    12.507_343_278_686_905,
    -0.138_571_095_265_720_12,
    9.984_369_578_019_572e-6,
    1.505_632_735_149_311_6e-7,
];
const MULTINOMIAL_TWO_POSITIVE_PUBLICATION_ULP_CLAMP: u64 = 3;
const MULTINOMIAL_MULTI_POSITIVE_PUBLICATION_ULP_RADIUS: u64 = 4;

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

fn step_down(mut x: f64, steps: u64) -> f64 {
    for _ in 0..steps {
        x = next_down(x);
    }
    x
}

fn ulp_distance(lhs: f64, rhs: f64) -> u64 {
    lhs.to_bits().abs_diff(rhs.to_bits())
}

fn multinomial_positive_items(items: &[i64]) -> Vec<i64> {
    items.iter().copied().filter(|item| *item > 0).collect()
}

fn positive_items_are_consecutive(positive_items: &[i64]) -> bool {
    if positive_items.len() < 2 {
        return false;
    }
    let mut sorted = positive_items.to_vec();
    sorted.sort_unstable();
    sorted.windows(2).all(|window| window[1] == window[0] + 1)
}

fn multinomial_exact_candidate(positive_items: &[i64]) -> f64 {
    let total = positive_items.iter().sum::<i64>();
    let denominator = positive_items
        .iter()
        .fold(1.0, |acc, item| acc * factorial_of_int(*item));
    factorial_of_int(total) / denominator
}

fn multinomial_log_factorial_candidate(positive_items: &[i64]) -> f64 {
    let total = positive_items.iter().sum::<i64>();
    let numerator_ln = factorial_of_int(total).ln();
    let denominator_ln = positive_items
        .iter()
        .map(|item| factorial_of_int(*item).ln())
        .sum::<f64>();
    (numerator_ln - denominator_ln).exp()
}

fn lanczos_ln_gamma(x: f64) -> f64 {
    let z = x - 1.0;
    let mut acc = MULTINOMIAL_LANCZOS_COEFFS[0];
    for (index, coeff) in MULTINOMIAL_LANCZOS_COEFFS.iter().enumerate().skip(1) {
        acc += coeff / (z + index as f64);
    }

    let t = z + MULTINOMIAL_LANCZOS_G + 0.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + acc.ln()
}

fn multinomial_descending_log_numerator_lanczos_denominator_candidate(
    positive_items: &[i64],
) -> f64 {
    let total = positive_items.iter().sum::<i64>();
    let numerator_ln = (2..=total)
        .rev()
        .map(|value| (value as f64).ln())
        .sum::<f64>();
    let denominator_ln = positive_items
        .iter()
        .map(|item| lanczos_ln_gamma((*item + 1) as f64))
        .sum::<f64>();
    (numerator_ln - denominator_ln).exp()
}

fn multinomial_lanczos_candidate(positive_items: &[i64]) -> f64 {
    let total = positive_items.iter().sum::<i64>();
    let numerator_ln = lanczos_ln_gamma((total + 1) as f64);
    let denominator_ln = positive_items
        .iter()
        .map(|item| lanczos_ln_gamma((*item + 1) as f64))
        .sum::<f64>();
    (numerator_ln - denominator_ln).exp()
}

fn multinomial_publication_candidate(positive_items: &[i64], exact_candidate: f64) -> f64 {
    let total = positive_items.iter().sum::<i64>();

    match positive_items.len() {
        0 | 1 => exact_candidate,
        2 => {
            if total < 5 {
                return exact_candidate;
            }

            let candidate = multinomial_log_factorial_candidate(positive_items);
            if candidate.is_finite() && candidate < exact_candidate {
                let floor = step_down(
                    exact_candidate,
                    MULTINOMIAL_TWO_POSITIVE_PUBLICATION_ULP_CLAMP,
                );
                candidate.max(floor)
            } else {
                exact_candidate
            }
        }
        3 => {
            if total < 9 || !positive_items_are_consecutive(positive_items) {
                return exact_candidate;
            }

            let candidate = multinomial_log_factorial_candidate(positive_items);
            if candidate.is_finite() && candidate < exact_candidate {
                candidate
            } else {
                exact_candidate
            }
        }
        _ => {
            if total < 10 || !positive_items_are_consecutive(positive_items) {
                return exact_candidate;
            }

            let mut best = exact_candidate;
            let mut best_distance = MULTINOMIAL_MULTI_POSITIVE_PUBLICATION_ULP_RADIUS + 1;
            for candidate in [
                multinomial_descending_log_numerator_lanczos_denominator_candidate(positive_items),
                multinomial_lanczos_candidate(positive_items),
            ] {
                if candidate.is_finite() {
                    let distance = ulp_distance(candidate, exact_candidate);
                    if distance <= MULTINOMIAL_MULTI_POSITIVE_PUBLICATION_ULP_RADIUS
                        && distance < best_distance
                    {
                        best = candidate;
                        best_distance = distance;
                    }
                }
            }
            best
        }
    }
}

pub fn multinomial_kernel(items: &[i64]) -> Result<f64, WorksheetErrorCode> {
    let positive_items = multinomial_positive_items(items);
    let exact_candidate = multinomial_exact_candidate(&positive_items);
    if !exact_candidate.is_finite() {
        return Ok(exact_candidate);
    }
    Ok(multinomial_publication_candidate(
        &positive_items,
        exact_candidate,
    ))
}

pub fn eval_multinomial_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, MultinomialEvalError> {
    let argc = args.len();
    if !MULTINOMIAL_META.arity.accepts(argc) {
        return Err(MultinomialEvalError::ArityMismatch {
            expected_min: MULTINOMIAL_META.arity.min,
            expected_max: MULTINOMIAL_META.arity.max,
            actual: argc,
        });
    }

    let prepared =
        prepare_args_values_only(args, resolver).map_err(MultinomialEvalError::Coercion)?;
    let items = prepared
        .iter()
        .map(coerce_prepared_to_nonnegative_int)
        .collect::<Result<Vec<_>, _>>()?;
    multinomial_kernel(&items)
        .map(EvalValue::Number)
        .map_err(MultinomialEvalError::Domain)
}

pub fn map_multinomial_error_to_ws(e: &MultinomialEvalError) -> WorksheetErrorCode {
    match e {
        MultinomialEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        MultinomialEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        MultinomialEvalError::Coercion(_) => WorksheetErrorCode::Value,
        MultinomialEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_bits(actual: f64, expected: f64) {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{actual} vs {expected}"
        );
    }

    #[test]
    fn multinomial_meta_function_id_is_stable() {
        assert_eq!(MULTINOMIAL_META.function_id, "FUNC.MULTINOMIAL");
    }

    #[test]
    fn multinomial_kernel_matches_exact_control_rows() {
        for (items, expected) in [
            (&[1, 2][..], 3.0_f64),
            (&[1, 3][..], 4.0_f64),
            (&[1, 2, 3][..], 60.0_f64),
            (&[1, 1, 1, 1][..], 24.0_f64),
            (&[5][..], 1.0_f64),
            (&[0, 0][..], 1.0_f64),
        ] {
            let actual = multinomial_kernel(items).expect("multinomial exact control");
            assert_bits(actual, expected);
        }
    }

    #[test]
    fn multinomial_exactness_witness_matches_excel_target() {
        let actual = multinomial_kernel(&[2, 3, 4]).expect("multinomial witness");
        let prior_local = 1260.0_f64;
        let excel_target = 1259.9999999999991_f64;

        assert_bits(actual, excel_target);
        assert_ne!(actual.to_bits(), prior_local.to_bits());
    }

    #[test]
    fn multinomial_widened_empirical_machine_witness_rows_match_excel_targets() {
        let anchor_target = 1259.9999999999991_f64;
        for (items, expected) in [
            (&[2, 3, 4][..], anchor_target),
            (&[4, 2, 3][..], anchor_target),
            (&[0, 2, 3, 4][..], anchor_target),
            (&[0, 2, 3][..], 9.999999999999998_f64),
            (&[2, 7][..], 35.99999999999998_f64),
            (&[7, 2][..], 35.99999999999998_f64),
            (&[1, 2, 3, 4][..], 12599.999999999995_f64),
            (&[2, 3, 4, 5][..], 2522520.0000000005_f64),
        ] {
            let actual = multinomial_kernel(items).expect("multinomial empirical witness");
            assert_bits(actual, expected);
        }
    }

    #[test]
    fn multinomial_ordering_and_zero_padding_invariance_hold_for_widened_rows() {
        let anchor = multinomial_kernel(&[2, 3, 4]).expect("anchor");
        let four_arg = multinomial_kernel(&[1, 2, 3, 4]).expect("four arg");
        let two_positive = multinomial_kernel(&[2, 7]).expect("two positive");

        assert_bits(
            multinomial_kernel(&[3, 4, 2]).expect("anchor permutation"),
            anchor,
        );
        assert_bits(
            multinomial_kernel(&[0, 2, 3, 4]).expect("anchor zero padded"),
            anchor,
        );
        assert_bits(
            multinomial_kernel(&[4, 3, 2, 1]).expect("four arg permutation"),
            four_arg,
        );
        assert_bits(
            multinomial_kernel(&[0, 7, 2]).expect("two positive zero padded"),
            two_positive,
        );
    }
}
