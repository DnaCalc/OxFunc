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

const OPTIONAL_ARITY_5: Arity = Arity { min: 4, max: 5 };
const OPTIONAL_ARITY_7: Arity = Arity { min: 5, max: 7 };
const EPSILON: f64 = 1.0e-12;

const DEPRECIATION_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DEPRECIATION_BASE",
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

pub const SLN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SLN",
    arity: Arity::exact(3),
    ..DEPRECIATION_BASE_META
};
pub const SYD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.SYD",
    arity: Arity::exact(4),
    ..DEPRECIATION_BASE_META
};
pub const DB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DB",
    arity: OPTIONAL_ARITY_5,
    ..DEPRECIATION_BASE_META
};
pub const DDB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DDB",
    arity: OPTIONAL_ARITY_5,
    ..DEPRECIATION_BASE_META
};
pub const VDB_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.VDB",
    arity: OPTIONAL_ARITY_7,
    ..DEPRECIATION_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum DepreciationEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> DepreciationEvalError {
    DepreciationEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn required_number(arg: &PreparedArgValue) -> Result<f64, DepreciationEvalError> {
    coerce_prepared_to_number(arg).map_err(DepreciationEvalError::Coercion)
}

fn optional_number(
    args: &[PreparedArgValue],
    idx: usize,
    default: f64,
) -> Result<f64, DepreciationEvalError> {
    match args.get(idx) {
        None | Some(PreparedArgValue::MissingArg) | Some(PreparedArgValue::EmptyCell) => {
            Ok(default)
        }
        Some(arg) => required_number(arg),
    }
}

fn validate_finite(values: &[f64]) -> Result<(), WorksheetErrorCode> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(WorksheetErrorCode::Num)
    }
}

fn round_half_away_from_zero(value: f64, digits: i32) -> f64 {
    let factor = 10_f64.powi(digits);
    let scaled = value * factor;
    let rounded = (scaled.abs() + 0.5).floor().copysign(scaled);
    rounded / factor
}

pub fn sln_kernel(cost: f64, salvage: f64, life: f64) -> Result<f64, WorksheetErrorCode> {
    validate_finite(&[cost, salvage, life])?;
    if cost < 0.0 || salvage < 0.0 || life < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    if life.abs() <= EPSILON {
        return Err(WorksheetErrorCode::Div0);
    }
    Ok((cost - salvage) / life)
}

pub fn syd_kernel(cost: f64, salvage: f64, life: f64, per: f64) -> Result<f64, WorksheetErrorCode> {
    validate_finite(&[cost, salvage, life, per])?;
    if cost < 0.0 || salvage < 0.0 || life <= 0.0 || per <= 0.0 || per > life {
        return Err(WorksheetErrorCode::Num);
    }
    let denominator = life * (life + 1.0);
    if denominator.abs() <= EPSILON {
        return Err(WorksheetErrorCode::Div0);
    }
    Ok((cost - salvage) * (life - per + 1.0) * 2.0 / denominator)
}

fn db_rate(cost: f64, salvage: f64, life: f64) -> Result<f64, WorksheetErrorCode> {
    validate_finite(&[cost, salvage, life])?;
    if cost <= 0.0 || salvage < 0.0 || life <= 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    let ratio = salvage / cost;
    if ratio < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    let rate = 1.0 - ratio.powf(1.0 / life);
    if !rate.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(round_half_away_from_zero(rate, 3))
}

pub fn db_kernel(
    cost: f64,
    salvage: f64,
    life: f64,
    period: f64,
    month: f64,
) -> Result<f64, WorksheetErrorCode> {
    validate_finite(&[cost, salvage, life, period, month])?;
    if cost <= 0.0 || salvage < 0.0 || life <= 0.0 || period <= 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    if month <= 0.0 || month > 12.0 {
        return Err(WorksheetErrorCode::Num);
    }
    let max_period = if month < 12.0 { life + 1.0 } else { life };
    if period > max_period + EPSILON {
        return Err(WorksheetErrorCode::Num);
    }
    let rate = db_rate(cost, salvage, life)?;
    let first = cost * rate * month / 12.0;
    if period <= 1.0 {
        return Ok(first);
    }

    let mut total = first;
    let whole_period = period.floor() as usize;
    let last_full_regular_period = life.floor() as usize;
    let mut dep = first;
    for p in 2..=whole_period {
        dep = if month < 12.0 && p == last_full_regular_period + 1 {
            (cost - total) * rate * (12.0 - month) / 12.0
        } else {
            (cost - total) * rate
        };
        total += dep;
    }

    let fractional = period - period.floor();
    if fractional > EPSILON {
        let next_period = whole_period + 1;
        let next_dep = if month < 12.0 && next_period == last_full_regular_period + 1 {
            (cost - total) * rate * (12.0 - month) / 12.0
        } else {
            (cost - total) * rate
        };
        Ok(dep + fractional * next_dep)
    } else {
        Ok(dep)
    }
}

fn declining_interval_depreciation(
    cost: f64,
    salvage: f64,
    life: f64,
    start_period: f64,
    end_period: f64,
    factor: f64,
    no_switch: bool,
) -> Result<f64, WorksheetErrorCode> {
    validate_finite(&[cost, salvage, life, start_period, end_period, factor])?;
    if cost < 0.0 || salvage < 0.0 || life <= 0.0 || factor <= 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    if start_period < 0.0 || end_period < start_period {
        return Err(WorksheetErrorCode::Num);
    }
    if (end_period - start_period).abs() <= EPSILON {
        return Ok(0.0);
    }

    let mut elapsed = 0.0;
    let mut book = cost;
    let mut total = 0.0;
    let declining_rate = factor / life;
    while elapsed < end_period - EPSILON && elapsed < life - EPSILON {
        let full_start = elapsed;
        let full_end = (elapsed + 1.0).min(life);
        let segment_len = full_end - full_start;
        let remaining_basis = (book - salvage).max(0.0);
        if remaining_basis <= EPSILON {
            break;
        }

        let declining = (book * declining_rate).min(remaining_basis);
        let straight = remaining_basis / (life - elapsed);
        let full_period_dep = if no_switch || declining >= straight {
            declining
        } else {
            straight
        }
        .min(remaining_basis);

        let overlap = (end_period.min(full_end) - start_period.max(full_start)).max(0.0);
        if overlap > EPSILON {
            total += full_period_dep * overlap / segment_len;
        }

        book -= full_period_dep;
        elapsed = full_end;
    }

    Ok(total)
}

pub fn ddb_kernel(
    cost: f64,
    salvage: f64,
    life: f64,
    period: f64,
    factor: f64,
) -> Result<f64, WorksheetErrorCode> {
    if period <= 0.0 || period > life + EPSILON {
        return Err(WorksheetErrorCode::Num);
    }
    declining_interval_depreciation(cost, salvage, life, period - 1.0, period, factor, true)
}

pub fn vdb_kernel(
    cost: f64,
    salvage: f64,
    life: f64,
    start_period: f64,
    end_period: f64,
    factor: f64,
    no_switch: bool,
) -> Result<f64, WorksheetErrorCode> {
    declining_interval_depreciation(
        cost,
        salvage,
        life,
        start_period,
        end_period,
        factor,
        no_switch,
    )
}

fn eval_sln_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, DepreciationEvalError> {
    if !SLN_META.arity.accepts(args.len()) {
        return Err(arity_error(&SLN_META, args.len()));
    }
    let cost = required_number(&args[0])?;
    let salvage = required_number(&args[1])?;
    let life = required_number(&args[2])?;
    Ok(match sln_kernel(cost, salvage, life) {
        Ok(value) => EvalValue::Number(value),
        Err(code) => EvalValue::Error(code),
    })
}

fn eval_syd_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, DepreciationEvalError> {
    if !SYD_META.arity.accepts(args.len()) {
        return Err(arity_error(&SYD_META, args.len()));
    }
    let cost = required_number(&args[0])?;
    let salvage = required_number(&args[1])?;
    let life = required_number(&args[2])?;
    let per = required_number(&args[3])?;
    Ok(match syd_kernel(cost, salvage, life, per) {
        Ok(value) => EvalValue::Number(value),
        Err(code) => EvalValue::Error(code),
    })
}

fn eval_db_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, DepreciationEvalError> {
    if !DB_META.arity.accepts(args.len()) {
        return Err(arity_error(&DB_META, args.len()));
    }
    let cost = required_number(&args[0])?;
    let salvage = required_number(&args[1])?;
    let life = required_number(&args[2])?;
    let period = required_number(&args[3])?;
    let month = optional_number(args, 4, 12.0)?;
    Ok(match db_kernel(cost, salvage, life, period, month) {
        Ok(value) => EvalValue::Number(value),
        Err(code) => EvalValue::Error(code),
    })
}

fn eval_ddb_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, DepreciationEvalError> {
    if !DDB_META.arity.accepts(args.len()) {
        return Err(arity_error(&DDB_META, args.len()));
    }
    let cost = required_number(&args[0])?;
    let salvage = required_number(&args[1])?;
    let life = required_number(&args[2])?;
    let period = required_number(&args[3])?;
    let factor = optional_number(args, 4, 2.0)?;
    Ok(match ddb_kernel(cost, salvage, life, period, factor) {
        Ok(value) => EvalValue::Number(value),
        Err(code) => EvalValue::Error(code),
    })
}

fn eval_vdb_prepared(args: &[PreparedArgValue]) -> Result<EvalValue, DepreciationEvalError> {
    if !VDB_META.arity.accepts(args.len()) {
        return Err(arity_error(&VDB_META, args.len()));
    }
    let cost = required_number(&args[0])?;
    let salvage = required_number(&args[1])?;
    let life = required_number(&args[2])?;
    let start_period = required_number(&args[3])?;
    let end_period = required_number(&args[4])?;
    let factor = optional_number(args, 5, 2.0)?;
    let no_switch = optional_number(args, 6, 0.0)? != 0.0;
    Ok(
        match vdb_kernel(
            cost,
            salvage,
            life,
            start_period,
            end_period,
            factor,
            no_switch,
        ) {
            Ok(value) => EvalValue::Number(value),
            Err(code) => EvalValue::Error(code),
        },
    )
}

pub fn eval_sln_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DepreciationEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_sln_prepared,
        DepreciationEvalError::Coercion,
    )
}

pub fn eval_syd_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DepreciationEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_syd_prepared,
        DepreciationEvalError::Coercion,
    )
}

pub fn eval_db_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DepreciationEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_db_prepared,
        DepreciationEvalError::Coercion,
    )
}

pub fn eval_ddb_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DepreciationEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_ddb_prepared,
        DepreciationEvalError::Coercion,
    )
}

pub fn eval_vdb_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<EvalValue, DepreciationEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_vdb_prepared,
        DepreciationEvalError::Coercion,
    )
}

pub fn map_depreciation_error_to_ws(error: &DepreciationEvalError) -> WorksheetErrorCode {
    match error {
        DepreciationEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        DepreciationEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        DepreciationEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ExcelText, ReferenceKind, ReferenceLike};

    struct NoRefResolver;

    impl ReferenceResolver for NoRefResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    fn num(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }

    fn text(s: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
            s.encode_utf16().collect(),
        )))
    }

    #[test]
    fn depreciation_metadata_matches_expected_shape() {
        assert_eq!(SLN_META.arity, Arity::exact(3));
        assert_eq!(DB_META.arity.max, 5);
        assert_eq!(
            DDB_META.arg_preparation_profile,
            ArgPreparationProfile::ValuesOnlyPreAdapter
        );
        assert_eq!(VDB_META.function_id, "FUNC.VDB");
    }

    #[test]
    fn sln_and_syd_match_support_examples() {
        assert_eq!(sln_kernel(30000.0, 7500.0, 10.0), Ok(2250.0));
        match syd_kernel(30000.0, 7500.0, 10.0, 1.0).unwrap() {
            n => assert!((n - 4090.909090909091).abs() < 1.0e-9),
        }
        match syd_kernel(30000.0, 7500.0, 10.0, 10.0).unwrap() {
            n => assert!((n - 409.09090909090907).abs() < 1.0e-9),
        }
    }

    #[test]
    fn db_matches_support_examples() {
        assert!(
            (db_kernel(1_000_000.0, 100_000.0, 6.0, 1.0, 7.0).unwrap() - 186083.33333333334).abs()
                < 1.0e-9
        );
        assert!(
            (db_kernel(1_000_000.0, 100_000.0, 6.0, 2.0, 7.0).unwrap() - 259639.41666666666).abs()
                < 1.0e-9
        );
        assert!(
            (db_kernel(1_000_000.0, 100_000.0, 6.0, 7.0, 7.0).unwrap() - 15845.098473848071).abs()
                < 1.0e-9
        );
    }

    #[test]
    fn ddb_and_vdb_match_seeded_examples() {
        assert_eq!(ddb_kernel(2400.0, 300.0, 10.0, 1.0, 2.0), Ok(480.0));
        assert_eq!(ddb_kernel(2400.0, 300.0, 10.0, 2.0, 2.0), Ok(384.0));
        assert!(
            (vdb_kernel(2400.0, 300.0, 3650.0, 0.0, 1.0, 2.0, false).unwrap() - 1.3150684931506849)
                .abs()
                < 1.0e-12
        );
        assert!(
            (vdb_kernel(2400.0, 300.0, 120.0, 0.0, 1.0, 2.0, false).unwrap() - 40.0).abs()
                < 1.0e-12
        );
        assert!(
            vdb_kernel(2400.0, 300.0, 120.0, 6.0, 18.0, 2.0, false)
                .unwrap()
                .to_bits()
                == 0x4078_c4e5_981b_af06
        );
        assert!(
            (vdb_kernel(2400.0, 300.0, 120.0, 6.0, 18.0, 1.5, false).unwrap() - 311.8089366582341)
                .abs()
                < 1.0e-9
        );
        assert!(
            (vdb_kernel(2400.0, 300.0, 10.0, 0.0, 0.875, 1.5, false).unwrap() - 315.0).abs()
                < 1.0e-12
        );
    }

    #[test]
    fn surface_evaluators_apply_defaults_and_numeric_coercion() {
        let resolver = NoRefResolver;
        assert_eq!(
            eval_sln_surface(&[num(30000.0), num(7500.0), num(10.0)], &resolver),
            Ok(EvalValue::Number(2250.0))
        );
        assert_eq!(
            eval_ddb_surface(&[num(2400.0), num(300.0), num(10.0), num(1.0)], &resolver),
            Ok(EvalValue::Number(480.0))
        );
        let got = eval_vdb_surface(
            &[
                num(2400.0),
                num(300.0),
                num(10.0),
                num(0.0),
                num(0.875),
                num(1.5),
            ],
            &resolver,
        )
        .unwrap();
        match got {
            EvalValue::Number(n) => assert!((n - 315.0).abs() < 1.0e-12),
            other => panic!("expected number, got {other:?}"),
        }
        assert_eq!(
            eval_db_surface(
                &[
                    num(1_000_000.0),
                    num(100_000.0),
                    num(6.0),
                    num(1.0),
                    text("7")
                ],
                &resolver,
            ),
            Ok(EvalValue::Number(186083.33333333334))
        );
    }

    #[test]
    fn domain_and_mapping_lanes_are_pinned() {
        let resolver = NoRefResolver;
        assert_eq!(
            sln_kernel(30000.0, 7500.0, 0.0),
            Err(WorksheetErrorCode::Div0)
        );
        assert_eq!(
            syd_kernel(30000.0, 7500.0, 10.0, 11.0),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            db_kernel(1000.0, 100.0, 6.0, 1.0, 13.0),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            vdb_kernel(2400.0, 300.0, 10.0, 2.0, 1.0, 2.0, false),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            map_depreciation_error_to_ws(&DepreciationEvalError::ArityMismatch {
                expected_min: 3,
                expected_max: 3,
                actual: 2,
            }),
            WorksheetErrorCode::Value
        );
        assert_eq!(
            eval_sln_surface(&[num(30000.0), num(7500.0)], &resolver),
            Err(DepreciationEvalError::ArityMismatch {
                expected_min: 3,
                expected_max: 3,
                actual: 2,
            })
        );
    }

    #[test]
    fn no_switch_flag_seed_lane_is_currently_equal() {
        let switched = vdb_kernel(2400.0, 300.0, 10.0, 6.0, 8.0, 2.0, false).unwrap();
        let pure_declining = vdb_kernel(2400.0, 300.0, 10.0, 6.0, 8.0, 2.0, true).unwrap();
        assert!((switched - pure_declining).abs() < 1.0e-12);
    }

    #[test]
    fn no_ref_resolver_reference_lane_stays_unresolved() {
        let resolver = NoRefResolver;
        let got = eval_sln_surface(
            &[
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "A1".to_string(),
                }),
                num(0.0),
                num(10.0),
            ],
            &resolver,
        );
        assert_eq!(
            got,
            Err(DepreciationEvalError::Coercion(
                CoercionError::RefResolution(RefResolutionError::UnresolvedReference {
                    target: "A1".to_string(),
                })
            ))
        );
    }
}
