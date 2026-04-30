use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

const DISCRETE_DIST_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DISCRETE_DIST_BASE",
    arity: Arity::exact(1),
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

pub const BINOM_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BINOM.DIST",
    arity: Arity::exact(4),
    ..DISCRETE_DIST_BASE_META
};

pub const BINOM_DIST_RANGE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BINOM.DIST.RANGE",
    arity: Arity { min: 3, max: 4 },
    ..DISCRETE_DIST_BASE_META
};

pub const BINOM_INV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BINOM.INV",
    arity: Arity::exact(3),
    ..DISCRETE_DIST_BASE_META
};

pub const BINOMDIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BINOMDIST",
    arity: Arity::exact(4),
    ..DISCRETE_DIST_BASE_META
};

pub const CRITBINOM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CRITBINOM",
    arity: Arity::exact(3),
    ..DISCRETE_DIST_BASE_META
};

pub const POISSON_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.POISSON",
    arity: Arity::exact(3),
    ..DISCRETE_DIST_BASE_META
};

pub const POISSON_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.POISSON.DIST",
    arity: Arity::exact(3),
    ..DISCRETE_DIST_BASE_META
};

pub const HYPGEOM_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.HYPGEOM.DIST",
    arity: Arity::exact(5),
    ..DISCRETE_DIST_BASE_META
};

pub const HYPGEOMDIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.HYPGEOMDIST",
    arity: Arity::exact(4),
    ..DISCRETE_DIST_BASE_META
};

pub const NEGBINOM_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NEGBINOM.DIST",
    arity: Arity::exact(4),
    ..DISCRETE_DIST_BASE_META
};

pub const NEGBINOMDIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NEGBINOMDIST",
    arity: Arity::exact(3),
    ..DISCRETE_DIST_BASE_META
};

pub const EXPON_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.EXPON.DIST",
    arity: Arity::exact(3),
    ..DISCRETE_DIST_BASE_META
};

pub const EXPONDIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.EXPONDIST",
    arity: Arity::exact(3),
    ..DISCRETE_DIST_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum DiscreteDistEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn prepared_len_error(meta: &FunctionMeta, actual: usize) -> DiscreteDistEvalError {
    DiscreteDistEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn number(prepared: &PreparedArgValue) -> Result<f64, DiscreteDistEvalError> {
    coerce_prepared_to_number(prepared).map_err(DiscreteDistEvalError::Coercion)
}

fn cumulative_flag(value: f64) -> bool {
    value != 0.0
}

fn trunc_i64(value: f64) -> Result<i64, WorksheetErrorCode> {
    if !value.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(value.trunc() as i64)
}

fn validate_probability_closed_unit(value: f64) -> Result<(), WorksheetErrorCode> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(())
}

fn ln_choose(n: u64, k: u64) -> Result<f64, WorksheetErrorCode> {
    if k > n {
        return Err(WorksheetErrorCode::Num);
    }
    let k = k.min(n - k);
    let mut acc = 0.0;
    for i in 1..=k {
        acc += ((n - k + i) as f64).ln() - (i as f64).ln();
    }
    Ok(acc)
}

fn choose_direct(n: u64, k: u64) -> f64 {
    let k = k.min(n - k);
    let mut acc = 1.0;
    for i in 1..=k {
        acc *= (n - k + i) as f64;
        acc /= i as f64;
    }
    acc
}

fn pow_u64(base: f64, exponent: u64) -> f64 {
    base.powi(exponent as i32)
}

fn direct_combinatoric_lane(max_n: u64) -> bool {
    max_n <= 200
}

fn binom_pmf_direct(number_s: u64, trials: u64, probability_s: f64) -> f64 {
    choose_direct(trials, number_s)
        * pow_u64(probability_s, number_s)
        * pow_u64(1.0 - probability_s, trials - number_s)
}

pub fn binom_dist_kernel(
    number_s: f64,
    trials: f64,
    probability_s: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    validate_probability_closed_unit(probability_s)?;
    let number_s = trunc_i64(number_s)?;
    let trials = trunc_i64(trials)?;
    if trials < 0 || number_s < 0 || number_s > trials {
        return Err(WorksheetErrorCode::Num);
    }
    let number_s = number_s as u64;
    let trials = trials as u64;
    if cumulative {
        let mut sum = 0.0;
        for k in 0..=number_s {
            sum += binom_dist_kernel(k as f64, trials as f64, probability_s, false)?;
        }
        Ok(sum)
    } else if probability_s == 0.0 {
        Ok(if number_s == 0 { 1.0 } else { 0.0 })
    } else if probability_s == 1.0 {
        Ok(if number_s == trials { 1.0 } else { 0.0 })
    } else {
        let log_pmf = ln_choose(trials, number_s)?
            + (number_s as f64) * probability_s.ln()
            + ((trials - number_s) as f64) * (1.0 - probability_s).ln();
        Ok(log_pmf.exp())
    }
}

pub fn binom_dist_range_kernel(
    trials: f64,
    probability_s: f64,
    number_s: f64,
    number_s2: Option<f64>,
) -> Result<f64, WorksheetErrorCode> {
    validate_probability_closed_unit(probability_s)?;
    let trials = trunc_i64(trials)?;
    let number_s = trunc_i64(number_s)?;
    let number_s2 = trunc_i64(number_s2.unwrap_or(number_s as f64))?;
    if trials < 0 || number_s < 0 || number_s2 < 0 || number_s > number_s2 || number_s2 > trials {
        return Err(WorksheetErrorCode::Num);
    }
    let trials_u = trials as u64;
    let mut sum = 0.0;
    if probability_s != 0.0 && probability_s != 1.0 && direct_combinatoric_lane(trials_u) {
        for k in number_s..=number_s2 {
            sum += binom_pmf_direct(k as u64, trials_u, probability_s);
        }
    } else {
        for k in number_s..=number_s2 {
            sum += binom_dist_kernel(k as f64, trials as f64, probability_s, false)?;
        }
    }
    Ok(sum)
}

pub fn binom_inv_kernel(
    trials: f64,
    probability_s: f64,
    alpha: f64,
) -> Result<f64, WorksheetErrorCode> {
    validate_probability_closed_unit(probability_s)?;
    validate_probability_closed_unit(alpha)?;
    let trials = trunc_i64(trials)?;
    if trials < 0 {
        return Err(WorksheetErrorCode::Num);
    }
    for x in 0..=trials {
        let cdf = binom_dist_kernel(x as f64, trials as f64, probability_s, true)?;
        if cdf >= alpha {
            return Ok(x as f64);
        }
    }
    Ok(trials as f64)
}

pub fn poisson_dist_kernel(x: f64, mean: f64, cumulative: bool) -> Result<f64, WorksheetErrorCode> {
    let x = trunc_i64(x)?;
    if x < 0 || !mean.is_finite() || mean < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    let x = x as u64;
    if cumulative {
        let mut sum = 0.0;
        for k in 0..=x {
            sum += poisson_dist_kernel(k as f64, mean, false)?;
        }
        Ok(sum)
    } else if mean == 0.0 {
        Ok(if x == 0 { 1.0 } else { 0.0 })
    } else {
        let mut ln_fact = 0.0;
        for i in 2..=x {
            ln_fact += (i as f64).ln();
        }
        Ok((-(mean) + (x as f64) * mean.ln() - ln_fact).exp())
    }
}

pub fn hypergeom_dist_kernel(
    sample_s: f64,
    number_sample: f64,
    population_s: f64,
    number_pop: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    let sample_s = trunc_i64(sample_s)?;
    let number_sample = trunc_i64(number_sample)?;
    let population_s = trunc_i64(population_s)?;
    let number_pop = trunc_i64(number_pop)?;
    if sample_s < 0
        || number_sample < 0
        || population_s < 0
        || number_pop < 0
        || population_s > number_pop
        || number_sample > number_pop
        || sample_s > number_sample
        || sample_s > population_s
        || number_sample - sample_s > number_pop - population_s
    {
        return Err(WorksheetErrorCode::Num);
    }
    let sample_s = sample_s as u64;
    let number_sample = number_sample as u64;
    let population_s = population_s as u64;
    let number_pop = number_pop as u64;
    let lower = number_sample.saturating_sub(number_pop - population_s);
    if cumulative {
        let mut sum = 0.0;
        for x in lower..=sample_s {
            sum += hypergeom_dist_kernel(
                x as f64,
                number_sample as f64,
                population_s as f64,
                number_pop as f64,
                false,
            )?;
        }
        Ok(sum)
    } else if direct_combinatoric_lane(number_pop) {
        Ok(choose_direct(population_s, sample_s)
            * choose_direct(number_pop - population_s, number_sample - sample_s)
            / choose_direct(number_pop, number_sample))
    } else {
        let log_pmf = ln_choose(population_s, sample_s)?
            + ln_choose(number_pop - population_s, number_sample - sample_s)?
            - ln_choose(number_pop, number_sample)?;
        Ok(log_pmf.exp())
    }
}

pub fn negbinom_dist_kernel(
    number_f: f64,
    number_s: f64,
    probability_s: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    validate_probability_closed_unit(probability_s)?;
    let number_f = trunc_i64(number_f)?;
    let number_s = trunc_i64(number_s)?;
    if number_f < 0 || number_s <= 0 {
        return Err(WorksheetErrorCode::Num);
    }
    let number_f = number_f as u64;
    let number_s = number_s as u64;
    if cumulative {
        let mut sum = 0.0;
        for failures in 0..=number_f {
            sum += negbinom_dist_kernel(failures as f64, number_s as f64, probability_s, false)?;
        }
        Ok(sum)
    } else if probability_s == 0.0 {
        Ok(0.0)
    } else if probability_s == 1.0 {
        Ok(if number_f == 0 { 1.0 } else { 0.0 })
    } else if direct_combinatoric_lane(number_f + number_s - 1) {
        Ok(choose_direct(number_f + number_s - 1, number_f)
            * pow_u64(probability_s, number_s)
            * pow_u64(1.0 - probability_s, number_f))
    } else {
        let log_pmf = ln_choose(number_f + number_s - 1, number_f)?
            + (number_s as f64) * probability_s.ln()
            + (number_f as f64) * (1.0 - probability_s).ln();
        Ok(log_pmf.exp())
    }
}

pub fn expon_dist_kernel(x: f64, lambda: f64, cumulative: bool) -> Result<f64, WorksheetErrorCode> {
    if !x.is_finite() || !lambda.is_finite() || x < 0.0 || lambda <= 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    if cumulative {
        Ok(1.0 - (-lambda * x).exp())
    } else {
        Ok(lambda * (-lambda * x).exp())
    }
}

fn eval_binom_dist_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, DiscreteDistEvalError> {
    if !BINOM_DIST_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&BINOM_DIST_META, args.len()));
    }
    let number_s = number(&args[0])?;
    let trials = number(&args[1])?;
    let probability_s = number(&args[2])?;
    let cumulative = number(&args[3])?;
    Ok(
        match binom_dist_kernel(number_s, trials, probability_s, cumulative_flag(cumulative)) {
            Ok(value) => EvalValue::Number(value),
            Err(code) => EvalValue::Error(code),
        },
    )
}

fn eval_binom_dist_range_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DiscreteDistEvalError> {
    if !BINOM_DIST_RANGE_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&BINOM_DIST_RANGE_META, args.len()));
    }
    let trials = number(&args[0])?;
    let probability_s = number(&args[1])?;
    let number_s = number(&args[2])?;
    let number_s2 = if args.len() == 4 {
        Some(number(&args[3])?)
    } else {
        None
    };
    Ok(
        match binom_dist_range_kernel(trials, probability_s, number_s, number_s2) {
            Ok(value) => EvalValue::Number(value),
            Err(code) => EvalValue::Error(code),
        },
    )
}

fn eval_binom_inv_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, DiscreteDistEvalError> {
    if !BINOM_INV_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&BINOM_INV_META, args.len()));
    }
    let trials = number(&args[0])?;
    let probability_s = number(&args[1])?;
    let alpha = number(&args[2])?;
    Ok(match binom_inv_kernel(trials, probability_s, alpha) {
        Ok(value) => EvalValue::Number(value),
        Err(code) => EvalValue::Error(code),
    })
}

fn eval_poisson_dist_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DiscreteDistEvalError> {
    if !POISSON_DIST_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&POISSON_DIST_META, args.len()));
    }
    let x = number(&args[0])?;
    let mean = number(&args[1])?;
    let cumulative = number(&args[2])?;
    Ok(
        match poisson_dist_kernel(x, mean, cumulative_flag(cumulative)) {
            Ok(value) => EvalValue::Number(value),
            Err(code) => EvalValue::Error(code),
        },
    )
}

fn eval_hypgeom_dist_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DiscreteDistEvalError> {
    if !HYPGEOM_DIST_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&HYPGEOM_DIST_META, args.len()));
    }
    let sample_s = number(&args[0])?;
    let number_sample = number(&args[1])?;
    let population_s = number(&args[2])?;
    let number_pop = number(&args[3])?;
    let cumulative = number(&args[4])?;
    Ok(
        match hypergeom_dist_kernel(
            sample_s,
            number_sample,
            population_s,
            number_pop,
            cumulative_flag(cumulative),
        ) {
            Ok(value) => EvalValue::Number(value),
            Err(code) => EvalValue::Error(code),
        },
    )
}

fn eval_negbinom_dist_prepared(
    args: &[PreparedArgValue],
) -> Result<EvalValue, DiscreteDistEvalError> {
    if !NEGBINOM_DIST_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&NEGBINOM_DIST_META, args.len()));
    }
    let number_f = number(&args[0])?;
    let number_s = number(&args[1])?;
    let probability_s = number(&args[2])?;
    let cumulative = number(&args[3])?;
    Ok(
        match negbinom_dist_kernel(
            number_f,
            number_s,
            probability_s,
            cumulative_flag(cumulative),
        ) {
            Ok(value) => EvalValue::Number(value),
            Err(code) => EvalValue::Error(code),
        },
    )
}

fn eval_expon_dist_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, DiscreteDistEvalError> {
    if !EXPON_DIST_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&EXPON_DIST_META, args.len()));
    }
    let x = number(&args[0])?;
    let lambda = number(&args[1])?;
    let cumulative = number(&args[2])?;
    Ok(
        match expon_dist_kernel(x, lambda, cumulative_flag(cumulative)) {
            Ok(value) => EvalValue::Number(value),
            Err(code) => EvalValue::Error(code),
        },
    )
}

pub fn eval_binom_dist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_binom_dist_prepared,
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_binom_dist_range_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_binom_dist_range_prepared,
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_binom_inv_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_binom_inv_prepared,
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_binomdist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscreteDistEvalError> {
    eval_binom_dist_surface(args, resolver)
}

pub fn eval_critbinom_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscreteDistEvalError> {
    eval_binom_inv_surface(args, resolver)
}

pub fn eval_poisson_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_poisson_dist_prepared,
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_poisson_dist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscreteDistEvalError> {
    eval_poisson_surface(args, resolver)
}

pub fn eval_hypgeom_dist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_hypgeom_dist_prepared,
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_hypgeomdist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !HYPGEOMDIST_META.arity.accepts(prepared.len()) {
                return Err(prepared_len_error(&HYPGEOMDIST_META, prepared.len()));
            }
            let sample_s = number(&prepared[0])?;
            let number_sample = number(&prepared[1])?;
            let population_s = number(&prepared[2])?;
            let number_pop = number(&prepared[3])?;
            Ok(
                match hypergeom_dist_kernel(
                    sample_s,
                    number_sample,
                    population_s,
                    number_pop,
                    false,
                ) {
                    Ok(value) => EvalValue::Number(value),
                    Err(code) => EvalValue::Error(code),
                },
            )
        },
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_negbinom_dist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_negbinom_dist_prepared,
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_negbinomdist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !NEGBINOMDIST_META.arity.accepts(prepared.len()) {
                return Err(prepared_len_error(&NEGBINOMDIST_META, prepared.len()));
            }
            let number_f = number(&prepared[0])?;
            let number_s = number(&prepared[1])?;
            let probability_s = number(&prepared[2])?;
            Ok(
                match negbinom_dist_kernel(number_f, number_s, probability_s, false) {
                    Ok(value) => EvalValue::Number(value),
                    Err(code) => EvalValue::Error(code),
                },
            )
        },
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_expon_dist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_expon_dist_prepared,
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_expondist_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DiscreteDistEvalError> {
    eval_expon_dist_surface(args, resolver)
}

pub fn map_discrete_dist_error_to_ws(err: &DiscreteDistEvalError) -> WorksheetErrorCode {
    match err {
        DiscreteDistEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        DiscreteDistEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        DiscreteDistEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceLike};
    use std::collections::HashMap;

    struct MockResolver {
        resolved_values: HashMap<String, EvalValue>,
    }

    impl MockResolver {
        fn empty() -> Self {
            Self {
                resolved_values: HashMap::new(),
            }
        }
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            self.resolved_values.get(&reference.target).cloned().ok_or(
                RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                },
            )
        }
    }

    fn num(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    fn text(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(s)))
    }

    fn bool_arg(b: bool) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Logical(b))
    }

    fn assert_ok_number_close(got: Result<EvalValue, DiscreteDistEvalError>, expected: f64) {
        match got {
            Ok(EvalValue::Number(value)) => assert!((value - expected).abs() < 1e-12),
            other => panic!("expected numeric result, got {other:?}"),
        }
    }

    fn assert_bits(actual: f64, expected_bits: u64) {
        assert_eq!(
            actual.to_bits(),
            expected_bits,
            "{actual} vs {}",
            f64::from_bits(expected_bits)
        );
    }

    #[test]
    fn binom_family_matches_seed_lanes() {
        assert!((binom_dist_kernel(2.0, 4.0, 0.25, false).unwrap() - 0.2109375).abs() < 1e-12);
        assert!((binom_dist_kernel(2.0, 4.0, 0.25, true).unwrap() - 0.94921875).abs() < 1e-12);
        assert!(
            (binom_dist_range_kernel(4.0, 0.25, 2.0, Some(3.0)).unwrap() - 0.2578125).abs() < 1e-12
        );
        assert_eq!(binom_inv_kernel(6.0, 0.5, 0.7).unwrap(), 4.0);
        assert_eq!(binom_inv_kernel(6.0, 0.5, 0.0).unwrap(), 0.0);
    }

    #[test]
    fn finite_combinatoric_witnesses_match_excel_bits() {
        assert_bits(
            binom_dist_range_kernel(4.0, 0.25, 2.0, Some(3.0)).unwrap(),
            0x3fd0_8000_0000_0000,
        );
        assert_bits(
            negbinom_dist_kernel(5.0, 3.0, 0.4, true).unwrap(),
            0x3fe5_e849_aaee_d68d,
        );
    }

    #[test]
    fn poisson_family_matches_seed_lanes() {
        assert!((poisson_dist_kernel(3.0, 2.0, false).unwrap() - 0.1804470443154836).abs() < 1e-12);
        assert!((poisson_dist_kernel(3.0, 2.0, true).unwrap() - 0.857123460498547).abs() < 1e-12);
        assert_eq!(poisson_dist_kernel(0.0, 0.0, false).unwrap(), 1.0);
    }

    #[test]
    fn hypergeom_family_matches_seed_lanes() {
        assert!(
            (hypergeom_dist_kernel(2.0, 5.0, 4.0, 10.0, false).unwrap() - 0.47619047619047616)
                .abs()
                < 1e-12
        );
        assert!(
            (hypergeom_dist_kernel(2.0, 5.0, 4.0, 10.0, true).unwrap() - 0.7380952380952381).abs()
                < 1e-12
        );
    }

    #[test]
    fn negbinom_family_matches_seed_lanes() {
        assert!((negbinom_dist_kernel(3.0, 2.0, 0.5, false).unwrap() - 0.125).abs() < 1e-12);
        assert!((negbinom_dist_kernel(3.0, 2.0, 0.5, true).unwrap() - 0.8125).abs() < 1e-12);
        assert_eq!(negbinom_dist_kernel(0.0, 2.0, 1.0, false).unwrap(), 1.0);
    }

    #[test]
    fn expon_family_matches_seed_lanes() {
        assert!((expon_dist_kernel(2.0, 1.5, false).unwrap() - 0.07468060255179591).abs() < 1e-12);
        assert!((expon_dist_kernel(2.0, 1.5, true).unwrap() - 0.950212931632136).abs() < 1e-12);
    }

    #[test]
    fn domain_errors_match_seed_expectations() {
        assert_eq!(
            binom_dist_kernel(5.0, 4.0, 0.25, false),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            binom_dist_range_kernel(4.0, 0.25, 3.0, Some(2.0)),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            poisson_dist_kernel(-1.0, 2.0, false),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            hypergeom_dist_kernel(4.0, 5.0, 2.0, 10.0, false),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            negbinom_dist_kernel(1.0, 0.0, 0.5, false),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            expon_dist_kernel(-1.0, 1.5, false),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn compatibility_alias_surfaces_follow_modern_kernels() {
        let resolver = MockResolver::empty();
        assert_ok_number_close(
            eval_binomdist_surface(&[num(2.0), num(4.0), num(0.25), bool_arg(false)], &resolver),
            0.2109375,
        );
        assert_eq!(
            eval_critbinom_surface(&[num(6.0), num(0.5), num(0.7)], &resolver),
            Ok(EvalValue::Number(4.0))
        );
        assert_ok_number_close(
            eval_poisson_surface(&[num(3.0), num(2.0), bool_arg(false)], &resolver),
            0.1804470443154836,
        );
        assert_ok_number_close(
            eval_hypgeomdist_surface(&[num(2.0), num(5.0), num(4.0), num(10.0)], &resolver),
            0.47619047619047616,
        );
        assert_ok_number_close(
            eval_negbinomdist_surface(&[num(3.0), num(2.0), num(0.5)], &resolver),
            0.125,
        );
        assert_ok_number_close(
            eval_expondist_surface(&[num(2.0), num(1.5), bool_arg(true)], &resolver),
            0.950212931632136,
        );
        assert_ok_number_close(
            eval_hypgeom_dist_surface(
                &[num(2.0), num(5.0), num(4.0), num(10.0), bool_arg(true)],
                &resolver,
            ),
            0.7380952380952381,
        );
        assert_ok_number_close(
            eval_negbinom_dist_surface(&[num(3.0), num(2.0), num(0.5), bool_arg(true)], &resolver),
            0.8125,
        );
    }

    #[test]
    fn values_only_surface_coercion_admits_numeric_text_and_logical_flags() {
        let resolver = MockResolver::empty();
        assert_eq!(
            eval_binom_inv_surface(&[text("6"), num(0.5), text("0.7")], &resolver),
            Ok(EvalValue::Number(4.0))
        );
        assert_ok_number_close(
            eval_poisson_dist_surface(&[num(3.0), text("2"), bool_arg(true)], &resolver),
            0.857123460498547,
        );
        assert_ok_number_close(
            eval_expon_dist_surface(&[num(2.0), text("1.5"), bool_arg(false)], &resolver),
            0.07468060255179591,
        );
    }

    #[test]
    fn metadata_shapes_and_error_mapping_are_exercised() {
        assert!(BINOMDIST_META.arity.accepts(4));
        assert!(CRITBINOM_META.arity.accepts(3));
        assert!(POISSON_META.arity.accepts(3));
        assert!(EXPONDIST_META.arity.accepts(3));
        assert_eq!(
            map_discrete_dist_error_to_ws(&DiscreteDistEvalError::ArityMismatch {
                expected_min: 3,
                expected_max: 4,
                actual: 2,
            }),
            WorksheetErrorCode::Value,
        );
    }

    #[test]
    fn missing_optional_upper_bound_defaults_to_exact_binom_mass() {
        let resolver = MockResolver::empty();
        assert_ok_number_close(
            eval_binom_dist_range_surface(&[num(4.0), num(0.25), num(2.0)], &resolver),
            0.2109375,
        );
    }
}
