//! Replay the W109 BESSELJ internal-trig battery.
//!
//! Scores the shipping kernel and the full 16-member matrix of independent
//! J0/J1 sine/cosine substitutions through Excel's established `fFSIN`/`fFCOS`
//! graphs.  Body arithmetic remains frozen unless a follow-up staging model is
//! added explicitly.

use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::bessel_convert_family::besselj_kernel;
use serde_json::{Value, json};
#[cfg(target_arch = "x86_64")]
use std::arch::asm;
use std::collections::{BTreeMap, HashMap};
#[cfg(windows)]
use std::ffi::{CString, c_void};
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::sync::OnceLock;

const ACC: f64 = 40.0;
const BIGNO: f64 = 1.0e10;
const BIGNI: f64 = 1.0e-10;
const NR_2_OVER_PI: f64 = 0.636_619_772;

#[derive(Clone)]
struct Witness {
    id: String,
    class: String,
    x: f64,
    order: f64,
    expected: u64,
}

#[derive(Clone)]
struct CosCapturedRow {
    id: String,
    class: String,
    x: f64,
    expected: u64,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct BodyStaging {
    /// Diagnostic only: inject fresh captured worksheet-COS bits at two phases.
    live_cos_corrections: bool,
    /// Route worksheet COS through the executable tangent-square sine branch.
    tangent_square_cos: bool,
    /// J0 setup bits: 8/x, z*z, x-phase, (2/pi)/x, sqrt, phase->fFCOS unspilled.
    j0_setup: u8,
    /// J0 wrapper bits: cos*p, z*sin, (z*sin)*q, subtraction, scale*body.
    j0: u8,
    /// J0 P Horner bits, two per multiply/add step from high to low degree.
    j0_p: u8,
    /// J0 Q Horner bits, two per multiply/add step from high to low degree.
    j0_q: u8,
    /// 0 disables; 1..=16 select an extended wrapper with spill-barrier mask mode-1.
    j0_continuous: u8,
    /// 0 disables; 1..=16 retain raw fFCOS through wrapper mask mode-1.
    j0_raw_cos: u8,
    /// 0 disables; 1 retains raw FSQRT, 2 also retains the scale division.
    j0_raw_scale: u8,
    /// J1 setup bits: 8/x, z*z, x-phase, (2/pi)/x, sqrt, phase->fFCOS unspilled.
    j1_setup: u8,
    /// J1 wrapper bits: cos*p, z*sin, (z*sin)*q, subtraction, scale*body.
    j1: u8,
    /// 0 disables; 1..=16 select an extended wrapper with spill-barrier mask mode-1.
    j1_continuous: u8,
    /// 0 disables; 1..=16 retain raw fFCOS through wrapper mask mode-1.
    j1_raw_cos: u8,
    /// 0 disables; 1 retains raw FSQRT, 2 also retains the scale division.
    j1_raw_scale: u8,
    /// Upward-recurrence bits: 2/x, j*tox, (j*tox)*current, subtraction.
    recurrence: u8,
}

#[derive(Clone, Copy)]
struct TrigRouting {
    j0_sin: bool,
    j0_cos: bool,
    j1_sin: bool,
    j1_cos: bool,
}

impl TrigRouting {
    const PLATFORM: Self = Self {
        j0_sin: false,
        j0_cos: false,
        j1_sin: false,
        j1_cos: false,
    };
    const J0_COS: Self = Self {
        j0_sin: false,
        j0_cos: true,
        j1_sin: false,
        j1_cos: false,
    };

    fn from_mask(mask: u8) -> Self {
        Self {
            j0_sin: mask & 1 != 0,
            j0_cos: mask & 2 != 0,
            j1_sin: mask & 4 != 0,
            j1_cos: mask & 8 != 0,
        }
    }

    fn label(self) -> String {
        format!(
            "J0(s{},c{}) J1(s{},c{})",
            u8::from(self.j0_sin),
            u8::from(self.j0_cos),
            u8::from(self.j1_sin),
            u8::from(self.j1_cos)
        )
    }
}

fn parse_bits(text: &str) -> f64 {
    f64::from_bits(
        u64::from_str_radix(text.trim_start_matches("0x"), 16).expect("valid hexadecimal f64 bits"),
    )
}

fn horner(y: f64, coeffs: &[f64]) -> f64 {
    coeffs
        .iter()
        .rev()
        .fold(0.0, |acc, coefficient| acc * y + coefficient)
}

fn format_bits(value: f64) -> String {
    format!("0x{:016x}", value.to_bits())
}

fn offset_positive_ulp(value: f64, offset: i32) -> f64 {
    assert!(value.is_sign_positive() && value.is_finite());
    f64::from_bits((i128::from(value.to_bits()) + i128::from(offset)) as u64)
}

fn j0_from_trig(x: f64, cosine: f64, sine: f64, x87_cosine_p: bool) -> f64 {
    let ax = x.abs();
    let z = 8.0 / ax;
    let y = z * z;
    let p = horner(
        y,
        &[
            1.0,
            -0.001_098_628_267,
            0.000_027_345_104_07,
            -0.000_002_073_370_639,
            0.000_000_209_388_721_1,
        ],
    );
    let q = horner(
        y,
        &[
            -0.015_624_999_95,
            0.000_143_048_876_5,
            -0.000_006_911_147_651,
            0.000_000_762_109_516_1,
            -0.000_000_093_493_515_2,
        ],
    );
    let cosine_p = if x87_cosine_p {
        rx::x87_mul(cosine, p)
    } else {
        cosine * p
    };
    (NR_2_OVER_PI / ax).sqrt() * (cosine_p - z * sine * q)
}

fn j1_from_trig(x: f64, cosine: f64, sine: f64, x87_cosine_p: bool) -> f64 {
    let ax = x.abs();
    let z = 8.0 / ax;
    let y = z * z;
    let p = horner(
        y,
        &[
            1.0,
            0.001_831_05,
            -0.000_035_163_964_96,
            -0.000_035_163_964_96,
            0.000_002_457_520_174,
            -0.000_000_240_337_019,
        ],
    );
    let q = horner(
        y,
        &[
            0.046_874_999_95,
            -0.000_200_269_087_3,
            0.000_008_449_199_096,
            -0.000_000_882_289_87,
            0.000_000_105_787_412,
        ],
    );
    let cosine_p = if x87_cosine_p {
        rx::x87_mul(cosine, p)
    } else {
        cosine * p
    };
    let value = (NR_2_OVER_PI / ax).sqrt() * (cosine_p - z * sine * q);
    if x < 0.0 { -value } else { value }
}

fn excel_sin_ext_value(x: &rx::Ext80, cw: u16) -> rx::Ext80 {
    let (residue, quotient) = rx::ext_prem1_quo(x, &rx::ext_pi(), cw);
    let mut value = rx::ext_sin(&residue, cw);
    if quotient & 1 == 1 {
        value = rx::ext_chs(&value, cw);
    }
    value
}

fn excel_sin_model(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    let cw = rx::CW_PC64_RN;
    rx::ext_to_f64(&excel_sin_ext_value(&rx::ext_from_f64(x), cw), cw)
}

fn excel_cos_ext_value(x: &rx::Ext80) -> rx::Ext80 {
    let cw = rx::CW_PC64_RN;
    let minus_one = rx::ext_chs(&rx::ext_one(), cw);
    let pi_half = rx::ext_scale(&rx::ext_pi(), &minus_one, cw);
    let xa = rx::ext_abs(x, cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &pi_half, cw);
    match quotient & 3 {
        0 => rx::ext_cos(&residue, cw),
        1 => rx::ext_chs(&rx::ext_sin(&residue, cw), cw),
        2 => rx::ext_chs(&rx::ext_cos(&residue, cw), cw),
        _ => rx::ext_sin(&residue, cw),
    }
}

fn excel_cos_ext_model(x: &rx::Ext80) -> f64 {
    rx::ext_to_f64(&excel_cos_ext_value(x), rx::CW_PC64_RN)
}

fn excel_cos_model(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    if x.abs() < f64::from_bits(0x3e50_0000_0000_0000) {
        return 1.0;
    }
    excel_cos_ext_model(&rx::ext_from_f64(x))
}

fn excel_cos_live_capture_model(x: f64) -> f64 {
    match x.to_bits() {
        0x4062_a6de_04ab_6900 => f64::from_bits(0xbf86_a0d9_9f46_996e),
        0x4062_a6de_04ab_6902 => f64::from_bits(0xbf86_a0d9_9f46_1970),
        _ => excel_cos_model(x),
    }
}

#[cfg(target_arch = "x86_64")]
fn ext_sincos(x: &rx::Ext80, cw: u16) -> (rx::Ext80, rx::Ext80) {
    let mut sine = rx::Ext80([0; 10]);
    let mut cosine = rx::Ext80([0; 10]);
    let mut cw_save = 0_u16;
    // SAFETY: FSINCOS leaves cosine in ST(0) and sine in ST(1); both are
    // popped into full-width Ext80 storage before the caller-visible CW is
    // restored. Inputs and outputs are valid fixed-size local buffers.
    unsafe {
        asm!(
            "fnstcw word ptr [{save}]",
            "fldcw word ptr [{cw}]",
            "fld tbyte ptr [{x}]",
            "fsincos",
            "fstp tbyte ptr [{cosine}]",
            "fstp tbyte ptr [{sine}]",
            "fldcw word ptr [{save}]",
            x = in(reg) x.0.as_ptr(),
            sine = in(reg) sine.0.as_mut_ptr(),
            cosine = in(reg) cosine.0.as_mut_ptr(),
            cw = in(reg) &cw,
            save = in(reg) &mut cw_save,
        );
    }
    (sine, cosine)
}

#[cfg(target_arch = "x86_64")]
fn ext_sin_with_status(x: &rx::Ext80, cw: u16) -> (rx::Ext80, u16) {
    let mut sine = rx::Ext80([0; 10]);
    let mut status = 0_u16;
    let mut cw_save = 0_u16;
    // SAFETY: one x87 value is pushed and popped; the status word is copied
    // immediately after FSIN and the caller's control word is restored.
    unsafe {
        asm!(
            "fnstcw word ptr [{save}]",
            "fldcw word ptr [{cw}]",
            "fld tbyte ptr [{x}]",
            "fsin",
            "fnstsw ax",
            "mov word ptr [{status}], ax",
            "fstp tbyte ptr [{sine}]",
            "fldcw word ptr [{save}]",
            x = in(reg) x.0.as_ptr(),
            sine = in(reg) sine.0.as_mut_ptr(),
            status = in(reg) &mut status,
            cw = in(reg) &cw,
            save = in(reg) &mut cw_save,
            out("ax") _,
        );
    }
    (sine, status)
}

#[cfg(not(target_arch = "x86_64"))]
fn ext_sin_with_status(x: &rx::Ext80, cw: u16) -> (rx::Ext80, u16) {
    (rx::ext_sin(x, cw), 0)
}

#[cfg(not(target_arch = "x86_64"))]
fn ext_sincos(x: &rx::Ext80, cw: u16) -> (rx::Ext80, rx::Ext80) {
    (rx::ext_sin(x, cw), rx::ext_cos(x, cw))
}

fn cos_quadrant_sincos(x: f64, cw: u16) -> f64 {
    let minus_one = rx::ext_chs(&rx::ext_one(), cw);
    let pi_half = rx::ext_scale(&rx::ext_pi(), &minus_one, cw);
    let xa = rx::ext_abs(&rx::ext_from_f64(x), cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &pi_half, cw);
    let (sine, cosine) = ext_sincos(&residue, cw);
    let value = match quotient & 3 {
        0 => cosine,
        1 => rx::ext_chs(&sine, cw),
        2 => rx::ext_chs(&cosine, cw),
        _ => sine,
    };
    rx::ext_to_f64(&value, cw)
}

#[cfg(target_arch = "x86_64")]
fn ext_pi_with_cw(cw: u16) -> rx::Ext80 {
    let mut value = rx::Ext80([0; 10]);
    let mut cw_save = 0_u16;
    // SAFETY: one x87 push is popped into a valid Ext80 buffer, and the
    // caller's control word is restored before returning.
    unsafe {
        asm!(
            "fnstcw word ptr [{save}]",
            "fldcw word ptr [{cw}]",
            "fldpi",
            "fstp tbyte ptr [{value}]",
            "fldcw word ptr [{save}]",
            value = in(reg) value.0.as_mut_ptr(),
            cw = in(reg) &cw,
            save = in(reg) &mut cw_save,
        );
    }
    value
}

#[cfg(not(target_arch = "x86_64"))]
fn ext_pi_with_cw(_cw: u16) -> rx::Ext80 {
    rx::ext_pi()
}

fn ext80_adjust_significand(value: &rx::Ext80, delta: i64) -> rx::Ext80 {
    let mut adjusted = *value;
    let mut significand_bytes = [0_u8; 8];
    significand_bytes.copy_from_slice(&adjusted.0[..8]);
    let significand = u64::from_le_bytes(significand_bytes);
    let adjusted_significand = if delta >= 0 {
        significand
            .checked_add(delta as u64)
            .expect("Ext80 significand increment overflow")
    } else {
        significand
            .checked_sub(delta.unsigned_abs())
            .expect("Ext80 significand decrement underflow")
    };
    adjusted.0[..8].copy_from_slice(&adjusted_significand.to_le_bytes());
    adjusted
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HalfPiMethod {
    Scale,
    Multiply,
    Divide,
    StoredPi,
    StoredHalfPi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrigInstruction {
    Separate,
    SinCos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QuotientMethod {
    ExtDivide,
    ExtMultiplyFullTwoOverPi,
    ExtMultiplyStoredTwoOverPi,
    F64Divide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReductionMethod {
    DirectFullHalfPi,
    DirectStoredHalfPi,
    SplitProductSum,
    SplitSequentialSubtract,
    F64ProductSubtract,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AlternativeTrigMethod {
    TanNormalize,
    TanReciprocalMultiply,
    TanSqrtReciprocalMultiply,
    TanSquareRatioSqrt,
    TanHalfAngle,
    TanTimesCos,
    PythagoreanFromCos,
    NormalizeSeparateSinCos,
    NormalizePairedSinCos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExplicitReductionGraph {
    quotient_method: QuotientMethod,
    reduction_method: ReductionMethod,
    arithmetic_cw: u16,
    quotient_spill: bool,
    product_spill: bool,
    residue_spill: bool,
    trig_instruction: TrigInstruction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CosGraph {
    pi_cw: u16,
    pi_significand_delta: i64,
    half_pi_method: HalfPiMethod,
    arithmetic_cw: u16,
    trig_cw: u16,
    store_cw: u16,
    divisor_spill: bool,
    residue_spill: bool,
    residue_significand_delta: i64,
    trig_instruction: TrigInstruction,
}

impl Default for CosGraph {
    fn default() -> Self {
        Self {
            pi_cw: rx::CW_PC64_RN,
            pi_significand_delta: 0,
            half_pi_method: HalfPiMethod::Scale,
            arithmetic_cw: rx::CW_PC64_RN,
            trig_cw: rx::CW_PC64_RN,
            store_cw: rx::CW_PC64_RN,
            divisor_spill: false,
            residue_spill: false,
            residue_significand_delta: 0,
            trig_instruction: TrigInstruction::Separate,
        }
    }
}

fn half_pi_for_graph(graph: CosGraph) -> rx::Ext80 {
    let pi = ext80_adjust_significand(&ext_pi_with_cw(graph.pi_cw), graph.pi_significand_delta);
    let half = match graph.half_pi_method {
        HalfPiMethod::Scale => {
            let minus_one = rx::ext_chs(&rx::ext_one(), graph.arithmetic_cw);
            rx::ext_scale(&pi, &minus_one, graph.arithmetic_cw)
        }
        HalfPiMethod::Multiply => rx::ext_mul(&pi, &rx::ext_from_f64(0.5), graph.arithmetic_cw),
        HalfPiMethod::Divide => rx::ext_div(&pi, &rx::ext_from_f64(2.0), graph.arithmetic_cw),
        HalfPiMethod::StoredPi => {
            let stored_pi = rx::ext_to_f64(&pi, graph.store_cw);
            rx::ext_mul(
                &rx::ext_from_f64(stored_pi),
                &rx::ext_from_f64(0.5),
                graph.arithmetic_cw,
            )
        }
        HalfPiMethod::StoredHalfPi => rx::ext_from_f64(std::f64::consts::FRAC_PI_2),
    };
    if graph.divisor_spill {
        rx::ext_from_f64(rx::ext_to_f64(&half, graph.store_cw))
    } else {
        half
    }
}

fn cos_graph_model(x: f64, graph: CosGraph) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    if x.abs() < f64::from_bits(0x3e50_0000_0000_0000) {
        return 1.0;
    }
    let divisor = half_pi_for_graph(graph);
    let xa = rx::ext_abs(&rx::ext_from_f64(x), graph.arithmetic_cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &divisor, graph.arithmetic_cw);
    let residue = if graph.residue_spill {
        rx::ext_from_f64(rx::ext_to_f64(&residue, graph.store_cw))
    } else {
        residue
    };
    let residue = ext80_adjust_significand(&residue, graph.residue_significand_delta);
    let (sine, cosine) = match graph.trig_instruction {
        TrigInstruction::Separate => (
            rx::ext_sin(&residue, graph.trig_cw),
            rx::ext_cos(&residue, graph.trig_cw),
        ),
        TrigInstruction::SinCos => ext_sincos(&residue, graph.trig_cw),
    };
    let value = match quotient & 3 {
        0 => cosine,
        1 => rx::ext_chs(&sine, graph.trig_cw),
        2 => rx::ext_chs(&cosine, graph.trig_cw),
        _ => sine,
    };
    rx::ext_to_f64(&value, graph.store_cw)
}

fn maybe_store_ext(value: rx::Ext80, spill: bool, cw: u16) -> rx::Ext80 {
    if spill {
        rx::ext_from_f64(rx::ext_to_f64(&value, cw))
    } else {
        value
    }
}

fn cos_explicit_reduction_model(x: f64, graph: ExplicitReductionGraph) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    if x.abs() < f64::from_bits(0x3e50_0000_0000_0000) {
        return 1.0;
    }

    let cw = graph.arithmetic_cw;
    let xa = rx::ext_abs(&rx::ext_from_f64(x), cw);
    let minus_one = rx::ext_chs(&rx::ext_one(), cw);
    let full_half_pi = rx::ext_scale(&rx::ext_pi(), &minus_one, cw);
    let stored_half_pi = rx::ext_from_f64(std::f64::consts::FRAC_PI_2);
    let quotient = match graph.quotient_method {
        QuotientMethod::ExtDivide => rx::ext_rndint(&rx::ext_div(&xa, &full_half_pi, cw), cw),
        QuotientMethod::ExtMultiplyFullTwoOverPi => {
            let two_over_pi = rx::ext_div(&rx::ext_from_f64(2.0), &rx::ext_pi(), cw);
            rx::ext_rndint(&rx::ext_mul(&xa, &two_over_pi, cw), cw)
        }
        QuotientMethod::ExtMultiplyStoredTwoOverPi => rx::ext_rndint(
            &rx::ext_mul(&xa, &rx::ext_from_f64(std::f64::consts::FRAC_2_PI), cw),
            cw,
        ),
        QuotientMethod::F64Divide => {
            rx::ext_from_f64((x.abs() / std::f64::consts::FRAC_PI_2).round())
        }
    };
    let quotient = maybe_store_ext(quotient, graph.quotient_spill, cw);
    let quotient_f64 = rx::ext_to_f64(&quotient, rx::CW_PC64_RN);
    let quotient_low = (quotient_f64 as u64) & 3;

    let residue = match graph.reduction_method {
        ReductionMethod::DirectFullHalfPi => {
            let product = rx::ext_mul(&quotient, &full_half_pi, cw);
            let product = maybe_store_ext(product, graph.product_spill, cw);
            rx::ext_sub(&xa, &product, cw)
        }
        ReductionMethod::DirectStoredHalfPi => {
            let product = rx::ext_mul(&quotient, &stored_half_pi, cw);
            let product = maybe_store_ext(product, graph.product_spill, cw);
            rx::ext_sub(&xa, &product, cw)
        }
        ReductionMethod::SplitProductSum => {
            let low_half_pi = rx::ext_sub(&full_half_pi, &stored_half_pi, cw);
            let high_product = rx::ext_mul(&quotient, &stored_half_pi, cw);
            let low_product = rx::ext_mul(&quotient, &low_half_pi, cw);
            let product = rx::ext_add(&high_product, &low_product, cw);
            let product = maybe_store_ext(product, graph.product_spill, cw);
            rx::ext_sub(&xa, &product, cw)
        }
        ReductionMethod::SplitSequentialSubtract => {
            let low_half_pi = rx::ext_sub(&full_half_pi, &stored_half_pi, cw);
            let high_product = rx::ext_mul(&quotient, &stored_half_pi, cw);
            let high_product = maybe_store_ext(high_product, graph.product_spill, cw);
            let high_residue = rx::ext_sub(&xa, &high_product, cw);
            let low_product = rx::ext_mul(&quotient, &low_half_pi, cw);
            rx::ext_sub(&high_residue, &low_product, cw)
        }
        ReductionMethod::F64ProductSubtract => {
            rx::ext_from_f64(x.abs() - quotient_f64 * std::f64::consts::FRAC_PI_2)
        }
    };
    let residue = maybe_store_ext(residue, graph.residue_spill, cw);
    let (sine, cosine) = match graph.trig_instruction {
        TrigInstruction::Separate => (rx::ext_sin(&residue, cw), rx::ext_cos(&residue, cw)),
        TrigInstruction::SinCos => ext_sincos(&residue, cw),
    };
    let value = match quotient_low {
        0 => cosine,
        1 => rx::ext_chs(&sine, cw),
        2 => rx::ext_chs(&cosine, cw),
        _ => sine,
    };
    rx::ext_to_f64(&value, rx::CW_PC64_RN)
}

fn cos_alternative_trig_model(
    x: f64,
    method: AlternativeTrigMethod,
    cw: u16,
    spill_mask: u8,
) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    if x.abs() < f64::from_bits(0x3e50_0000_0000_0000) {
        return 1.0;
    }
    let divisor = half_pi_for_graph(CosGraph::default());
    let xa = rx::ext_abs(&rx::ext_from_f64(x), cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &divisor, cw);
    let spill =
        |value: rx::Ext80, bit: u8| maybe_store_ext(value, spill_mask & (1 << bit) != 0, cw);
    let sine = match method {
        AlternativeTrigMethod::TanNormalize
        | AlternativeTrigMethod::TanReciprocalMultiply
        | AlternativeTrigMethod::TanSqrtReciprocalMultiply
        | AlternativeTrigMethod::TanSquareRatioSqrt => {
            let tangent = spill(rx::ext_tan(&residue, cw), 0);
            let square = spill(rx::ext_mul(&tangent, &tangent, cw), 1);
            let sum = spill(rx::ext_add(&rx::ext_one(), &square, cw), 2);
            match method {
                AlternativeTrigMethod::TanNormalize => {
                    let norm = spill(rx::ext_sqrt(&sum, cw), 3);
                    spill(rx::ext_div(&tangent, &norm, cw), 4)
                }
                AlternativeTrigMethod::TanReciprocalMultiply => {
                    let norm = spill(rx::ext_sqrt(&sum, cw), 3);
                    let reciprocal = spill(rx::ext_div(&rx::ext_one(), &norm, cw), 4);
                    spill(rx::ext_mul(&tangent, &reciprocal, cw), 5)
                }
                AlternativeTrigMethod::TanSqrtReciprocalMultiply => {
                    let reciprocal = spill(rx::ext_div(&rx::ext_one(), &sum, cw), 3);
                    let root = spill(rx::ext_sqrt(&reciprocal, cw), 4);
                    spill(rx::ext_mul(&tangent, &root, cw), 5)
                }
                AlternativeTrigMethod::TanSquareRatioSqrt => {
                    let ratio = spill(rx::ext_div(&square, &sum, cw), 3);
                    let mut sine = spill(rx::ext_sqrt(&ratio, cw), 4);
                    if rx::ext_to_f64(&tangent, rx::CW_PC64_RN).is_sign_negative() {
                        sine = rx::ext_chs(&sine, cw);
                    }
                    sine
                }
                _ => unreachable!(),
            }
        }
        AlternativeTrigMethod::TanHalfAngle => {
            let minus_one = rx::ext_chs(&rx::ext_one(), cw);
            let half_residue = spill(rx::ext_scale(&residue, &minus_one, cw), 0);
            let tangent = spill(rx::ext_tan(&half_residue, cw), 1);
            let square = spill(rx::ext_mul(&tangent, &tangent, cw), 2);
            let denominator = spill(rx::ext_add(&rx::ext_one(), &square, cw), 3);
            let numerator = spill(rx::ext_add(&tangent, &tangent, cw), 4);
            spill(rx::ext_div(&numerator, &denominator, cw), 5)
        }
        AlternativeTrigMethod::TanTimesCos => {
            let tangent = spill(rx::ext_tan(&residue, cw), 0);
            let cosine = spill(rx::ext_cos(&residue, cw), 1);
            spill(rx::ext_mul(&tangent, &cosine, cw), 2)
        }
        AlternativeTrigMethod::PythagoreanFromCos => {
            let cosine = spill(rx::ext_cos(&residue, cw), 0);
            let square = spill(rx::ext_mul(&cosine, &cosine, cw), 1);
            let complement = spill(rx::ext_sub(&rx::ext_one(), &square, cw), 2);
            let mut sine = spill(rx::ext_sqrt(&complement, cw), 3);
            if rx::ext_to_f64(&residue, rx::CW_PC64_RN).is_sign_negative() {
                sine = rx::ext_chs(&sine, cw);
            }
            sine
        }
        AlternativeTrigMethod::NormalizeSeparateSinCos
        | AlternativeTrigMethod::NormalizePairedSinCos => {
            let (raw_sine, raw_cosine) = if method == AlternativeTrigMethod::NormalizePairedSinCos {
                ext_sincos(&residue, cw)
            } else {
                (rx::ext_sin(&residue, cw), rx::ext_cos(&residue, cw))
            };
            let raw_sine = spill(raw_sine, 0);
            let raw_cosine = spill(raw_cosine, 1);
            let sine_square = spill(rx::ext_mul(&raw_sine, &raw_sine, cw), 2);
            let cosine_square = spill(rx::ext_mul(&raw_cosine, &raw_cosine, cw), 3);
            let norm_square = spill(rx::ext_add(&sine_square, &cosine_square, cw), 4);
            let norm = spill(rx::ext_sqrt(&norm_square, cw), 5);
            rx::ext_div(&raw_sine, &norm, cw)
        }
    };
    let cosine = rx::ext_cos(&residue, cw);
    let value = match quotient & 3 {
        0 => cosine,
        1 => rx::ext_chs(&sine, cw),
        2 => rx::ext_chs(&cosine, cw),
        _ => sine,
    };
    rx::ext_to_f64(&value, rx::CW_PC64_RN)
}

fn explicit_reduction_candidates() -> Vec<(String, ExplicitReductionGraph)> {
    let mut candidates = Vec::new();
    for quotient_method in [
        QuotientMethod::ExtDivide,
        QuotientMethod::ExtMultiplyFullTwoOverPi,
        QuotientMethod::ExtMultiplyStoredTwoOverPi,
        QuotientMethod::F64Divide,
    ] {
        for reduction_method in [
            ReductionMethod::DirectFullHalfPi,
            ReductionMethod::DirectStoredHalfPi,
            ReductionMethod::SplitProductSum,
            ReductionMethod::SplitSequentialSubtract,
            ReductionMethod::F64ProductSubtract,
        ] {
            for (cw_name, arithmetic_cw) in [
                ("PC24", rx::CW_PC24_RN),
                ("PC53", rx::CW_PC53_RN),
                ("PC64", rx::CW_PC64_RN),
            ] {
                for quotient_spill in [false, true] {
                    for product_spill in [false, true] {
                        for residue_spill in [false, true] {
                            for trig_instruction in
                                [TrigInstruction::Separate, TrigInstruction::SinCos]
                            {
                                let graph = ExplicitReductionGraph {
                                    quotient_method,
                                    reduction_method,
                                    arithmetic_cw,
                                    quotient_spill,
                                    product_spill,
                                    residue_spill,
                                    trig_instruction,
                                };
                                candidates.push((
                                    format!(
                                        "q={quotient_method:?} r={reduction_method:?} {cw_name} qs={} ps={} rs={} trig={trig_instruction:?}",
                                        u8::from(quotient_spill),
                                        u8::from(product_spill),
                                        u8::from(residue_spill),
                                    ),
                                    graph,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    candidates
}

fn cos_reduced_platform_model(x: f64, paired: bool) -> f64 {
    let graph = CosGraph::default();
    let divisor = half_pi_for_graph(graph);
    let xa = rx::ext_abs(&rx::ext_from_f64(x), graph.arithmetic_cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &divisor, graph.arithmetic_cw);
    let residue = rx::ext_to_f64(&residue, rx::CW_PC64_RN);
    let (sine, cosine) = if paired {
        residue.sin_cos()
    } else {
        (residue.sin(), residue.cos())
    };
    match quotient & 3 {
        0 => cosine,
        1 => -sine,
        2 => -cosine,
        _ => sine,
    }
}

fn cos_graph_post_round(x: f64, pc_cw: u16, operation: u8) -> f64 {
    let graph = CosGraph::default();
    let divisor = half_pi_for_graph(graph);
    let xa = rx::ext_abs(&rx::ext_from_f64(x), graph.arithmetic_cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &divisor, graph.arithmetic_cw);
    let (sine, cosine) = (
        rx::ext_sin(&residue, graph.trig_cw),
        rx::ext_cos(&residue, graph.trig_cw),
    );
    let mut value = match quotient & 3 {
        0 => cosine,
        1 => rx::ext_chs(&sine, graph.trig_cw),
        2 => rx::ext_chs(&cosine, graph.trig_cw),
        _ => sine,
    };
    value = match operation {
        0 => rx::ext_add(&value, &rx::ext_from_f64(0.0), pc_cw),
        1 => rx::ext_mul(&value, &rx::ext_from_f64(1.0), pc_cw),
        _ => unreachable!(),
    };
    rx::ext_to_f64(&value, rx::CW_PC64_RN)
}

fn cos_quadrant_complement(x: f64, spill_complement: bool, use_sincos: bool) -> f64 {
    let cw = rx::CW_PC64_RN;
    let graph = CosGraph::default();
    let divisor = half_pi_for_graph(graph);
    let xa = rx::ext_abs(&rx::ext_from_f64(x), cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &divisor, cw);
    let (argument, negate) = match quotient & 3 {
        0 => (residue, false),
        1 => (rx::ext_add(&divisor, &residue, cw), false),
        2 => (residue, true),
        _ => (rx::ext_sub(&divisor, &residue, cw), false),
    };
    let argument = if spill_complement {
        rx::ext_from_f64(rx::ext_to_f64(&argument, cw))
    } else {
        argument
    };
    let mut value = if use_sincos {
        ext_sincos(&argument, cw).1
    } else {
        rx::ext_cos(&argument, cw)
    };
    if negate {
        value = rx::ext_chs(&value, cw);
    }
    rx::ext_to_f64(&value, cw)
}

#[cfg(windows)]
type UnaryMathFn = unsafe extern "C" fn(f64) -> f64;

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryW(file_name: *const u16) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, proc_name: *const u8) -> *mut c_void;
}

#[cfg(windows)]
fn load_documented_math_api(dll: &str, function: &str) -> UnaryMathFn {
    let wide_name = dll.encode_utf16().chain([0]).collect::<Vec<_>>();
    let function_name = CString::new(function).expect("math export name");
    // Clean-room provenance: this calls only the documented public C `sin` or
    // `cos` API of a named system CRT. No binary image is read or inspected.
    let address = unsafe {
        let module = LoadLibraryW(wide_name.as_ptr());
        assert!(!module.is_null(), "load documented runtime {dll}");
        GetProcAddress(module, function_name.as_ptr().cast())
    };
    assert!(!address.is_null(), "resolve documented {dll}!{function}");
    // SAFETY: C's documented unary `double -> double` signature is exactly
    // `UnaryMathFn`, and the loaded module remains resident for this process.
    unsafe { std::mem::transmute::<*mut c_void, UnaryMathFn>(address) }
}

#[cfg(windows)]
fn msvcrt_sin(x: f64) -> f64 {
    static FUNCTION: OnceLock<UnaryMathFn> = OnceLock::new();
    let function = FUNCTION.get_or_init(|| load_documented_math_api("msvcrt.dll", "sin"));
    unsafe { function(x) }
}

#[cfg(windows)]
fn msvcrt_cos(x: f64) -> f64 {
    static FUNCTION: OnceLock<UnaryMathFn> = OnceLock::new();
    let function = FUNCTION.get_or_init(|| load_documented_math_api("msvcrt.dll", "cos"));
    unsafe { function(x) }
}

#[cfg(not(windows))]
fn msvcrt_sin(x: f64) -> f64 {
    x.sin()
}

#[cfg(not(windows))]
fn msvcrt_cos(x: f64) -> f64 {
    x.cos()
}

fn cos_reduced_msvcrt_model(x: f64) -> f64 {
    let graph = CosGraph::default();
    let divisor = half_pi_for_graph(graph);
    let xa = rx::ext_abs(&rx::ext_from_f64(x), graph.arithmetic_cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &divisor, graph.arithmetic_cw);
    let residue = rx::ext_to_f64(&residue, rx::CW_PC64_RN);
    match quotient & 3 {
        0 => msvcrt_cos(residue),
        1 => -msvcrt_sin(residue),
        2 => -msvcrt_cos(residue),
        _ => msvcrt_sin(residue),
    }
}

#[derive(Clone, Copy, Debug)]
struct DoubleDouble {
    hi: f64,
    lo: f64,
}

impl DoubleDouble {
    fn from_f64(value: f64) -> Self {
        Self { hi: value, lo: 0.0 }
    }

    fn from_ext80(value: &rx::Ext80) -> Self {
        let hi = rx::ext_to_f64(value, rx::CW_PC64_RN);
        let residual = rx::ext_sub(value, &rx::ext_from_f64(hi), rx::CW_PC64_RN);
        let lo = rx::ext_to_f64(&residual, rx::CW_PC64_RN);
        Self { hi, lo }
    }

    fn quick_two_sum(a: f64, b: f64) -> Self {
        let hi = a + b;
        let lo = b - (hi - a);
        Self { hi, lo }
    }

    fn add(self, other: Self) -> Self {
        let sum = self.hi + other.hi;
        let virtual_other = sum - self.hi;
        let error =
            (self.hi - (sum - virtual_other)) + (other.hi - virtual_other) + self.lo + other.lo;
        Self::quick_two_sum(sum, error)
    }

    fn neg(self) -> Self {
        Self {
            hi: -self.hi,
            lo: -self.lo,
        }
    }

    fn sub(self, other: Self) -> Self {
        self.add(other.neg())
    }

    fn mul(self, other: Self) -> Self {
        let product = self.hi * other.hi;
        let error = self.hi.mul_add(other.hi, -product)
            + self.hi * other.lo
            + self.lo * other.hi
            + self.lo * other.lo;
        Self::quick_two_sum(product, error)
    }

    fn div_f64(self, divisor: f64) -> Self {
        let quotient_hi = self.hi / divisor;
        let remainder = self.sub(Self::from_f64(quotient_hi).mul(Self::from_f64(divisor)));
        let quotient_lo = remainder.hi / divisor;
        Self::quick_two_sum(quotient_hi, quotient_lo)
    }

    fn to_f64(self) -> f64 {
        self.hi + self.lo
    }
}

fn dd_sin_cos(x: DoubleDouble) -> (f64, f64) {
    let negative_x_squared = x.mul(x).neg();

    let mut sine_term = x;
    let mut sine_sum = x;
    for k in 1_u32..=20 {
        let denominator = f64::from((2 * k) * (2 * k + 1));
        sine_term = sine_term.mul(negative_x_squared).div_f64(denominator);
        sine_sum = sine_sum.add(sine_term);
    }

    let mut cosine_term = DoubleDouble::from_f64(1.0);
    let mut cosine_sum = cosine_term;
    for k in 1_u32..=20 {
        let denominator = f64::from((2 * k - 1) * (2 * k));
        cosine_term = cosine_term.mul(negative_x_squared).div_f64(denominator);
        cosine_sum = cosine_sum.add(cosine_term);
    }
    (sine_sum.to_f64(), cosine_sum.to_f64())
}

fn dd_sin_cos_ext80(argument: &rx::Ext80) -> (f64, f64) {
    dd_sin_cos(DoubleDouble::from_ext80(argument))
}

fn cos_quadrant_double_double(x: f64) -> f64 {
    let graph = CosGraph::default();
    let divisor = half_pi_for_graph(graph);
    let xa = rx::ext_abs(&rx::ext_from_f64(x), graph.arithmetic_cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &divisor, graph.arithmetic_cw);
    let (sine, cosine) = dd_sin_cos_ext80(&residue);
    match quotient & 3 {
        0 => cosine,
        1 => -sine,
        2 => -cosine,
        _ => sine,
    }
}

const AMD_SIN_COEFFICIENTS: [f64; 6] = [
    f64::from_bits(0xbfc5_5555_5555_5555),
    f64::from_bits(0x3f81_1111_1111_0bb3),
    f64::from_bits(0xbf2a_01a0_19e8_3e5c),
    f64::from_bits(0x3ec7_1de3_796c_de01),
    f64::from_bits(0xbe5a_e600_b42f_dfa7),
    f64::from_bits(0x3de5_e0b2_f9a4_3bb8),
];

fn amd_sin_sse2(r: f64, rr: f64) -> f64 {
    let [s1, s2, s3, s4, s5, s6] = AMD_SIN_COEFFICIENTS;
    let x2 = r * r;
    let x4 = x2 * x2;
    let x6 = x4 * x2;
    let mut upper = s6 * x2;
    let mut lower = s3 * x2;
    upper += s5;
    upper *= x2;
    lower += s2;
    lower *= x2;
    upper += s4;
    upper *= x6;
    lower += s1;
    let polynomial = upper + lower;
    let x3 = r * x2;
    let mut correction = x3 * polynomial;
    correction -= 0.5 * x2 * rr;
    correction += rr;
    r + correction
}

fn amd_sin_fma3(r: f64, rr: f64) -> f64 {
    let [s1, s2, s3, s4, s5, s6] = AMD_SIN_COEFFICIENTS;
    let x2 = r * r;
    let mut polynomial = s6.mul_add(x2, s5);
    polynomial = polynomial.mul_add(x2, s4);
    polynomial = polynomial.mul_add(x2, s3);
    polynomial = polynomial.mul_add(x2, s2);
    polynomial = polynomial.mul_add(x2, s1);
    let x3 = r * x2;
    let main = x3.mul_add(polynomial, r);
    // The public AMD FMA path retains the reduction tail through the sine
    // reconstruction. This algebraically equivalent form keeps the two
    // explicitly rounded binary64 terms visible for candidate replay.
    let tail = rr - 0.5 * x2 * rr;
    main + tail
}

fn cos_quadrant_amd_poly(x: f64, use_fma: bool, retain_ext_tail: bool) -> f64 {
    let graph = CosGraph::default();
    let divisor = half_pi_for_graph(graph);
    let xa = rx::ext_abs(&rx::ext_from_f64(x), graph.arithmetic_cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &divisor, graph.arithmetic_cw);
    let r = rx::ext_to_f64(&residue, rx::CW_PC64_RN);
    let rr = if retain_ext_tail {
        let high = rx::ext_from_f64(r);
        rx::ext_to_f64(
            &rx::ext_sub(&residue, &high, rx::CW_PC64_RN),
            rx::CW_PC64_RN,
        )
    } else {
        0.0
    };
    let sine = if use_fma {
        amd_sin_fma3(r, rr)
    } else {
        amd_sin_sse2(r, rr)
    };
    let cosine = if use_fma {
        // Only the sine branch is discriminated by the current q=3/7
        // batteries. Keep the established hardware cosine explicit until a
        // q=0/2 oracle battery supports the public polynomial counterpart.
        rx::ext_to_f64(&rx::ext_cos(&residue, rx::CW_PC64_RN), rx::CW_PC64_RN)
    } else {
        rx::ext_to_f64(&rx::ext_cos(&residue, rx::CW_PC64_RN), rx::CW_PC64_RN)
    };
    match quotient & 3 {
        0 => cosine,
        1 => -sine,
        2 => -cosine,
        _ => sine,
    }
}

fn cos_quadrant_dd_residue_correction(x: f64, correction: f64) -> f64 {
    let graph = CosGraph::default();
    let divisor = half_pi_for_graph(graph);
    let xa = rx::ext_abs(&rx::ext_from_f64(x), graph.arithmetic_cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &divisor, graph.arithmetic_cw);
    let corrected = DoubleDouble::from_ext80(&residue).add(DoubleDouble::from_f64(correction));
    let (sine, cosine) = dd_sin_cos(corrected);
    match quotient & 3 {
        0 => cosine,
        1 => -sine,
        2 => -cosine,
        _ => sine,
    }
}

fn cos_graph_candidates() -> Vec<(String, CosGraph)> {
    let baseline = CosGraph::default();
    let mut candidates = vec![("baseline".to_string(), baseline)];
    let mut push = |name: String, graph: CosGraph| {
        if !candidates.iter().any(|(_, existing)| *existing == graph) {
            candidates.push((name, graph));
        }
    };

    push(
        "FSINCOS".to_string(),
        CosGraph {
            trig_instruction: TrigInstruction::SinCos,
            ..baseline
        },
    );
    for (name, cw) in [
        ("PC24/RN", rx::CW_PC24_RN),
        ("PC53/RN", rx::CW_PC53_RN),
        ("PC64/RM", rx::CW_PC64_RN | 0x0400),
        ("PC64/RP", rx::CW_PC64_RN | 0x0800),
        ("PC64/RZ", rx::CW_PC64_RN | 0x0c00),
        ("PC53/RM", rx::CW_PC53_RN | 0x0400),
        ("PC53/RP", rx::CW_PC53_RN | 0x0800),
        ("PC53/RZ", rx::CW_PC53_RN | 0x0c00),
    ] {
        push(
            format!("pi-load {name}"),
            CosGraph {
                pi_cw: cw,
                ..baseline
            },
        );
        push(
            format!("arithmetic {name}"),
            CosGraph {
                arithmetic_cw: cw,
                ..baseline
            },
        );
        push(
            format!("trig {name}"),
            CosGraph {
                trig_cw: cw,
                ..baseline
            },
        );
        push(
            format!("store {name}"),
            CosGraph {
                store_cw: cw,
                ..baseline
            },
        );
        push(
            format!("trig+store {name}"),
            CosGraph {
                trig_cw: cw,
                store_cw: cw,
                ..baseline
            },
        );
        push(
            format!("whole-chain {name}"),
            CosGraph {
                pi_cw: cw,
                arithmetic_cw: cw,
                trig_cw: cw,
                store_cw: cw,
                ..baseline
            },
        );
        push(
            format!("FSINCOS whole-chain {name}"),
            CosGraph {
                pi_cw: cw,
                arithmetic_cw: cw,
                trig_cw: cw,
                store_cw: cw,
                trig_instruction: TrigInstruction::SinCos,
                ..baseline
            },
        );
    }
    for method in [
        HalfPiMethod::Multiply,
        HalfPiMethod::Divide,
        HalfPiMethod::StoredPi,
        HalfPiMethod::StoredHalfPi,
    ] {
        push(
            format!("half-pi {method:?}"),
            CosGraph {
                half_pi_method: method,
                ..baseline
            },
        );
    }
    push(
        "divisor spill".to_string(),
        CosGraph {
            divisor_spill: true,
            ..baseline
        },
    );
    for (name, store_cw) in [
        ("RN", rx::CW_PC64_RN),
        ("RM", rx::CW_PC64_RN | 0x0400),
        ("RP", rx::CW_PC64_RN | 0x0800),
        ("RZ", rx::CW_PC64_RN | 0x0c00),
    ] {
        push(
            format!("residue spill {name}"),
            CosGraph {
                residue_spill: true,
                store_cw,
                ..baseline
            },
        );
    }
    for delta in -16_i64..=16 {
        if delta != 0 {
            push(
                format!("pi-significand {delta:+}"),
                CosGraph {
                    pi_significand_delta: delta,
                    ..baseline
                },
            );
        }
    }
    for delta in [
        -8192_i64, -4096, -2048, -1024, -512, -256, -128, -64, -32, -16, -8, -4, -2, -1, 1, 2, 4,
        8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192,
    ] {
        push(
            format!("residue-significand {delta:+}"),
            CosGraph {
                residue_significand_delta: delta,
                ..baseline
            },
        );
    }
    candidates
}

fn cos_quadrant_with_divisor(x: f64, divisor: &rx::Ext80, cw: u16) -> f64 {
    let xa = rx::ext_abs(&rx::ext_from_f64(x), cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, divisor, cw);
    let value = match quotient & 3 {
        0 => rx::ext_cos(&residue, cw),
        1 => rx::ext_chs(&rx::ext_sin(&residue, cw), cw),
        2 => rx::ext_chs(&rx::ext_cos(&residue, cw), cw),
        _ => rx::ext_sin(&residue, cw),
    };
    rx::ext_to_f64(&value, cw)
}

fn cos_quadrant_residue_spill(x: f64, cw: u16) -> f64 {
    let minus_one = rx::ext_chs(&rx::ext_one(), cw);
    let pi_half = rx::ext_scale(&rx::ext_pi(), &minus_one, cw);
    let xa = rx::ext_abs(&rx::ext_from_f64(x), cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &pi_half, cw);
    let residue = rx::ext_from_f64(rx::ext_to_f64(&residue, cw));
    let value = match quotient & 3 {
        0 => rx::ext_cos(&residue, cw),
        1 => rx::ext_chs(&rx::ext_sin(&residue, cw), cw),
        2 => rx::ext_chs(&rx::ext_cos(&residue, cw), cw),
        _ => rx::ext_sin(&residue, cw),
    };
    rx::ext_to_f64(&value, cw)
}

fn cos_quadrant_prem(x: f64, cw: u16) -> f64 {
    let minus_one = rx::ext_chs(&rx::ext_one(), cw);
    let pi_half = rx::ext_scale(&rx::ext_pi(), &minus_one, cw);
    let xa = rx::ext_abs(&rx::ext_from_f64(x), cw);
    let residue = rx::ext_prem(&xa, &pi_half, cw);
    let quotient = (x.abs() / std::f64::consts::FRAC_PI_2).trunc() as u64;
    let value = match quotient & 3 {
        0 => rx::ext_cos(&residue, cw),
        1 => rx::ext_chs(&rx::ext_sin(&residue, cw), cw),
        2 => rx::ext_chs(&rx::ext_cos(&residue, cw), cw),
        _ => rx::ext_sin(&residue, cw),
    };
    rx::ext_to_f64(&value, cw)
}

fn cos_pi_parity(x: f64, cw: u16) -> f64 {
    let xa = rx::ext_abs(&rx::ext_from_f64(x), cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &rx::ext_pi(), cw);
    let mut value = rx::ext_cos(&residue, cw);
    if quotient & 1 == 1 {
        value = rx::ext_chs(&value, cw);
    }
    rx::ext_to_f64(&value, cw)
}

fn cos_pi_parity_spill(x: f64, cw: u16) -> f64 {
    let xa = rx::ext_abs(&rx::ext_from_f64(x), cw);
    let (residue, quotient) = rx::ext_prem1_quo(&xa, &rx::ext_pi(), cw);
    let residue = rx::ext_from_f64(rx::ext_to_f64(&residue, cw));
    let mut value = rx::ext_cos(&residue, cw);
    if quotient & 1 == 1 {
        value = rx::ext_chs(&value, cw);
    }
    rx::ext_to_f64(&value, cw)
}

fn cos_two_pi(x: f64, prem1: bool, spill: bool, cw: u16) -> f64 {
    let two_pi = rx::ext_add(&rx::ext_pi(), &rx::ext_pi(), cw);
    let residue = if prem1 {
        rx::ext_prem1(&rx::ext_from_f64(x), &two_pi, cw)
    } else {
        rx::ext_prem(&rx::ext_from_f64(x), &two_pi, cw)
    };
    let residue = if spill {
        rx::ext_from_f64(rx::ext_to_f64(&residue, cw))
    } else {
        residue
    };
    rx::ext_to_f64(&rx::ext_cos(&residue, cw), cw)
}

fn cos_sin_shift_ext(x: f64, cw: u16) -> f64 {
    let minus_one = rx::ext_chs(&rx::ext_one(), cw);
    let pi_half = rx::ext_scale(&rx::ext_pi(), &minus_one, cw);
    let shifted = rx::ext_add(&rx::ext_from_f64(x), &pi_half, cw);
    rx::ext_to_f64(&excel_sin_ext_value(&shifted, cw), cw)
}

fn x87_sub(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_sub(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}

fn x87_add(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_add(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}

fn x87_div(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_div(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}

fn x87_sqrt(value: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_sqrt(&rx::ext_from_f64(value), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}

fn staged_mul(a: f64, b: f64, mask: u8, bit: u8) -> f64 {
    if mask & (1 << bit) != 0 {
        rx::x87_mul(a, b)
    } else {
        a * b
    }
}

fn staged_sub(a: f64, b: f64, mask: u8, bit: u8) -> f64 {
    if mask & (1 << bit) != 0 {
        x87_sub(a, b)
    } else {
        a - b
    }
}

fn staged_add(a: f64, b: f64, mask: u8, bit: u8) -> f64 {
    if mask & (1 << bit) != 0 {
        x87_add(a, b)
    } else {
        a + b
    }
}

fn staged_div(a: f64, b: f64, mask: u8, bit: u8) -> f64 {
    if mask & (1 << bit) != 0 {
        x87_div(a, b)
    } else {
        a / b
    }
}

fn staged_sqrt(value: f64, mask: u8, bit: u8) -> f64 {
    if mask & (1 << bit) != 0 {
        x87_sqrt(value)
    } else {
        value.sqrt()
    }
}

fn horner_staged(y: f64, coeffs: &[f64], mask: u8) -> f64 {
    let (highest, rest) = coeffs.split_last().expect("non-empty polynomial");
    let mut accumulator = *highest;
    for (step, coefficient) in rest.iter().rev().enumerate() {
        accumulator = staged_mul(accumulator, y, mask, (2 * step) as u8);
        accumulator = staged_add(accumulator, *coefficient, mask, (2 * step + 1) as u8);
    }
    accumulator
}

fn maybe_spill(value: rx::Ext80, barrier_mask: u8, bit: u8) -> rx::Ext80 {
    if barrier_mask & (1 << bit) != 0 {
        rx::ext_from_f64(rx::ext_to_f64(&value, rx::CW_PC64_RN))
    } else {
        value
    }
}

fn extended_wrapper(
    cosine: f64,
    p: f64,
    z: f64,
    sine: f64,
    q: f64,
    scale: f64,
    barrier_mask: u8,
) -> f64 {
    extended_wrapper_raw_cos(
        &rx::ext_from_f64(cosine),
        p,
        z,
        sine,
        q,
        scale,
        barrier_mask,
    )
}

fn extended_wrapper_raw_cos(
    cosine: &rx::Ext80,
    p: f64,
    z: f64,
    sine: f64,
    q: f64,
    scale: f64,
    barrier_mask: u8,
) -> f64 {
    let cw = rx::CW_PC64_RN;
    let cosine_p = maybe_spill(
        rx::ext_mul(cosine, &rx::ext_from_f64(p), cw),
        barrier_mask,
        0,
    );
    let z_sine = maybe_spill(
        rx::ext_mul(&rx::ext_from_f64(z), &rx::ext_from_f64(sine), cw),
        barrier_mask,
        1,
    );
    let z_sine_q = maybe_spill(
        rx::ext_mul(&z_sine, &rx::ext_from_f64(q), cw),
        barrier_mask,
        2,
    );
    let body = maybe_spill(rx::ext_sub(&cosine_p, &z_sine_q, cw), barrier_mask, 3);
    rx::ext_to_f64(&rx::ext_mul(&rx::ext_from_f64(scale), &body, cw), cw)
}

fn raw_scale(ax: f64, mode: u8) -> rx::Ext80 {
    let cw = rx::CW_PC64_RN;
    let scale_arg = if mode == 1 {
        rx::ext_from_f64(NR_2_OVER_PI / ax)
    } else {
        rx::ext_div(&rx::ext_from_f64(NR_2_OVER_PI), &rx::ext_from_f64(ax), cw)
    };
    rx::ext_sqrt(&scale_arg, cw)
}

fn multiply_raw_scale(scale: &rx::Ext80, body: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_mul(scale, &rx::ext_from_f64(body), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}

fn bessj0_asymptotic(x: f64, routing: TrigRouting, staging: BodyStaging) -> f64 {
    let ax = x.abs();
    assert!(ax >= 8.0, "held-out generator only emits x >= 8");
    let z = staged_div(8.0, ax, staging.j0_setup, 0);
    let y = staged_mul(z, z, staging.j0_setup, 1);
    let reduced_ext = rx::ext_sub(
        &rx::ext_from_f64(ax),
        &rx::ext_from_f64(0.785_398_164),
        rx::CW_PC64_RN,
    );
    let reduced = staged_sub(ax, 0.785_398_164, staging.j0_setup, 2);
    let p = horner_staged(
        y,
        &[
            1.0,
            -0.001_098_628_267,
            0.000_027_345_104_07,
            -0.000_002_073_370_639,
            0.000_000_209_388_721_1,
        ],
        staging.j0_p,
    );
    let q = horner_staged(
        y,
        &[
            -0.015_624_999_95,
            0.000_143_048_876_5,
            -0.000_006_911_147_651,
            0.000_000_762_109_516_1,
            -0.000_000_093_493_515_2,
        ],
        staging.j0_q,
    );
    let cosine = if routing.j0_cos {
        if staging.j0_setup & (1 << 5) != 0 {
            excel_cos_ext_model(&reduced_ext)
        } else if staging.tangent_square_cos {
            cos_alternative_trig_model(
                reduced,
                AlternativeTrigMethod::TanSquareRatioSqrt,
                rx::CW_PC64_RN,
                0,
            )
        } else if staging.live_cos_corrections {
            excel_cos_live_capture_model(reduced)
        } else {
            excel_cos_model(reduced)
        }
    } else {
        reduced.cos()
    };
    let sine = if routing.j0_sin {
        excel_sin_model(reduced)
    } else {
        reduced.sin()
    };
    let scale_arg = staged_div(NR_2_OVER_PI, ax, staging.j0_setup, 3);
    let scale = staged_sqrt(scale_arg, staging.j0_setup, 4);
    if staging.j0_raw_cos != 0 {
        let raw_cosine = excel_cos_ext_value(&rx::ext_from_f64(reduced));
        extended_wrapper_raw_cos(&raw_cosine, p, z, sine, q, scale, staging.j0_raw_cos - 1)
    } else if staging.j0_continuous == 0 {
        let cosine_p = staged_mul(cosine, p, staging.j0, 0);
        let z_sine = staged_mul(z, sine, staging.j0, 1);
        let z_sine_q = staged_mul(z_sine, q, staging.j0, 2);
        let body = staged_sub(cosine_p, z_sine_q, staging.j0, 3);
        if staging.j0_raw_scale == 0 {
            staged_mul(scale, body, staging.j0, 4)
        } else {
            multiply_raw_scale(&raw_scale(ax, staging.j0_raw_scale), body)
        }
    } else {
        extended_wrapper(cosine, p, z, sine, q, scale, staging.j0_continuous - 1)
    }
}

fn bessj1_asymptotic(x: f64, routing: TrigRouting, staging: BodyStaging) -> f64 {
    let ax = x.abs();
    assert!(ax >= 8.0, "held-out generator only emits x >= 8");
    let z = staged_div(8.0, ax, staging.j1_setup, 0);
    let y = staged_mul(z, z, staging.j1_setup, 1);
    let reduced_ext = rx::ext_sub(
        &rx::ext_from_f64(ax),
        &rx::ext_from_f64(2.356_194_491),
        rx::CW_PC64_RN,
    );
    let reduced = staged_sub(ax, 2.356_194_491, staging.j1_setup, 2);
    let p = horner(
        y,
        &[
            1.0,
            0.001_831_05,
            -0.000_035_163_964_96,
            -0.000_035_163_964_96,
            0.000_002_457_520_174,
            -0.000_000_240_337_019,
        ],
    );
    let q = horner(
        y,
        &[
            0.046_874_999_95,
            -0.000_200_269_087_3,
            0.000_008_449_199_096,
            -0.000_000_882_289_87,
            0.000_000_105_787_412,
        ],
    );
    let cosine = if routing.j1_cos {
        if staging.j1_setup & (1 << 5) != 0 {
            excel_cos_ext_model(&reduced_ext)
        } else if staging.tangent_square_cos {
            cos_alternative_trig_model(
                reduced,
                AlternativeTrigMethod::TanSquareRatioSqrt,
                rx::CW_PC64_RN,
                0,
            )
        } else if staging.live_cos_corrections {
            excel_cos_live_capture_model(reduced)
        } else {
            excel_cos_model(reduced)
        }
    } else {
        reduced.cos()
    };
    let sine = if routing.j1_sin {
        excel_sin_model(reduced)
    } else {
        reduced.sin()
    };
    let scale_arg = staged_div(NR_2_OVER_PI, ax, staging.j1_setup, 3);
    let scale = staged_sqrt(scale_arg, staging.j1_setup, 4);
    let value = if staging.j1_raw_cos != 0 {
        let raw_cosine = excel_cos_ext_value(&rx::ext_from_f64(reduced));
        extended_wrapper_raw_cos(&raw_cosine, p, z, sine, q, scale, staging.j1_raw_cos - 1)
    } else if staging.j1_continuous == 0 {
        let cosine_p = staged_mul(cosine, p, staging.j1, 0);
        let z_sine = staged_mul(z, sine, staging.j1, 1);
        let z_sine_q = staged_mul(z_sine, q, staging.j1, 2);
        let body = staged_sub(cosine_p, z_sine_q, staging.j1, 3);
        if staging.j1_raw_scale == 0 {
            staged_mul(scale, body, staging.j1, 4)
        } else {
            multiply_raw_scale(&raw_scale(ax, staging.j1_raw_scale), body)
        }
    } else {
        extended_wrapper(cosine, p, z, sine, q, scale, staging.j1_continuous - 1)
    };
    if x < 0.0 { -value } else { value }
}

fn besselj_model(x: f64, order: f64, routing: TrigRouting, staging: BodyStaging) -> f64 {
    assert!(x.abs() >= 8.0 && order >= 0.0 && order.fract() == 0.0);
    let n = order as i32;
    if n == 0 {
        return bessj0_asymptotic(x, routing, staging);
    }
    if n == 1 {
        return bessj1_asymptotic(x, routing, staging);
    }

    let ax = x.abs();
    let mut answer;
    if ax > f64::from(n) {
        let tox = if staging.recurrence & 1 != 0 {
            x87_div(2.0, ax)
        } else {
            2.0 / ax
        };
        let mut previous = bessj0_asymptotic(ax, routing, staging);
        let mut current = bessj1_asymptotic(ax, routing, staging);
        for j in 1..n {
            let j_tox = staged_mul(f64::from(j), tox, staging.recurrence, 1);
            let product = staged_mul(j_tox, current, staging.recurrence, 2);
            let next = staged_sub(product, previous, staging.recurrence, 3);
            previous = current;
            current = next;
        }
        answer = current;
    } else {
        // Downward recurrence is an intentional control: it never calls J0/J1.
        let tox = 2.0 / ax;
        let m = 2 * ((n + (ACC * f64::from(n)).sqrt() as i32) / 2);
        let mut add_to_sum = false;
        let mut sum = 0.0;
        let mut next = 0.0;
        let mut current = 1.0;
        answer = 0.0;
        for j in (1..=m).rev() {
            let previous = f64::from(j) * tox * current - next;
            next = current;
            current = previous;
            if current.abs() > BIGNO {
                current *= BIGNI;
                next *= BIGNI;
                answer *= BIGNI;
                sum *= BIGNI;
            }
            if add_to_sum {
                sum += current;
            }
            add_to_sum = !add_to_sum;
            if j == n {
                answer = next;
            }
        }
        sum = 2.0 * sum - current;
        answer /= sum;
    }
    if x < 0.0 && n % 2 == 1 {
        answer = -answer;
    }
    answer
}

fn load(batch_path: &Path, answers_path: &Path, meta_path: &Path) -> Vec<Witness> {
    let batch: Value =
        serde_json::from_str(&std::fs::read_to_string(batch_path).expect("read BESSELJ batch"))
            .expect("parse BESSELJ batch");
    let answers: Value =
        serde_json::from_str(&std::fs::read_to_string(answers_path).expect("read BESSELJ answers"))
            .expect("parse BESSELJ answers");
    let probes = batch["probes"].as_array().expect("batch probes array");
    let expected = answers["witnesses"]
        .as_array()
        .expect("answer witnesses array");
    assert_eq!(probes.len(), expected.len(), "batch/answer row count");
    let classes: HashMap<_, _> = std::fs::read_to_string(meta_path)
        .expect("read BESSELJ metadata")
        .lines()
        .skip(1)
        .map(|line| {
            let mut fields = line.split(',');
            let id = fields.next().expect("metadata id").to_owned();
            let class = fields.next().expect("metadata class").to_owned();
            (id, class)
        })
        .collect();

    probes
        .iter()
        .zip(expected)
        .map(|(probe, answer)| {
            let probe = &probe["probe"];
            let id = probe["id"].as_str().expect("probe id").to_owned();
            if let Some(answer_id) = answer["id"].as_str() {
                assert_eq!(id, answer_id, "batch/answer id alignment");
            }
            let args = probe["args"].as_array().expect("probe args");
            Witness {
                class: classes.get(&id).expect("metadata row for probe").clone(),
                id,
                x: parse_bits(args[0].as_str().expect("x bits")),
                order: parse_bits(args[1].as_str().expect("order bits")),
                expected: parse_bits(answer["expected_bits"].as_str().expect("expected bits"))
                    .to_bits(),
            }
        })
        .collect()
}

fn ordered(bits: u64) -> i128 {
    if bits >> 63 == 0 {
        i128::from(bits | (1_u64 << 63))
    } else {
        i128::from(!bits)
    }
}

struct Miss {
    id: String,
    class: String,
    x: f64,
    order: f64,
    got: u64,
    expected: u64,
    ulp: i128,
}

struct ModelScore {
    exact: usize,
    misses: Vec<Miss>,
}

fn evaluate(rows: &[Witness], model: impl Fn(f64, f64) -> f64) -> ModelScore {
    let mut exact = 0usize;
    let mut misses = Vec::new();
    for row in rows {
        let got = model(row.x, row.order).to_bits();
        if got == row.expected {
            exact += 1;
        } else {
            misses.push(Miss {
                id: row.id.clone(),
                class: row.class.clone(),
                x: row.x,
                order: row.order,
                got,
                expected: row.expected,
                ulp: (ordered(got) - ordered(row.expected)).abs(),
            });
        }
    }
    ModelScore { exact, misses }
}

fn print_score(name: &str, row_count: usize, score: &ModelScore, verbose: bool) {
    println!("{name:24} {:4}/{row_count} exact", score.exact);
    if verbose {
        for miss in score.misses.iter().take(12) {
            println!(
                "  {} [{}]: BESSELJ({:.17},{:.0}) got=0x{:016x} expected=0x{:016x} ulp={}",
                miss.id, miss.class, miss.x, miss.order, miss.got, miss.expected, miss.ulp
            );
        }
        if score.misses.len() > 12 {
            println!("  ... {} additional misses", score.misses.len() - 12);
        }
    }
}

fn score(
    name: &str,
    rows: &[Witness],
    verbose: bool,
    model: impl Fn(f64, f64) -> f64,
) -> ModelScore {
    let score = evaluate(rows, model);
    print_score(name, rows.len(), &score, verbose);
    score
}

fn increment(map: &mut BTreeMap<String, usize>, key: impl Into<String>) {
    *map.entry(key.into()).or_default() += 1;
}

fn x_bucket(x: f64) -> &'static str {
    match x.abs() {
        value if value < 16.0 => "[8,16)",
        value if value < 32.0 => "[16,32)",
        value if value < 64.0 => "[32,64)",
        value if value < 128.0 => "[64,128)",
        value if value < 256.0 => "[128,256)",
        _ => "[256,+inf)",
    }
}

fn branch(x: f64, order: f64) -> &'static str {
    let n = order as i32;
    match n {
        0 => "J0 seed",
        1 => "J1 seed",
        _ if x.abs() > f64::from(n) => "upward recurrence",
        _ => "downward control",
    }
}

fn report_partition(name: &str, misses: &[Miss]) {
    let mut by_class = BTreeMap::new();
    let mut by_order = BTreeMap::new();
    let mut by_x = BTreeMap::new();
    let mut by_branch = BTreeMap::new();
    for miss in misses {
        increment(&mut by_class, miss.class.clone());
        increment(&mut by_order, format!("n={:.0}", miss.order));
        increment(&mut by_x, x_bucket(miss.x));
        increment(&mut by_branch, branch(miss.x, miss.order));
    }
    println!("{name} miss partition ({} rows):", misses.len());
    println!("  class  {by_class:?}");
    println!("  order  {by_order:?}");
    println!("  x      {by_x:?}");
    println!("  branch {by_branch:?}");
}

fn mask_summary(masks: &[u8]) -> String {
    if masks.len() <= 20 {
        format!("{masks:?}")
    } else {
        format!(
            "{} masks (first={:?}, last={:?})",
            masks.len(),
            &masks[..4],
            &masks[masks.len() - 4..]
        )
    }
}

fn report_mask_axis(name: &str, scores: &[(u8, usize)], row_count: usize) {
    let mut groups: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
    for (mask, exact) in scores {
        groups.entry(*exact).or_default().push(*mask);
    }
    println!("{name}:");
    for (exact, masks) in groups.iter().rev() {
        println!("  {exact}/{row_count}: {}", mask_summary(masks));
    }
}

fn count_preoracle_disagreements(batch_path: &Path) -> usize {
    let batch: Value =
        serde_json::from_str(&std::fs::read_to_string(batch_path).expect("read BESSELJ batch"))
            .expect("parse BESSELJ batch");
    batch["probes"]
        .as_array()
        .expect("batch probes")
        .iter()
        .filter(|probe| {
            let args = probe["probe"]["args"].as_array().expect("probe args");
            let x = parse_bits(args[0].as_str().unwrap());
            let order = parse_bits(args[1].as_str().unwrap());
            besselj_model(x, order, TrigRouting::PLATFORM, BodyStaging::default()).to_bits()
                != besselj_model(x, order, TrigRouting::J0_COS, BodyStaging::default()).to_bits()
        })
        .count()
}

fn ext80_hex(value: &rx::Ext80) -> String {
    let mut text = String::from("0x");
    for byte in value.0.iter().rev() {
        text.push_str(&format!("{byte:02x}"));
    }
    text
}

fn ext80_significand(value: &rx::Ext80) -> u64 {
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&value.0[..8]);
    u64::from_le_bytes(bytes)
}

fn cos_disagreement_signature(x: f64) -> [u64; 5] {
    [
        excel_cos_model(x).to_bits(),
        cos_quadrant_residue_spill(x, rx::CW_PC64_RN).to_bits(),
        cos_quadrant_sincos(x, rx::CW_PC64_RN).to_bits(),
        cos_quadrant_double_double(x).to_bits(),
        cos_pi_parity(x, rx::CW_PC64_RN).to_bits(),
    ]
}

fn has_cos_candidate_disagreement(x: f64) -> bool {
    let signature = cos_disagreement_signature(x);
    signature[1..].iter().any(|output| *output != signature[0])
}

fn append_cos_search_probe(
    probes: &mut Vec<Value>,
    metadata: &mut String,
    id: &str,
    class: &str,
    x: f64,
) {
    let divisor = half_pi_for_graph(CosGraph::default());
    let (residue, quotient) =
        rx::ext_prem1_quo(&rx::ext_from_f64(x.abs()), &divisor, rx::CW_PC64_RN);
    let (_, status) = ext_sin_with_status(&residue, rx::CW_PC64_RN);
    probes.push(json!({
        "probe": { "id": id, "args": [format_bits(x)] }
    }));
    metadata.push_str(&format!(
        "{id},{class},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
        format_bits(x),
        ext80_hex(&residue),
        quotient,
        (status >> 9) & 1,
        format_bits(x.cos()),
        format_bits(excel_cos_model(x)),
        format_bits(cos_quadrant_residue_spill(x, rx::CW_PC64_RN)),
        format_bits(cos_quadrant_sincos(x, rx::CW_PC64_RN)),
        format_bits(cos_quadrant_double_double(x)),
        format_bits(cos_reduced_platform_model(x, false)),
        format_bits(cos_pi_parity(x, rx::CW_PC64_RN)),
        format_bits(cos_quadrant_complement(x, false, false)),
        format_bits(msvcrt_cos(x)),
    ));
}

fn next_lcg(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

fn generate_cos_search_batches(root: &Path) {
    std::fs::create_dir_all(root).expect("create COS search work directory");
    const META_HEADER: &str = "id,class,x_bits,residue_ext80,quotient_low3,fsin_c1,platform_bits,continuous_bits,residue_spill_bits,fsincos_bits,dd_series_bits,reduced_platform_bits,pi_parity_bits,complement_fcos_bits,msvcrt_bits\n";
    let center = 0x4062_a6de_04ab_6901_u64;

    let mut adjacent_probes = Vec::new();
    let mut adjacent_meta = String::from(META_HEADER);
    for (index, offset) in (-256_i64..=256).enumerate() {
        let bits = (i128::from(center) + i128::from(offset)) as u64;
        append_cos_search_probe(
            &mut adjacent_probes,
            &mut adjacent_meta,
            &format!("cos-adj-{index:04}"),
            "adjacent",
            f64::from_bits(bits),
        );
    }

    let mut random_values = BTreeMap::<u64, &'static str>::new();
    let mut state = 0x5731_09c0_5e11_2026_u64;
    while random_values
        .values()
        .filter(|class| **class == "near-random")
        .count()
        < 256
    {
        let raw = next_lcg(&mut state);
        let offset = i64::try_from(raw & 0x01ff_ffff).unwrap() - 0x0100_0000;
        let bits = (i128::from(center) + i128::from(offset)) as u64;
        let x = f64::from_bits(bits);
        if has_cos_candidate_disagreement(x) {
            random_values.insert(bits, "near-random");
        }
    }
    while random_values
        .values()
        .filter(|class| **class == "broad-random")
        .count()
        < 256
    {
        let raw_q = next_lcg(&mut state);
        let raw_r = next_lcg(&mut state);
        let quotient = 3_u64 + 4 * (raw_q % 16_777_215);
        let residue = ((raw_r >> 11) as f64 / ((1_u64 << 53) as f64) - 0.5) * 1.4;
        let x = quotient as f64 * std::f64::consts::FRAC_PI_2 + residue;
        if x < f64::from_bits(0x41a0_0000_0000_0000) && has_cos_candidate_disagreement(x) {
            random_values.insert(x.to_bits(), "broad-random");
        }
    }
    random_values.insert(0x4062_a6de_04ab_6900, "known-discriminator");
    random_values.insert(0x4062_a6de_04ab_6902, "known-discriminator");

    let mut random_probes = Vec::new();
    let mut random_meta = String::from(META_HEADER);
    for (index, (bits, class)) in random_values.into_iter().enumerate() {
        append_cos_search_probe(
            &mut random_probes,
            &mut random_meta,
            &format!("cos-rnd-{index:04}"),
            class,
            f64::from_bits(bits),
        );
    }

    let adjacent_batch = json!({
        "function": "COS",
        "row_id": "w109-cos-adjacent-disagreement-20260809",
        "probes": adjacent_probes,
    });
    let random_batch = json!({
        "function": "COS",
        "row_id": "w109-cos-random-disagreement-20260809",
        "probes": random_probes,
    });
    let adjacent_count = adjacent_batch["probes"].as_array().unwrap().len();
    let random_count = random_batch["probes"].as_array().unwrap().len();
    let manifest = json!({
        "schema_version": "oxfunc.w109.cos_disagreement_search.v1",
        "method": "oracle-blind deterministic candidate-disagreement search",
        "clean_room_sources": [
            "documented x87 public instruction semantics",
            "documented C sin/cos APIs",
            "live worksheet COS through public Excel interfaces"
        ],
        "adjacent_center_bits": format!("0x{center:016x}"),
        "adjacent_offsets_ulp": [-256, 256],
        "random_seed": "0x573109c05e112026",
        "candidate_axes": [
            "FCOS/FSIN/FSINCOS",
            "PC24/PC53/PC64 and RC modes",
            "FLDPI/2 construction",
            "residue store",
            "double-double trig series",
            "documented MSVCRT sin/cos"
        ],
        "adjacent_count": adjacent_count,
        "random_count": random_count,
    });

    for (file_name, value) in [
        (
            "batch-cos-adjacent-disagreement-20260809.json",
            adjacent_batch,
        ),
        ("batch-cos-random-disagreement-20260809.json", random_batch),
        ("batch-cos-disagreement-search-manifest.json", manifest),
    ] {
        let path = root.join(file_name);
        std::fs::write(&path, serde_json::to_string_pretty(&value).unwrap())
            .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
        println!("  {}", path.display());
    }
    std::fs::write(
        root.join("batch-cos-adjacent-disagreement-20260809-meta.csv"),
        adjacent_meta,
    )
    .expect("write COS adjacent metadata");
    std::fs::write(
        root.join("batch-cos-random-disagreement-20260809-meta.csv"),
        random_meta,
    )
    .expect("write COS random metadata");
    println!(
        "generated COS disagreement batches: {} adjacent + {} random",
        adjacent_count, random_count
    );
}

fn append_tangent_heldout_probe(
    probes: &mut Vec<Value>,
    metadata: &mut String,
    id: &str,
    class: &str,
    x: f64,
) {
    let divisor = half_pi_for_graph(CosGraph::default());
    let (residue, quotient) =
        rx::ext_prem1_quo(&rx::ext_from_f64(x.abs()), &divisor, rx::CW_PC64_RN);
    let tangent_square = cos_alternative_trig_model(
        x,
        AlternativeTrigMethod::TanSquareRatioSqrt,
        rx::CW_PC64_RN,
        0,
    );
    probes.push(json!({
        "probe": { "id": id, "args": [format_bits(x)] }
    }));
    metadata.push_str(&format!(
        "{id},{class},{},{},{},{},{}\n",
        format_bits(x),
        quotient,
        ext80_hex(&residue),
        format_bits(excel_cos_model(x)),
        format_bits(tangent_square),
    ));
}

fn generate_cos_tangent_heldout(root: &Path) {
    std::fs::create_dir_all(root).expect("create COS tangent held-out directory");
    let center = 0x4062_a6de_04ab_6901_u64;
    let known = [0x4062_a6de_04ab_6900_u64, 0x4062_a6de_04ab_6902];
    let differs = |x: f64| {
        excel_cos_model(x).to_bits()
            != cos_alternative_trig_model(
                x,
                AlternativeTrigMethod::TanSquareRatioSqrt,
                rx::CW_PC64_RN,
                0,
            )
            .to_bits()
    };

    let mut heldout = BTreeMap::<u64, &'static str>::new();
    let mut near_state = 0x91e1_0da5_5eed_2026_u64;
    let mut near_attempts = 0_u64;
    while heldout
        .values()
        .filter(|class| **class == "near-heldout")
        .count()
        < 256
    {
        near_attempts += 1;
        assert!(near_attempts < 20_000_000, "near held-out search exhausted");
        let raw = next_lcg(&mut near_state);
        let offset = i64::try_from(raw & 0x07ff_ffff).unwrap() - 0x0400_0000;
        let bits = (i128::from(center) + i128::from(offset)) as u64;
        if !known.contains(&bits) && differs(f64::from_bits(bits)) {
            heldout.insert(bits, "near-heldout");
        }
    }

    let mut broad_state = 0xa11c_e5ed_f109_2026_u64;
    let mut broad_attempts = 0_u64;
    while heldout
        .values()
        .filter(|class| **class == "broad-heldout")
        .count()
        < 256
    {
        broad_attempts += 1;
        assert!(
            broad_attempts < 20_000_000,
            "broad held-out search exhausted"
        );
        let raw_q = next_lcg(&mut broad_state);
        let raw_r = next_lcg(&mut broad_state);
        let quotient = 1_u64 + 2 * (raw_q % 16_777_215);
        let residue = ((raw_r >> 11) as f64 / ((1_u64 << 53) as f64) - 0.5) * 1.55;
        let x = quotient as f64 * std::f64::consts::FRAC_PI_2 + residue;
        if x < f64::from_bits(0x41a0_0000_0000_0000) && !known.contains(&x.to_bits()) && differs(x)
        {
            heldout.insert(x.to_bits(), "broad-heldout");
        }
    }

    let mut probes = Vec::new();
    let mut metadata = String::from(
        "id,class,x_bits,quotient_low3,residue_ext80,continuous_bits,tangent_square_bits\n",
    );
    for (index, (bits, class)) in heldout.into_iter().enumerate() {
        append_tangent_heldout_probe(
            &mut probes,
            &mut metadata,
            &format!("cos-tan-held-{index:04}"),
            class,
            f64::from_bits(bits),
        );
    }
    for (index, bits) in known.into_iter().enumerate() {
        append_tangent_heldout_probe(
            &mut probes,
            &mut metadata,
            &format!("cos-tan-control-{index:02}"),
            "known-control",
            f64::from_bits(bits),
        );
    }
    let batch = json!({
        "function": "COS",
        "row_id": "w109-cos-tangent-square-heldout-20260809",
        "probes": probes,
    });
    let manifest = json!({
        "schema_version": "oxfunc.w109.cos_tangent_square_heldout.v1",
        "method": "oracle-blind deterministic candidate-disagreement holdout",
        "frozen_candidate": "FPREM1(|x|, FLDPI/2); odd-quadrant sine = sign(FPTAN(r))*FSQRT(FPTAN(r)^2/(1+FPTAN(r)^2)); PC64/RN; no spills",
        "baseline": "continuous FPREM1 plus FSIN/FCOS dispatch",
        "near_seed": "0x91e10da55eed2026",
        "broad_seed": "0xa11ce5edf1092026",
        "near_attempts": near_attempts,
        "broad_attempts": broad_attempts,
        "heldout_count": 512,
        "known_control_count": 2,
        "oracle_answers_used_during_generation": false,
    });
    for (file_name, value) in [
        ("batch-cos-tangent-square-heldout-20260809.json", batch),
        (
            "batch-cos-tangent-square-heldout-20260809-manifest.json",
            manifest,
        ),
    ] {
        let path = root.join(file_name);
        std::fs::write(&path, serde_json::to_string_pretty(&value).unwrap())
            .unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
        println!("  {}", path.display());
    }
    let meta_path = root.join("batch-cos-tangent-square-heldout-20260809-meta.csv");
    std::fs::write(&meta_path, metadata)
        .unwrap_or_else(|error| panic!("write {}: {error}", meta_path.display()));
    println!("  {}", meta_path.display());
    println!(
        "generated COS tangent-square holdout: 512 held-out + 2 controls; attempts near={near_attempts} broad={broad_attempts}"
    );
}

fn generate_intermediate_discriminator(root: &Path) {
    std::fs::create_dir_all(root).expect("create BESSELJ work directory");
    let failing_x_bits = 0x4062_bfff_ffff_ffff_u64;
    let mut probes = Vec::new();
    let mut meta = String::from(
        "id,source_x_bits,source_offset_ulp,phase_offset_ulp,reduced_bits,platform_cos_bits,excel_cos_bits,platform_sin_bits,excel_sin_bits,j0_platform_bits,j0_excel_cos_bits,j0_excel_both_bits,j0_excel_both_cp_x87_bits\n",
    );

    for source_offset in -2_i32..=2 {
        let x = f64::from_bits((i128::from(failing_x_bits) + i128::from(source_offset)) as u64);
        let reduced = x - 0.785_398_164;
        for phase_offset in -2_i32..=2 {
            let phase = offset_positive_ulp(reduced, phase_offset);
            let id = format!("besj-j0-mid-{:03}", probes.len());
            let platform_cosine = phase.cos();
            let excel_cosine = excel_cos_model(phase);
            let platform_sine = phase.sin();
            let excel_sine = excel_sin_model(phase);
            probes.push(json!({
                "probe": {
                    "id": id,
                    "args": [format_bits(phase)]
                }
            }));
            meta.push_str(&format!(
                "{id},{},{source_offset},{phase_offset},{},{},{},{},{},{},{},{},{}\n",
                format_bits(x),
                format_bits(phase),
                format_bits(platform_cosine),
                format_bits(excel_cosine),
                format_bits(platform_sine),
                format_bits(excel_sine),
                format_bits(j0_from_trig(x, platform_cosine, platform_sine, false)),
                format_bits(j0_from_trig(x, excel_cosine, platform_sine, false)),
                format_bits(j0_from_trig(x, excel_cosine, excel_sine, false)),
                format_bits(j0_from_trig(x, excel_cosine, excel_sine, true)),
            ));
        }
    }

    let cos_batch = json!({
        "function": "COS",
        "row_id": "besselj-j0-intermediate-cos-scratch-20260809",
        "probes": probes,
    });
    let sin_batch = json!({
        "function": "SIN",
        "row_id": "besselj-j0-intermediate-sin-scratch-20260809",
        "probes": cos_batch["probes"].clone(),
    });
    let manifest = json!({
        "schema_version": "oxfunc.w109.besselj_j0_intermediate_discriminator.v0",
        "method": "clean-room live worksheet COS/SIN probes through bit-exact cell references",
        "failing_x_bits": format!("0x{failing_x_bits:016x}"),
        "source_x_offsets_ulp": [-2, -1, 0, 1, 2],
        "reduced_phase_offsets_ulp": [-2, -1, 0, 1, 2],
        "probe_count_per_function": cos_batch["probes"].as_array().unwrap().len(),
        "reconstruction": "meta freezes platform and established fFSIN/fFCOS outputs plus J0 plain/x87-cosine-product variants before oracle capture",
    });

    let cos_path = root.join("batch-besselj-j0-intermediate-cos-scratch.json");
    let sin_path = root.join("batch-besselj-j0-intermediate-sin-scratch.json");
    let meta_path = root.join("batch-besselj-j0-intermediate-scratch-meta.csv");
    let manifest_path = root.join("batch-besselj-j0-intermediate-scratch-manifest.json");
    std::fs::write(&cos_path, serde_json::to_string_pretty(&cos_batch).unwrap())
        .expect("write COS discriminator batch");
    std::fs::write(&sin_path, serde_json::to_string_pretty(&sin_batch).unwrap())
        .expect("write SIN discriminator batch");
    std::fs::write(&meta_path, meta).expect("write discriminator metadata");
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .expect("write discriminator manifest");
    println!(
        "wrote BESSELJ J0 intermediate discriminator: {} COS + {} SIN probes",
        cos_batch["probes"].as_array().unwrap().len(),
        sin_batch["probes"].as_array().unwrap().len()
    );
    println!("  {}", cos_path.display());
    println!("  {}", sin_path.display());
    println!("  {}", meta_path.display());
    println!("  {}", manifest_path.display());

    let followup_rows = [
        (
            "besj-cos-followup-000",
            0x405b_1c17_f000_0000_u64,
            "J0",
            0.785_398_164,
        ),
        (
            "besj-cos-followup-001",
            0x405b_1c17_f000_0000_u64,
            "J1",
            2.356_194_491,
        ),
        (
            "besj-cos-followup-002",
            0x4062_bfff_ffff_ffff_u64,
            "J1",
            2.356_194_491,
        ),
    ];
    let mut followup_probes = Vec::new();
    let mut followup_meta =
        String::from("id,site,x_bits,reduced_bits,platform_cos_bits,excel_cos_model_bits\n");
    for (id, x_bits, site, phase_constant) in followup_rows {
        let x = f64::from_bits(x_bits);
        let reduced = x - phase_constant;
        followup_probes.push(json!({
            "probe": { "id": id, "args": [format_bits(reduced)] }
        }));
        followup_meta.push_str(&format!(
            "{id},{site},{},{},{},{}\n",
            format_bits(x),
            format_bits(reduced),
            format_bits(reduced.cos()),
            format_bits(excel_cos_model(reduced)),
        ));
    }
    let followup_batch = json!({
        "function": "COS",
        "row_id": "besselj-cos-phase-followup-scratch-20260809",
        "probes": followup_probes,
    });
    let followup_path = root.join("batch-besselj-cos-phase-followup-scratch.json");
    let followup_meta_path = root.join("batch-besselj-cos-phase-followup-scratch-meta.csv");
    std::fs::write(
        &followup_path,
        serde_json::to_string_pretty(&followup_batch).unwrap(),
    )
    .expect("write COS phase follow-up batch");
    std::fs::write(&followup_meta_path, followup_meta).expect("write COS phase follow-up metadata");
    println!("wrote BESSELJ COS phase follow-up: 3 probes");
    println!("  {}", followup_path.display());
    println!("  {}", followup_meta_path.display());
}

fn load_cos_search_rows(root: &Path) -> Vec<CosCapturedRow> {
    let mut rows = Vec::new();
    for stem in [
        "cos-adjacent-disagreement-20260809",
        "cos-random-disagreement-20260809",
    ] {
        let meta_path = root.join(format!("batch-{stem}-meta.csv"));
        let mut classes = HashMap::new();
        for line in std::fs::read_to_string(&meta_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", meta_path.display()))
            .lines()
            .skip(1)
        {
            let mut fields = line.split(',');
            let id = fields.next().expect("COS metadata id");
            let class = fields.next().expect("COS metadata class");
            classes.insert(id.to_string(), class.to_string());
        }

        let answers_path = root.join(format!("answers-{stem}.json"));
        let answers: Value = serde_json::from_str(
            &std::fs::read_to_string(&answers_path)
                .unwrap_or_else(|error| panic!("read {}: {error}", answers_path.display())),
        )
        .unwrap_or_else(|error| panic!("parse {}: {error}", answers_path.display()));
        for witness in answers["witnesses"].as_array().expect("COS witnesses") {
            let id = witness["id"].as_str().expect("COS witness id").to_string();
            rows.push(CosCapturedRow {
                class: classes
                    .get(&id)
                    .unwrap_or_else(|| panic!("metadata class for {id}"))
                    .clone(),
                id,
                x: parse_bits(witness["args"][0].as_str().expect("COS argument bits")),
                expected: parse_bits(
                    witness["expected_bits"]
                        .as_str()
                        .expect("COS expected bits"),
                )
                .to_bits(),
            });
        }
    }
    rows
}

fn analyze_cos_search(root: &Path) {
    let rows = load_cos_search_rows(root);
    println!("worksheet COS large-battery replay ({} rows):", rows.len());
    for (name, candidate) in [
        ("platform", f64::cos as fn(f64) -> f64),
        ("continuous FPREM1", excel_cos_model as fn(f64) -> f64),
        (
            "residue-spill FPREM1",
            (|x| cos_quadrant_residue_spill(x, rx::CW_PC64_RN)) as fn(f64) -> f64,
        ),
        (
            "FSINCOS FPREM1",
            (|x| cos_quadrant_sincos(x, rx::CW_PC64_RN)) as fn(f64) -> f64,
        ),
        (
            "double-double series",
            cos_quadrant_double_double as fn(f64) -> f64,
        ),
        (
            "AMD poly SSE2 + ext tail",
            (|x| cos_quadrant_amd_poly(x, false, true)) as fn(f64) -> f64,
        ),
        (
            "AMD poly SSE2 spilled",
            (|x| cos_quadrant_amd_poly(x, false, false)) as fn(f64) -> f64,
        ),
        (
            "AMD poly FMA3 + ext tail",
            (|x| cos_quadrant_amd_poly(x, true, true)) as fn(f64) -> f64,
        ),
        (
            "AMD poly FMA3 spilled",
            (|x| cos_quadrant_amd_poly(x, true, false)) as fn(f64) -> f64,
        ),
    ] {
        let exact = rows
            .iter()
            .filter(|row| candidate(row.x).to_bits() == row.expected)
            .count();
        println!("  {name:26} {exact}/{}", rows.len());
    }

    let mut continuous_amd_relation = BTreeMap::<&'static str, usize>::new();
    for row in &rows {
        let continuous = excel_cos_model(row.x).to_bits() == row.expected;
        let amd = cos_quadrant_amd_poly(row.x, false, true).to_bits() == row.expected;
        let key = match (continuous, amd) {
            (true, true) => "both",
            (true, false) => "continuous-only",
            (false, true) => "amd-only",
            (false, false) => "neither",
        };
        *continuous_amd_relation.entry(key).or_default() += 1;
    }
    println!("  continuous/AMD relation {continuous_amd_relation:?}");

    let mut alternative_scores = Vec::new();
    for method in [
        AlternativeTrigMethod::TanNormalize,
        AlternativeTrigMethod::TanReciprocalMultiply,
        AlternativeTrigMethod::TanSqrtReciprocalMultiply,
        AlternativeTrigMethod::TanSquareRatioSqrt,
        AlternativeTrigMethod::TanHalfAngle,
        AlternativeTrigMethod::TanTimesCos,
        AlternativeTrigMethod::PythagoreanFromCos,
        AlternativeTrigMethod::NormalizeSeparateSinCos,
        AlternativeTrigMethod::NormalizePairedSinCos,
    ] {
        for (cw_name, cw) in [
            ("PC24", rx::CW_PC24_RN),
            ("PC53", rx::CW_PC53_RN),
            ("PC64", rx::CW_PC64_RN),
        ] {
            for spill_mask in 0_u8..64 {
                let exact = rows
                    .iter()
                    .filter(|row| {
                        cos_alternative_trig_model(row.x, method, cw, spill_mask).to_bits()
                            == row.expected
                    })
                    .count();
                alternative_scores.push((exact, method, cw_name, spill_mask));
            }
        }
    }
    alternative_scores.sort_by(|left, right| right.0.cmp(&left.0));
    println!(
        "alternative reduced-trig matrix: {} graphs; best {}/{}",
        alternative_scores.len(),
        alternative_scores[0].0,
        rows.len()
    );
    for (exact, method, cw_name, spill_mask) in alternative_scores.iter().take(24) {
        println!(
            "  {exact:4}/{} method={method:?} {cw_name} spill=0x{spill_mask:02x}",
            rows.len()
        );
    }
    let (_, best_method, _, best_spill_mask) = alternative_scores[0];
    let best_cw = rx::CW_PC64_RN;
    let mut continuous_alternative_relation = BTreeMap::<&'static str, usize>::new();
    println!("best alternative residuals:");
    for row in &rows {
        let continuous = excel_cos_model(row.x).to_bits() == row.expected;
        let candidate =
            cos_alternative_trig_model(row.x, best_method, best_cw, best_spill_mask).to_bits();
        let alternative = candidate == row.expected;
        let key = match (continuous, alternative) {
            (true, true) => "both",
            (true, false) => "continuous-only",
            (false, true) => "alternative-only",
            (false, false) => "neither",
        };
        *continuous_alternative_relation.entry(key).or_default() += 1;
        if !alternative {
            println!(
                "  id={} class={} x={} live=0x{:016x} candidate=0x{candidate:016x} continuous=0x{:016x}",
                row.id,
                row.class,
                format_bits(row.x),
                row.expected,
                excel_cos_model(row.x).to_bits(),
            );
        }
    }
    println!("  continuous/alternative relation {continuous_alternative_relation:?}");

    let prior_path = root
        .parent()
        .expect("W109 work root")
        .join("G4-01-cos/answers-validation.json");
    let prior: Value = serde_json::from_str(
        &std::fs::read_to_string(&prior_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", prior_path.display())),
    )
    .unwrap_or_else(|error| panic!("parse {}: {error}", prior_path.display()));
    let prior_rows = prior["witnesses"].as_array().expect("prior COS witnesses");
    let prior_continuous = prior_rows
        .iter()
        .filter(|row| {
            let x = parse_bits(row["args"][0].as_str().expect("prior COS argument"));
            excel_cos_model(x).to_bits()
                == parse_bits(
                    row["expected_bits"]
                        .as_str()
                        .expect("prior COS expected bits"),
                )
                .to_bits()
        })
        .count();
    let prior_alternative = prior_rows
        .iter()
        .filter(|row| {
            let x = parse_bits(row["args"][0].as_str().expect("prior COS argument"));
            cos_alternative_trig_model(x, best_method, best_cw, best_spill_mask).to_bits()
                == parse_bits(
                    row["expected_bits"]
                        .as_str()
                        .expect("prior COS expected bits"),
                )
                .to_bits()
        })
        .count();
    println!(
        "prior G4-01 validation: continuous {prior_continuous}/{}; tangent-square hybrid {prior_alternative}/{}",
        prior_rows.len(),
        prior_rows.len(),
    );

    let mut scores = explicit_reduction_candidates()
        .into_iter()
        .map(|(name, graph)| {
            let exact = rows
                .iter()
                .filter(|row| cos_explicit_reduction_model(row.x, graph).to_bits() == row.expected)
                .count();
            (exact, name, graph)
        })
        .collect::<Vec<_>>();
    scores.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    let best_exact = scores[0].0;
    println!(
        "explicit quotient/product reduction: {} graphs; best {best_exact}/{}",
        scores.len(),
        rows.len()
    );
    for (exact, name, _) in scores.iter().take(24) {
        println!("  {exact:4}/{} {name}", rows.len());
    }

    let (_, best_name, best_graph) = &scores[0];
    println!("best explicit graph residuals ({best_name}):");
    let mut by_class = BTreeMap::<String, usize>::new();
    let residuals = rows
        .iter()
        .filter(|row| cos_explicit_reduction_model(row.x, *best_graph).to_bits() != row.expected)
        .collect::<Vec<_>>();
    for row in &residuals {
        *by_class.entry(row.class.clone()).or_default() += 1;
    }
    for row in residuals.iter().take(32) {
        println!(
            "  {} class={} x={} expected=0x{:016x} candidate=0x{:016x} continuous=0x{:016x}",
            row.id,
            row.class,
            format_bits(row.x),
            row.expected,
            cos_explicit_reduction_model(row.x, *best_graph).to_bits(),
            excel_cos_model(row.x).to_bits(),
        );
    }
    println!(
        "  residual classes {by_class:?}; displayed {}/{}",
        residuals.len().min(32),
        residuals.len()
    );

    let mut rounding_rows = rows
        .iter()
        .map(|row| {
            let raw = excel_cos_ext_value(&rx::ext_from_f64(row.x));
            let discarded = ext80_significand(&raw) & 0x7ff;
            let midpoint_distance = discarded.abs_diff(0x400);
            (
                midpoint_distance,
                discarded,
                excel_cos_model(row.x).to_bits() != row.expected,
                row,
                raw,
            )
        })
        .collect::<Vec<_>>();
    rounding_rows.sort_by_key(|row| row.0);
    println!("closest raw-x87 results to a binary64 rounding midpoint:");
    for (distance, discarded, miss, row, raw) in rounding_rows.iter().take(32) {
        println!(
            "  dist={distance:4} low11=0x{discarded:03x} miss={} id={} x={} raw={} live=0x{:016x}",
            u8::from(*miss),
            row.id,
            format_bits(row.x),
            ext80_hex(raw),
            row.expected,
        );
        if *miss {
            println!(
                "    AMD-SSE2-tail=0x{:016x} spill=0x{:016x} DD=0x{:016x}",
                cos_quadrant_amd_poly(row.x, false, true).to_bits(),
                cos_quadrant_residue_spill(row.x, rx::CW_PC64_RN).to_bits(),
                cos_quadrant_double_double(row.x).to_bits(),
            );
        }
    }
}

fn analyze_cos_tangent_heldout(root: &Path) {
    let answers_path = root.join("answers-cos-tangent-square-heldout-20260809.json");
    if !answers_path.exists() {
        println!(
            "awaiting live COS tangent holdout: {}",
            answers_path.display()
        );
        return;
    }
    let answers: Value = serde_json::from_str(
        &std::fs::read_to_string(&answers_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", answers_path.display())),
    )
    .unwrap_or_else(|error| panic!("parse {}: {error}", answers_path.display()));
    let rows = answers["witnesses"]
        .as_array()
        .expect("COS tangent witnesses");
    let mut baseline_exact = 0_usize;
    let mut tangent_exact = 0_usize;
    let mut tangent_failures = Vec::new();
    for row in rows {
        let x = parse_bits(row["args"][0].as_str().expect("COS tangent argument"));
        let expected = parse_bits(
            row["expected_bits"]
                .as_str()
                .expect("COS tangent expected bits"),
        )
        .to_bits();
        baseline_exact += usize::from(excel_cos_model(x).to_bits() == expected);
        let candidate = cos_alternative_trig_model(
            x,
            AlternativeTrigMethod::TanSquareRatioSqrt,
            rx::CW_PC64_RN,
            0,
        )
        .to_bits();
        tangent_exact += usize::from(candidate == expected);
        if candidate != expected {
            tangent_failures.push((
                row["id"].as_str().expect("COS tangent id"),
                x,
                expected,
                candidate,
            ));
        }
    }
    println!(
        "COS tangent-square oracle-blind holdout: baseline {baseline_exact}/{}; tangent-square {tangent_exact}/{}",
        rows.len(),
        rows.len(),
    );
    for (id, x, expected, candidate) in tangent_failures.iter().take(32) {
        println!(
            "  miss id={id} x={} expected=0x{expected:016x} candidate=0x{candidate:016x} baseline=0x{:016x}",
            format_bits(*x),
            excel_cos_model(*x).to_bits(),
        );
    }
    if !tangent_failures.is_empty() {
        std::process::exit(2);
    }
}

fn analyze_intermediate_discriminator(root: &Path) {
    let cos_path = root.join("answers-besselj-j0-intermediate-cos-scratch.json");
    let sin_path = root.join("answers-besselj-j0-intermediate-sin-scratch.json");
    let cos_answers: Value = serde_json::from_str(
        &std::fs::read_to_string(&cos_path).expect("capture COS discriminator first"),
    )
    .expect("parse COS discriminator answers");
    let sin_answers: Value = serde_json::from_str(
        &std::fs::read_to_string(&sin_path).expect("capture SIN discriminator first"),
    )
    .expect("parse SIN discriminator answers");
    let cos_rows = cos_answers["witnesses"].as_array().expect("COS witnesses");
    let sin_rows = sin_answers["witnesses"].as_array().expect("SIN witnesses");
    assert_eq!(cos_rows.len(), sin_rows.len());

    let minus_one_64 = rx::ext_chs(&rx::ext_one(), rx::CW_PC64_RN);
    let pi_half_64 = rx::ext_scale(&rx::ext_pi(), &minus_one_64, rx::CW_PC64_RN);
    let minus_one_53 = rx::ext_chs(&rx::ext_one(), rx::CW_PC53_RN);
    let pi_half_53 = rx::ext_scale(&rx::ext_pi(), &minus_one_53, rx::CW_PC53_RN);
    let f64_pi_half = rx::ext_from_f64(std::f64::consts::FRAC_PI_2);
    let candidates: Vec<(&str, Box<dyn Fn(f64) -> f64>)> = vec![
        ("platform cos", Box::new(f64::cos)),
        ("MSVCRT cos", Box::new(msvcrt_cos)),
        ("fFCOS PC64", Box::new(excel_cos_model)),
        (
            "fFSINCOS PC64",
            Box::new(|x| cos_quadrant_sincos(x, rx::CW_PC64_RN)),
        ),
        (
            "reduced platform",
            Box::new(|x| cos_reduced_platform_model(x, false)),
        ),
        (
            "reduced sin_cos",
            Box::new(|x| cos_reduced_platform_model(x, true)),
        ),
        ("reduced MSVCRT", Box::new(cos_reduced_msvcrt_model)),
        ("reduced DD series", Box::new(cos_quadrant_double_double)),
        (
            "post-add-zero PC53",
            Box::new(|x| cos_graph_post_round(x, rx::CW_PC53_RN, 0)),
        ),
        (
            "post-mul-one PC53",
            Box::new(|x| cos_graph_post_round(x, rx::CW_PC53_RN, 1)),
        ),
        (
            "post-add-zero PC64",
            Box::new(|x| cos_graph_post_round(x, rx::CW_PC64_RN, 0)),
        ),
        (
            "complement FCOS",
            Box::new(|x| cos_quadrant_complement(x, false, false)),
        ),
        (
            "complement FSINCOS",
            Box::new(|x| cos_quadrant_complement(x, false, true)),
        ),
        (
            "complement FCOS spill",
            Box::new(|x| cos_quadrant_complement(x, true, false)),
        ),
        (
            "fFCOS residue spill",
            Box::new(|x| cos_quadrant_residue_spill(x, rx::CW_PC64_RN)),
        ),
        (
            "fFCOS FPREM",
            Box::new(|x| cos_quadrant_prem(x, rx::CW_PC64_RN)),
        ),
        (
            "fFCOS PC53",
            Box::new(move |x| cos_quadrant_with_divisor(x, &pi_half_53, rx::CW_PC53_RN)),
        ),
        (
            "fFCOS f64-pi/2",
            Box::new(move |x| cos_quadrant_with_divisor(x, &f64_pi_half, rx::CW_PC64_RN)),
        ),
        (
            "pi-parity PC64",
            Box::new(|x| cos_pi_parity(x, rx::CW_PC64_RN)),
        ),
        (
            "pi-parity spill",
            Box::new(|x| cos_pi_parity_spill(x, rx::CW_PC64_RN)),
        ),
        (
            "FPREM1 2pi cont",
            Box::new(|x| cos_two_pi(x, true, false, rx::CW_PC64_RN)),
        ),
        (
            "FPREM1 2pi spill",
            Box::new(|x| cos_two_pi(x, true, true, rx::CW_PC64_RN)),
        ),
        (
            "FPREM 2pi cont",
            Box::new(|x| cos_two_pi(x, false, false, rx::CW_PC64_RN)),
        ),
        (
            "FPREM 2pi spill",
            Box::new(|x| cos_two_pi(x, false, true, rx::CW_PC64_RN)),
        ),
        (
            "sin(x+pi/2) ext",
            Box::new(|x| cos_sin_shift_ext(x, rx::CW_PC64_RN)),
        ),
        (
            "sin(f64 shift)",
            Box::new(|x| excel_sin_model(x + std::f64::consts::FRAC_PI_2)),
        ),
        (
            "raw FCOS PC64",
            Box::new(|x| {
                rx::ext_to_f64(
                    &rx::ext_cos(&rx::ext_from_f64(x), rx::CW_PC64_RN),
                    rx::CW_PC64_RN,
                )
            }),
        ),
        (
            "raw FCOS PC53",
            Box::new(|x| {
                rx::ext_to_f64(
                    &rx::ext_cos(&rx::ext_from_f64(x), rx::CW_PC53_RN),
                    rx::CW_PC53_RN,
                )
            }),
        ),
        (
            "raw FSINCOS PC64",
            Box::new(|x| {
                let (_, cosine) = ext_sincos(&rx::ext_from_f64(x), rx::CW_PC64_RN);
                rx::ext_to_f64(&cosine, rx::CW_PC64_RN)
            }),
        ),
        (
            "fFCOS final RM",
            Box::new(|x| {
                rx::ext_to_f64(
                    &excel_cos_ext_value(&rx::ext_from_f64(x)),
                    rx::CW_PC64_RN | 0x0400,
                )
            }),
        ),
        (
            "fFCOS final RP",
            Box::new(|x| {
                rx::ext_to_f64(
                    &excel_cos_ext_value(&rx::ext_from_f64(x)),
                    rx::CW_PC64_RN | 0x0800,
                )
            }),
        ),
        (
            "fFCOS final RZ",
            Box::new(|x| {
                rx::ext_to_f64(
                    &excel_cos_ext_value(&rx::ext_from_f64(x)),
                    rx::CW_PC64_RN | 0x0c00,
                )
            }),
        ),
    ];
    let _keep_pi_half_64_alive = pi_half_64;

    println!("worksheet COS candidate matrix ({} rows):", cos_rows.len());
    for (name, candidate) in &candidates {
        let exact = cos_rows
            .iter()
            .filter(|row| {
                let x = parse_bits(row["args"][0].as_str().unwrap());
                candidate(x).to_bits()
                    == parse_bits(row["expected_bits"].as_str().unwrap()).to_bits()
            })
            .count();
        println!("  {name:22} {exact}/{}", cos_rows.len());
    }
    let mut graph_scores = cos_graph_candidates()
        .into_iter()
        .map(|(name, graph)| {
            let exact = cos_rows
                .iter()
                .filter(|row| {
                    let x = parse_bits(row["args"][0].as_str().unwrap());
                    cos_graph_model(x, graph).to_bits()
                        == parse_bits(row["expected_bits"].as_str().unwrap()).to_bits()
                })
                .count();
            (exact, name, graph)
        })
        .collect::<Vec<_>>();
    graph_scores.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    let best_graph_exact = graph_scores[0].0;
    println!(
        "focused x87 graph matrix: {} candidates; best {best_graph_exact}/{}",
        graph_scores.len(),
        cos_rows.len()
    );
    for (exact, name, graph) in graph_scores
        .iter()
        .filter(|(exact, _, _)| *exact == best_graph_exact)
        .take(24)
    {
        println!("  {exact:2}/{} {name:34} {graph:?}", cos_rows.len());
    }
    let captured_cos = cos_rows
        .iter()
        .map(|row| {
            (
                parse_bits(row["args"][0].as_str().unwrap()),
                parse_bits(row["expected_bits"].as_str().unwrap()).to_bits(),
            )
        })
        .collect::<Vec<_>>();
    let mut unique_phase_rows = BTreeMap::new();
    for (x, expected) in &captured_cos {
        unique_phase_rows.insert(x.to_bits(), *expected);
    }
    println!("unique phase diagnostics:");
    for (phase_bits, expected) in unique_phase_rows {
        let x = f64::from_bits(phase_bits);
        let divisor = half_pi_for_graph(CosGraph::default());
        let (residue, quotient) =
            rx::ext_prem1_quo(&rx::ext_from_f64(x.abs()), &divisor, rx::CW_PC64_RN);
        let (_, status) = ext_sin_with_status(&residue, rx::CW_PC64_RN);
        let baseline = excel_cos_model(x).to_bits();
        let spill = cos_quadrant_residue_spill(x, rx::CW_PC64_RN).to_bits();
        let mut significand_bytes = [0_u8; 8];
        significand_bytes.copy_from_slice(&residue.0[..8]);
        let significand = u64::from_le_bytes(significand_bytes);
        println!(
            "  x=0x{phase_bits:016x} residue_l16=0x{:04x} q={} C1={} live=0x{expected:016x} baseline={} spill={}",
            significand & 0xffff,
            quotient,
            (status >> 9) & 1,
            if baseline == expected { "yes" } else { "no" },
            if spill == expected { "yes" } else { "no" },
        );
    }
    let correction_unit = f64::from_bits(((1023 - 70) as u64) << 52);
    let mut best_correction_exact = 0_usize;
    let mut best_correction_steps = Vec::new();
    for steps in -8192_i32..=8192 {
        let correction = f64::from(steps) * correction_unit;
        let exact = captured_cos
            .iter()
            .filter(|(x, expected)| {
                cos_quadrant_dd_residue_correction(*x, correction).to_bits() == *expected
            })
            .count();
        if exact > best_correction_exact {
            best_correction_exact = exact;
            best_correction_steps.clear();
            best_correction_steps.push(steps);
        } else if exact == best_correction_exact {
            best_correction_steps.push(steps);
        }
    }
    println!(
        "ideal-DD fixed-residue correction: best {best_correction_exact}/{} at 2^-70 steps {:?} ({} total)",
        captured_cos.len(),
        &best_correction_steps[..best_correction_steps.len().min(16)],
        best_correction_steps.len()
    );
    let mut best_hardware_correction_exact = 0_usize;
    let mut best_hardware_correction_steps = Vec::new();
    for steps in -8192_i64..=8192 {
        let graph = CosGraph {
            residue_significand_delta: steps,
            ..CosGraph::default()
        };
        let exact = captured_cos
            .iter()
            .filter(|(x, expected)| cos_graph_model(*x, graph).to_bits() == *expected)
            .count();
        if exact > best_hardware_correction_exact {
            best_hardware_correction_exact = exact;
            best_hardware_correction_steps.clear();
            best_hardware_correction_steps.push(steps);
        } else if exact == best_hardware_correction_exact {
            best_hardware_correction_steps.push(steps);
        }
    }
    println!(
        "hardware fixed-residue correction: best {best_hardware_correction_exact}/{} at Ext80-significand steps {:?} ({} total)",
        captured_cos.len(),
        &best_hardware_correction_steps[..best_hardware_correction_steps.len().min(24)],
        best_hardware_correction_steps.len()
    );
    for phase_bits in [0x4062_a6de_04ab_6900_u64, 0x4062_a6de_04ab_6902] {
        let x = f64::from_bits(phase_bits);
        let mut outputs = BTreeMap::<u64, Vec<&str>>::new();
        for (_, name, graph) in &graph_scores {
            outputs
                .entry(cos_graph_model(x, *graph).to_bits())
                .or_default()
                .push(name);
        }
        println!("candidate outputs at phase 0x{phase_bits:016x}:");
        let divisor = half_pi_for_graph(CosGraph::default());
        let (residue, quotient) =
            rx::ext_prem1_quo(&rx::ext_from_f64(x.abs()), &divisor, rx::CW_PC64_RN);
        println!(
            "  residue={residue:?} quotient={} DD=0x{:016x} reduced-platform=0x{:016x}",
            quotient,
            cos_quadrant_double_double(x).to_bits(),
            cos_reduced_platform_model(x, false).to_bits()
        );
        for (output, names) in outputs {
            println!(
                "  0x{output:016x}: {} candidates; examples={:?}",
                names.len(),
                &names[..names.len().min(4)]
            );
        }
    }
    let sin_exact = sin_rows
        .iter()
        .filter(|row| {
            let x = parse_bits(row["args"][0].as_str().unwrap());
            excel_sin_model(x).to_bits()
                == parse_bits(row["expected_bits"].as_str().unwrap()).to_bits()
        })
        .count();
    println!("worksheet SIN fFSIN: {sin_exact}/{}", sin_rows.len());

    let anchor_id = "besj-j0-mid-012";
    let cos_anchor = cos_rows
        .iter()
        .find(|row| row["id"].as_str() == Some(anchor_id))
        .expect("COS anchor");
    let sin_anchor = sin_rows
        .iter()
        .find(|row| row["id"].as_str() == Some(anchor_id))
        .expect("SIN anchor");
    let x = f64::from_bits(0x4062_bfff_ffff_ffff);
    let live_cos = parse_bits(cos_anchor["expected_bits"].as_str().unwrap());
    let live_sin = parse_bits(sin_anchor["expected_bits"].as_str().unwrap());
    let j0_plain = j0_from_trig(x, live_cos, live_sin, false);
    let j0_cp_x87 = j0_from_trig(x, live_cos, live_sin, true);
    let both_cos = TrigRouting::from_mask(0b1010);
    let j1 = bessj1_asymptotic(x, both_cos, BodyStaging::default());
    let j2_plain = (2.0 / x) * j1 - j0_plain;
    let j2_cp_x87 = (2.0 / x) * j1 - j0_cp_x87;
    println!("anchor reconstruction at x=0x{:016x}:", x.to_bits());
    println!("  live COS       {}", format_bits(live_cos));
    println!("  live SIN       {}", format_bits(live_sin));
    println!("  J0 plain       {}", format_bits(j0_plain));
    println!("  J0 cp-x87      {}", format_bits(j0_cp_x87));
    println!("  J2 plain       {}", format_bits(j2_plain));
    println!("  J2 cp-x87      {}", format_bits(j2_cp_x87));
    println!("  Excel J0 target 0xbf495d8a81b9c8bf");
    println!("  Excel J2 target 0xbf18c693cd8c2560");

    let followup: Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("answers-besselj-cos-phase-followup-scratch.json"))
            .expect("capture COS phase follow-up first"),
    )
    .expect("parse COS phase follow-up answers");
    let followup_rows = followup["witnesses"].as_array().expect("follow-up rows");
    let live = |id: &str| {
        parse_bits(
            followup_rows
                .iter()
                .find(|row| row["id"].as_str() == Some(id))
                .expect("follow-up id")["expected_bits"]
                .as_str()
                .unwrap(),
        )
    };
    let x108 = f64::from_bits(0x405b_1c17_f000_0000);
    let j0_phase108 = x108 - 0.785_398_164;
    let j1_phase108 = x108 - 2.356_194_491;
    let j0_cos108 = live("besj-cos-followup-000");
    let j1_cos108 = live("besj-cos-followup-001");
    let j0_sin108 = excel_sin_model(j0_phase108);
    let j1_sin108 = excel_sin_model(j1_phase108);
    let j0_108_plain = j0_from_trig(x108, j0_cos108, j0_sin108, false);
    let j0_108_cp = j0_from_trig(x108, j0_cos108, j0_sin108, true);
    let j1_108 = j1_from_trig(x108, j1_cos108, j1_sin108, false);
    let j2_108_plain = (2.0 / x108) * j1_108 - j0_108_plain;
    let j2_108_cp = (2.0 / x108) * j1_108 - j0_108_cp;
    println!("x=108.43896102905273438 independent staging discriminator:");
    println!("  live J0 COS     {}", format_bits(j0_cos108));
    println!(
        "  fFCOS J0 model {}",
        format_bits(excel_cos_model(j0_phase108))
    );
    println!("  live J1 COS     {}", format_bits(j1_cos108));
    println!(
        "  fFCOS J1 model {}",
        format_bits(excel_cos_model(j1_phase108))
    );
    println!("  J2 plain-cp    {}", format_bits(j2_108_plain));
    println!("  J2 J0-cp-x87   {}", format_bits(j2_108_cp));
    println!("  Excel J2 target 0xbfa9b1eac88983f1");

    let j1_cos150 = live("besj-cos-followup-002");
    println!("x=150-1ULP J1 COS follow-up:");
    println!("  live            {}", format_bits(j1_cos150));
    println!(
        "  fFCOS model     {}",
        format_bits(excel_cos_model(x - 2.356_194_491))
    );
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109/G4-besselj");
    if std::env::args().any(|arg| arg == "--generate-cos-search") {
        generate_cos_search_batches(&root);
        return;
    }
    if std::env::args().any(|arg| arg == "--generate-cos-tangent-heldout") {
        generate_cos_tangent_heldout(&root);
        return;
    }
    if std::env::args().any(|arg| arg == "--generate-intermediate") {
        generate_intermediate_discriminator(&root);
        return;
    }
    if std::env::args().any(|arg| arg == "--analyze-intermediate") {
        analyze_intermediate_discriminator(&root);
        return;
    }
    if std::env::args().any(|arg| arg == "--analyze-cos-search") {
        analyze_cos_search(&root);
        return;
    }
    if std::env::args().any(|arg| arg == "--analyze-cos-tangent-heldout") {
        analyze_cos_tangent_heldout(&root);
        return;
    }
    let batch_path = root.join("batch-besselj-internal-trig-heldout-20260809.json");
    let answers_path = root.join("answers-besselj-internal-trig-heldout-20260809.json");
    let meta_path = root.join("batch-besselj-internal-trig-heldout-20260809-meta.csv");
    if !answers_path.exists() {
        let batch: Value = serde_json::from_str(
            &std::fs::read_to_string(&batch_path).expect("run generator before replay"),
        )
        .expect("parse BESSELJ batch");
        let count = batch["probes"].as_array().unwrap().len();
        let disagreements = count_preoracle_disagreements(&batch_path);
        println!("BESSELJ batch ready: {count} rows, {disagreements} model disagreements");
        println!("awaiting live answers: {}", answers_path.display());
        return;
    }

    let rows = load(&batch_path, &answers_path, &meta_path);
    let production_score = score("production kernel", &rows, true, |x, order| {
        besselj_kernel(x, order).expect("generated BESSELJ row is valid")
    });
    report_partition("production", &production_score.misses);
    println!("trig-routing candidate matrix:");
    let mut matrix = Vec::new();
    for mask in 0_u8..16 {
        let routing = TrigRouting::from_mask(mask);
        let label = routing.label();
        let candidate = score(&label, &rows, false, |x, order| {
            besselj_model(x, order, routing, BodyStaging::default())
        });
        matrix.push((mask, candidate.exact));
    }
    let platform_exact = matrix[0].1;
    let best_exact = matrix.iter().map(|(_, exact)| *exact).max().unwrap();
    let best_masks: Vec<_> = matrix
        .iter()
        .filter(|(_, exact)| *exact == best_exact)
        .map(|(mask, _)| *mask)
        .collect();
    let diagnostic_routing = TrigRouting::from_mask(best_masks[0]);
    let routing_score = score("best routing details", &rows, true, |x, order| {
        besselj_model(x, order, diagnostic_routing, BodyStaging::default())
    });
    report_partition("best routing", &routing_score.misses);

    // Mask 0b1010 is the minimal member of the best routing family: both cosine
    // sites use fFCOS, while both sine sites stay on the platform operation.
    let both_cos = TrigRouting::from_mask(0b1010);
    let mut j0_wrapper_axis = Vec::new();
    for j0 in 0_u8..32 {
        let staging = BodyStaging {
            j0,
            ..BodyStaging::default()
        };
        let exact = evaluate(&rows, |x, order| besselj_model(x, order, both_cos, staging)).exact;
        j0_wrapper_axis.push((j0, exact));
    }
    report_mask_axis(
        "J0 wrapper mask (cp,zs,zsq,sub,scale)",
        &j0_wrapper_axis,
        rows.len(),
    );
    let mut j1_wrapper_axis = Vec::new();
    for j1 in 0_u8..32 {
        let staging = BodyStaging {
            j1,
            ..BodyStaging::default()
        };
        let exact = evaluate(&rows, |x, order| besselj_model(x, order, both_cos, staging)).exact;
        j1_wrapper_axis.push((j1, exact));
    }
    report_mask_axis(
        "J1 wrapper mask (cp,zs,zsq,sub,scale)",
        &j1_wrapper_axis,
        rows.len(),
    );

    let mut best_body_exact = 0usize;
    let mut best_body = Vec::new();
    for j0 in 0_u8..32 {
        for j1 in 0_u8..32 {
            let staging = BodyStaging {
                j0,
                j1,
                ..BodyStaging::default()
            };
            let exact =
                evaluate(&rows, |x, order| besselj_model(x, order, both_cos, staging)).exact;
            match exact.cmp(&best_body_exact) {
                std::cmp::Ordering::Greater => {
                    best_body_exact = exact;
                    best_body.clear();
                    best_body.push(staging);
                }
                std::cmp::Ordering::Equal => best_body.push(staging),
                std::cmp::Ordering::Less => {}
            }
        }
    }
    let minimum_body_sites = best_body
        .iter()
        .map(|staging| staging.j0.count_ones() + staging.j1.count_ones())
        .min()
        .unwrap();
    best_body
        .retain(|staging| staging.j0.count_ones() + staging.j1.count_ones() == minimum_body_sites);
    println!(
        "joint wrapper best: {best_body_exact}/{}; minimum staged sites={minimum_body_sites}; masks={:?}",
        rows.len(),
        best_body
            .iter()
            .map(|staging| (staging.j0, staging.j1))
            .collect::<Vec<_>>()
    );

    let body = best_body[0];
    let mut best_continuous_exact = 0usize;
    let mut best_continuous = Vec::new();
    for j0_continuous in 0_u8..=16 {
        for j1_continuous in 0_u8..=16 {
            let mut staging = body;
            staging.j0_continuous = j0_continuous;
            staging.j1_continuous = j1_continuous;
            if j0_continuous != 0 {
                staging.j0 = 0;
            }
            if j1_continuous != 0 {
                staging.j1 = 0;
            }
            let exact =
                evaluate(&rows, |x, order| besselj_model(x, order, both_cos, staging)).exact;
            match exact.cmp(&best_continuous_exact) {
                std::cmp::Ordering::Greater => {
                    best_continuous_exact = exact;
                    best_continuous.clear();
                    best_continuous.push(staging);
                }
                std::cmp::Ordering::Equal => best_continuous.push(staging),
                std::cmp::Ordering::Less => {}
            }
        }
    }
    println!(
        "continuous-wrapper best: {best_continuous_exact}/{} modes={:?} (mode 0 disabled; mode-1 is cp/zs/zsq/body spill mask)",
        rows.len(),
        best_continuous
            .iter()
            .map(|staging| (staging.j0_continuous, staging.j1_continuous))
            .collect::<Vec<_>>()
    );

    let mut best_raw_cos_exact = 0usize;
    let mut best_raw_cos = Vec::new();
    for j0_raw_cos in 0_u8..=16 {
        for j1_raw_cos in 0_u8..=16 {
            let mut staging = body;
            staging.j0_raw_cos = j0_raw_cos;
            staging.j1_raw_cos = j1_raw_cos;
            if j0_raw_cos != 0 {
                staging.j0 = 0;
                staging.j0_continuous = 0;
            }
            if j1_raw_cos != 0 {
                staging.j1 = 0;
                staging.j1_continuous = 0;
            }
            let exact =
                evaluate(&rows, |x, order| besselj_model(x, order, both_cos, staging)).exact;
            match exact.cmp(&best_raw_cos_exact) {
                std::cmp::Ordering::Greater => {
                    best_raw_cos_exact = exact;
                    best_raw_cos.clear();
                    best_raw_cos.push(staging);
                }
                std::cmp::Ordering::Equal => best_raw_cos.push(staging),
                std::cmp::Ordering::Less => {}
            }
        }
    }
    println!(
        "raw-fFCOS-return best: {best_raw_cos_exact}/{} modes={:?} (mode 0 disabled; mode-1 is cp/zs/zsq/body spill mask)",
        rows.len(),
        best_raw_cos
            .iter()
            .map(|staging| (staging.j0_raw_cos, staging.j1_raw_cos))
            .collect::<Vec<_>>()
    );

    let mut best_raw_scale_exact = 0usize;
    let mut best_raw_scale = Vec::new();
    for j0_raw_scale in 0_u8..=2 {
        for j1_raw_scale in 0_u8..=2 {
            let staging = BodyStaging {
                j0_raw_scale,
                j1_raw_scale,
                ..body
            };
            let exact =
                evaluate(&rows, |x, order| besselj_model(x, order, both_cos, staging)).exact;
            println!(
                "raw-scale J0={j0_raw_scale} J1={j1_raw_scale}: {exact}/{}",
                rows.len()
            );
            match exact.cmp(&best_raw_scale_exact) {
                std::cmp::Ordering::Greater => {
                    best_raw_scale_exact = exact;
                    best_raw_scale.clear();
                    best_raw_scale.push(staging);
                }
                std::cmp::Ordering::Equal => best_raw_scale.push(staging),
                std::cmp::Ordering::Less => {}
            }
        }
    }
    println!(
        "raw-scale best: {best_raw_scale_exact}/{} modes={:?}",
        rows.len(),
        best_raw_scale
            .iter()
            .map(|staging| (staging.j0_raw_scale, staging.j1_raw_scale))
            .collect::<Vec<_>>()
    );

    let body = best_raw_scale[0];
    let mut j0_setup_axis = Vec::new();
    for j0_setup in 0_u8..64 {
        let staging = BodyStaging { j0_setup, ..body };
        let exact = evaluate(&rows, |x, order| besselj_model(x, order, both_cos, staging)).exact;
        j0_setup_axis.push((j0_setup, exact));
    }
    report_mask_axis(
        "J0 setup mask (8/x,z*z,x-phase,(2/pi)/x,sqrt,phase->cos80)",
        &j0_setup_axis,
        rows.len(),
    );
    let mut j1_setup_axis = Vec::new();
    for j1_setup in 0_u8..64 {
        let staging = BodyStaging { j1_setup, ..body };
        let exact = evaluate(&rows, |x, order| besselj_model(x, order, both_cos, staging)).exact;
        j1_setup_axis.push((j1_setup, exact));
    }
    report_mask_axis(
        "J1 setup mask (8/x,z*z,x-phase,(2/pi)/x,sqrt,phase->cos80)",
        &j1_setup_axis,
        rows.len(),
    );

    let mut best_setup_exact = 0usize;
    let mut best_setup = Vec::new();
    for body in &best_raw_scale {
        for j0_setup in 0_u8..64 {
            for j1_setup in 0_u8..64 {
                let staging = BodyStaging {
                    j0_setup,
                    j1_setup,
                    ..*body
                };
                let exact =
                    evaluate(&rows, |x, order| besselj_model(x, order, both_cos, staging)).exact;
                match exact.cmp(&best_setup_exact) {
                    std::cmp::Ordering::Greater => {
                        best_setup_exact = exact;
                        best_setup.clear();
                        best_setup.push(staging);
                    }
                    std::cmp::Ordering::Equal => best_setup.push(staging),
                    std::cmp::Ordering::Less => {}
                }
            }
        }
    }
    let minimum_setup_sites = best_setup
        .iter()
        .map(|staging| {
            staging.j0_setup.count_ones()
                + staging.j0.count_ones()
                + staging.j0_p.count_ones()
                + staging.j0_q.count_ones()
                + staging.j1_setup.count_ones()
                + staging.j1.count_ones()
        })
        .min()
        .unwrap();
    best_setup.retain(|staging| {
        staging.j0_setup.count_ones()
            + staging.j0.count_ones()
            + staging.j0_p.count_ones()
            + staging.j0_q.count_ones()
            + staging.j1_setup.count_ones()
            + staging.j1.count_ones()
            == minimum_setup_sites
    });
    println!(
        "joint setup best: {best_setup_exact}/{}; minimum staged sites={minimum_setup_sites}; masks={:?}",
        rows.len(),
        best_setup
            .iter()
            .map(|staging| (staging.j0_setup, staging.j0, staging.j1_setup, staging.j1))
            .collect::<Vec<_>>()
    );

    let setup = best_setup[0];
    let mut best_poly_exact = 0usize;
    let mut best_poly = Vec::new();
    let mut p_axis = Vec::new();
    let mut q_axis = Vec::new();
    for j0_p in 0_u8..=u8::MAX {
        let staging = BodyStaging { j0_p, ..setup };
        let exact = evaluate(&rows, |x, order| besselj_model(x, order, both_cos, staging)).exact;
        p_axis.push((j0_p, exact));
        match exact.cmp(&best_poly_exact) {
            std::cmp::Ordering::Greater => {
                best_poly_exact = exact;
                best_poly.clear();
                best_poly.push(staging);
            }
            std::cmp::Ordering::Equal => best_poly.push(staging),
            std::cmp::Ordering::Less => {}
        }
    }
    for j0_q in 0_u8..=u8::MAX {
        let staging = BodyStaging { j0_q, ..setup };
        let exact = evaluate(&rows, |x, order| besselj_model(x, order, both_cos, staging)).exact;
        q_axis.push((j0_q, exact));
        match exact.cmp(&best_poly_exact) {
            std::cmp::Ordering::Greater => {
                best_poly_exact = exact;
                best_poly.clear();
                best_poly.push(staging);
            }
            std::cmp::Ordering::Equal => best_poly.push(staging),
            std::cmp::Ordering::Less => {}
        }
    }
    let p_best = p_axis.iter().map(|(_, exact)| *exact).max().unwrap();
    let q_best = q_axis.iter().map(|(_, exact)| *exact).max().unwrap();
    let p_best_masks: Vec<_> = p_axis
        .iter()
        .filter(|(_, exact)| *exact == p_best)
        .map(|(mask, _)| *mask)
        .collect();
    let q_best_masks: Vec<_> = q_axis
        .iter()
        .filter(|(_, exact)| *exact == q_best)
        .map(|(mask, _)| *mask)
        .collect();
    println!(
        "J0 Horner axes (bits alternate multiply/add, high-to-low): P best={p_best}/{} {}; Q best={q_best}/{} {}",
        rows.len(),
        mask_summary(&p_best_masks),
        rows.len(),
        mask_summary(&q_best_masks)
    );
    let minimum_poly_sites = best_poly
        .iter()
        .map(|staging| {
            staging.j0_setup.count_ones()
                + staging.j0.count_ones()
                + staging.j0_p.count_ones()
                + staging.j0_q.count_ones()
                + staging.j1_setup.count_ones()
                + staging.j1.count_ones()
        })
        .min()
        .unwrap();
    best_poly.retain(|staging| {
        staging.j0_setup.count_ones()
            + staging.j0.count_ones()
            + staging.j0_p.count_ones()
            + staging.j0_q.count_ones()
            + staging.j1_setup.count_ones()
            + staging.j1.count_ones()
            == minimum_poly_sites
    });
    best_poly.dedup();
    println!(
        "single-polynomial-axis best: {best_poly_exact}/{}; minimum staged sites={minimum_poly_sites}; masks={:?}",
        rows.len(),
        best_poly
            .iter()
            .map(|staging| (staging.j0_p, staging.j0_q))
            .collect::<Vec<_>>()
    );

    let mut best_final_exact = 0usize;
    let mut best_final = Vec::new();
    for setup in &best_poly {
        println!(
            "upward-recurrence staging axis for J0S=0x{:02x}, J0=0x{:02x}, J0P=0x{:02x}, J0Q=0x{:02x}, J1S=0x{:02x}, J1=0x{:02x} (bits div,jtox,mul,sub):",
            setup.j0_setup, setup.j0, setup.j0_p, setup.j0_q, setup.j1_setup, setup.j1
        );
        for recurrence in 0_u8..16 {
            let staging = BodyStaging {
                recurrence,
                ..*setup
            };
            let exact =
                evaluate(&rows, |x, order| besselj_model(x, order, both_cos, staging)).exact;
            println!("  R=0x{recurrence:01x} {exact:4}/{}", rows.len());
            match exact.cmp(&best_final_exact) {
                std::cmp::Ordering::Greater => {
                    best_final_exact = exact;
                    best_final.clear();
                    best_final.push(staging);
                }
                std::cmp::Ordering::Equal => best_final.push(staging),
                std::cmp::Ordering::Less => {}
            }
        }
    }
    let minimum_final_sites = best_final
        .iter()
        .map(|staging| {
            staging.j0_setup.count_ones()
                + staging.j0.count_ones()
                + staging.j0_p.count_ones()
                + staging.j0_q.count_ones()
                + staging.j1_setup.count_ones()
                + staging.j1.count_ones()
                + staging.recurrence.count_ones()
        })
        .min()
        .unwrap();
    best_final.retain(|staging| {
        staging.j0_setup.count_ones()
            + staging.j0.count_ones()
            + staging.j0_p.count_ones()
            + staging.j0_q.count_ones()
            + staging.j1_setup.count_ones()
            + staging.j1.count_ones()
            + staging.recurrence.count_ones()
            == minimum_final_sites
    });
    let final_staging = best_final[0];
    println!(
        "staged best: {best_final_exact}/{}; minimum staged sites={minimum_final_sites}; masks={:?}",
        rows.len(),
        best_final
            .iter()
            .map(|staging| {
                (
                    staging.j0_setup,
                    staging.j0,
                    staging.j0_p,
                    staging.j0_q,
                    staging.j1_setup,
                    staging.j1,
                    staging.recurrence,
                )
            })
            .collect::<Vec<_>>()
    );
    let final_score = score("staged best details", &rows, true, |x, order| {
        besselj_model(x, order, both_cos, final_staging)
    });
    report_partition("staged best", &final_score.misses);
    let live_cos_diagnostic = BodyStaging {
        live_cos_corrections: true,
        j0: 1,
        ..BodyStaging::default()
    };
    let live_cos_score = score("live-COS decomposition", &rows, true, |x, order| {
        besselj_model(x, order, both_cos, live_cos_diagnostic)
    });
    let tangent_square_cos_candidate = BodyStaging {
        tangent_square_cos: true,
        j0: 1,
        ..BodyStaging::default()
    };
    let tangent_square_cos_score = score(
        "tangent-square COS + J0 cos*p x87",
        &rows,
        true,
        |x, order| besselj_model(x, order, both_cos, tangent_square_cos_candidate),
    );
    println!(
        "summary: production={}/{} platform={platform_exact}/{} trig_best={best_exact}/{} trig_masks={best_masks:?} staged_best={best_final_exact}/{} live_cos_decomposition={}/{} tangent_square_cos={}/{} staged_masks={:?}",
        production_score.exact,
        rows.len(),
        rows.len(),
        rows.len(),
        rows.len(),
        live_cos_score.exact,
        rows.len(),
        tangent_square_cos_score.exact,
        rows.len(),
        best_final
            .iter()
            .map(|staging| {
                (
                    staging.j0_setup,
                    staging.j0,
                    staging.j0_p,
                    staging.j0_q,
                    staging.j1_setup,
                    staging.j1,
                    staging.recurrence,
                )
            })
            .collect::<Vec<_>>()
    );
    if tangent_square_cos_score.exact != rows.len() {
        std::process::exit(2);
    }
}
