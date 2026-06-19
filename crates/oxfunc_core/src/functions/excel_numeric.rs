use crate::value::WorksheetErrorCode;

pub(crate) fn excel_underflow_to_zero(value: f64) -> f64 {
    if value.is_finite() && value != 0.0 && value.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        value
    }
}

/// Excel never publishes a non-finite numeric result: a kernel that overflows to
/// `±Inf` or produces `NaN` surfaces as `#NUM!`. Use this to guard kernels whose
/// output can run off to infinity (SINH/COSH, PERMUTATIONA, ...).
///
/// Functions that *saturate* in Excel (e.g. COTH/TANH/FISHERINV return `±1` for
/// large arguments) must NOT use this guard — they should produce the saturated
/// value directly.
pub(crate) fn finite_or_num(value: f64) -> Result<f64, WorksheetErrorCode> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(WorksheetErrorCode::Num)
    }
}

/// What Excel publishes when a real kernel's raw result is non-finite (`±Inf` / `NaN`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NonFinite {
    /// Pass the raw value through — the kernel cannot produce a non-finite result for
    /// any valid argument (so this never fires; it documents that intent).
    Allow,
    /// Overflow / `NaN` → `#NUM!` (EXP, SINH, COSH, FACT, FACTDOUBLE, DEGREES, PERMUTATIONA).
    Num,
    /// Overflow → saturate to `sign(arg)` i.e. `±1` (COTH).
    SaturateSign,
}

/// Declarative Excel publication rule for a real-valued numeric kernel: how Excel maps
/// the kernel's raw IEEE-754 output — plus an optional argument-domain pre-check — to a
/// published worksheet value.
///
/// Declared once per function and applied at every dispatch site (scalar, array-lift,
/// by-index), so the Excel behaviour is explicit and cannot diverge between paths. This
/// is the declarative companion to the `FunctionMeta` profile enums; it currently lives
/// as a per-function const surfaced through `real_result_policy(function_id)`, with a
/// documented path to promote it into `FunctionMeta` itself.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ExcelRealPolicy {
    /// Reject `|arg| >= limit` with `#NUM!` before publishing (circular trig: `2^27`).
    pub arg_abs_limit: Option<f64>,
    /// How a non-finite raw result is published.
    pub non_finite: NonFinite,
}

/// Excel's circular trig argument-reduction limit: `SIN/COS/TAN/COT/SEC/CSC` return
/// `#NUM!` once `|x| >= 2^27`. Verified live Excel 16.0 build 20026.
pub(crate) const EXCEL_TRIG_MAX_ABS_ARG: f64 = 134_217_728.0; // 2^27

impl ExcelRealPolicy {
    /// No argument guard, raw passes through unchanged.
    pub const PASS: Self = Self {
        arg_abs_limit: None,
        non_finite: NonFinite::Allow,
    };
    /// No argument guard; overflow/`NaN` → `#NUM!`.
    pub const FINITE: Self = Self {
        arg_abs_limit: None,
        non_finite: NonFinite::Num,
    };
    /// No argument guard; overflow → saturate to `±1` (COTH).
    pub const SATURATE_SIGN: Self = Self {
        arg_abs_limit: None,
        non_finite: NonFinite::SaturateSign,
    };
    /// Circular trig: `|x| >= 2^27` → `#NUM!`; any non-finite result → `#NUM!`.
    pub const CIRCULAR_TRIG: Self = Self {
        arg_abs_limit: Some(EXCEL_TRIG_MAX_ABS_ARG),
        non_finite: NonFinite::Num,
    };

    /// Argument-domain pre-check only — for `Result`-returning kernels (COT/SEC/CSC) that
    /// own their compute path and just need the `|arg|` guard.
    pub fn check_arg(self, arg: f64) -> Result<(), WorksheetErrorCode> {
        match self.arg_abs_limit {
            Some(limit) if arg.abs() >= limit => Err(WorksheetErrorCode::Num),
            _ => Ok(()),
        }
    }

    /// Publish a raw `f64` kernel result given its argument.
    pub fn publish(self, arg: f64, raw: f64) -> Result<f64, WorksheetErrorCode> {
        self.check_arg(arg)?;
        if raw.is_finite() {
            return Ok(raw);
        }
        match self.non_finite {
            NonFinite::Allow => Ok(raw),
            NonFinite::Num => Err(WorksheetErrorCode::Num),
            NonFinite::SaturateSign => Ok(arg.signum()),
        }
    }

    /// Wrap a raw `f64 -> f64` kernel into the guarded `Fn(f64) -> Result<f64, _>` the
    /// unary surface helpers consume — so the policy applies to scalars and array
    /// elements alike, at every dispatch site.
    pub fn wrap(
        self,
        raw_kernel: impl Fn(f64) -> f64 + Copy,
    ) -> impl Fn(f64) -> Result<f64, WorksheetErrorCode> + Copy {
        move |n| self.publish(n, raw_kernel(n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_or_num_passes_finite_and_rejects_non_finite() {
        assert_eq!(finite_or_num(1.5), Ok(1.5));
        assert_eq!(finite_or_num(0.0), Ok(0.0));
        assert_eq!(finite_or_num(f64::INFINITY), Err(WorksheetErrorCode::Num));
        assert_eq!(
            finite_or_num(f64::NEG_INFINITY),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(finite_or_num(f64::NAN), Err(WorksheetErrorCode::Num));
    }

    #[test]
    fn excel_real_policy_finite_maps_overflow_to_num() {
        let p = ExcelRealPolicy::FINITE;
        assert_eq!(
            p.publish(1000.0, 1000.0_f64.exp()),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(p.publish(1.0, 1.0_f64.exp()), Ok(1.0_f64.exp()));
    }

    #[test]
    fn excel_real_policy_saturate_sign_publishes_unit() {
        let p = ExcelRealPolicy::SATURATE_SIGN;
        // cosh/sinh = Inf/Inf = NaN at large |x|.
        assert_eq!(p.publish(800.0, f64::NAN), Ok(1.0));
        assert_eq!(p.publish(-800.0, f64::NAN), Ok(-1.0));
        assert_eq!(p.publish(1.0, 1.3130352854993312), Ok(1.3130352854993312));
    }

    #[test]
    fn excel_real_policy_circular_trig_guards_arg_at_two_pow_27() {
        let p = ExcelRealPolicy::CIRCULAR_TRIG;
        assert_eq!(p.publish(134_217_727.0, 0.5), Ok(0.5));
        assert_eq!(p.publish(134_217_728.0, 0.5), Err(WorksheetErrorCode::Num));
        assert_eq!(p.publish(-134_217_728.0, 0.5), Err(WorksheetErrorCode::Num));
        assert_eq!(p.check_arg(134_217_728.0), Err(WorksheetErrorCode::Num));
        assert_eq!(p.check_arg(1.0), Ok(()));
    }

    #[test]
    fn excel_real_policy_wrap_threads_through_kernel() {
        let sin = ExcelRealPolicy::CIRCULAR_TRIG.wrap(f64::sin);
        assert_eq!(sin(0.0), Ok(0.0));
        assert_eq!(sin(200_000_000.0), Err(WorksheetErrorCode::Num));
    }
}
