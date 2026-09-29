import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptComplex [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b => if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b => if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def complexTextMetaBase : FunctionMeta := {
  functionId := "FUNC.IM_TEXT_BASE"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.none
}

def complexNumberMetaBase : FunctionMeta := {
  complexTextMetaBase with
  functionId := "FUNC.IM_NUMBER_BASE"
}

def complexMeta : FunctionMeta := {
  complexTextMetaBase with
  functionId := "FUNC.COMPLEX"
  arity := { min := 2, max := 3 }
}

def imabsMeta : FunctionMeta := { complexNumberMetaBase with functionId := "FUNC.IMABS" }
def imaginaryMeta : FunctionMeta := { complexNumberMetaBase with functionId := "FUNC.IMAGINARY" }
def imargumentMeta : FunctionMeta := { complexNumberMetaBase with functionId := "FUNC.IMARGUMENT" }
def imconjugateMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMCONJUGATE" }
def imcosMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMCOS" }
def imcoshMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMCOSH" }
def imcotMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMCOT" }
def imcscMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMCSC" }
def imcschMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMCSCH" }
def imdivMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMDIV", arity := Arity.exact 2 }
def imexpMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMEXP" }
def imlnMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMLN" }
def imlog10Meta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMLOG10" }
def imlog2Meta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMLOG2" }
def impowerMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMPOWER", arity := Arity.exact 2 }
def improductMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMPRODUCT", arity := { min := 1, max := 255 } }
def imrealMeta : FunctionMeta := { complexNumberMetaBase with functionId := "FUNC.IMREAL" }
def imsecMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMSEC" }
def imsechMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMSECH" }
def imsinMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMSIN" }
def imsinhMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMSINH" }
def imsqrtMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMSQRT" }
def imsubMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMSUB", arity := Arity.exact 2 }
def imsumMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMSUM", arity := { min := 1, max := 255 } }
def imtanMeta : FunctionMeta := { complexTextMetaBase with functionId := "FUNC.IMTAN" }

theorem complexFamily_meta_profiles :
    complexMeta.arity = { min := 2, max := 3 }
    ∧ imabsMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ imaginaryMeta.kernelSignatureClass = KernelSignatureClass.custom
    ∧ imargumentMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ imconjugateMeta.threadSafety = ThreadSafetyClass.safePure
    ∧ imdivMeta.arity = Arity.exact 2
    ∧ impowerMeta.arity = Arity.exact 2
    ∧ improductMeta.arity = { min := 1, max := 255 }
    ∧ imsumMeta.arity = { min := 1, max := 255 }
    ∧ imtanMeta.hostInteraction = HostInteractionClass.none := by
  simp [
    complexTextMetaBase,
    complexNumberMetaBase,
    complexMeta,
    imabsMeta,
    imaginaryMeta,
    imargumentMeta,
    imconjugateMeta,
    imdivMeta,
    impowerMeta,
    improductMeta,
    imsumMeta,
    imtanMeta
  ]

theorem complexFamily_ids :
    complexMeta.functionId = "FUNC.COMPLEX"
    ∧ imabsMeta.functionId = "FUNC.IMABS"
    ∧ imdivMeta.functionId = "FUNC.IMDIV"
    ∧ impowerMeta.functionId = "FUNC.IMPOWER"
    ∧ imsumMeta.functionId = "FUNC.IMSUM"
    ∧ imtanMeta.functionId = "FUNC.IMTAN" := by
  simp [complexMeta, imabsMeta, imdivMeta, impowerMeta, imsumMeta, imtanMeta]

/-- Exact positive-rational nearest-integer rounding, with ties to even. -/
def complexRatioNearestEven (numerator denominator : Nat) : Nat :=
  let q := numerator / denominator
  let r := numerator % denominator
  q + if 2 * r > denominator ∨ (2 * r = denominator ∧ q % 2 = 1) then 1 else 0

/-- Multiply a positive rational by an exact binary power, without floating arithmetic. -/
def complexRatioBinaryScale (numerator denominator : Nat) (shift : Int) : Nat × Nat :=
  if shift ≥ 0 then (numerator * 2 ^ shift.toNat, denominator)
  else (numerator, denominator * 2 ^ (-shift).toNat)

/-- RN64 as an executable mathematical substrate: a 64-bit binary significand rounded to
nearest-even, with unbounded exponent. Inputs are positive. The result `(m,e)` denotes
`m * 2^e`; this is an arithmetic model, not a statement about Excel's internal machinery. -/
def complexRatioRound64 (numerator denominator : Nat) : Nat × Int :=
  let trial := (Nat.log2 numerator : Int) - (Nat.log2 denominator : Int)
  let comparison := complexRatioBinaryScale denominator 1 trial
  let exponent := if numerator * comparison.2 ≥ comparison.1 then trial else trial - 1
  let scaled := complexRatioBinaryScale numerator denominator (63 - exponent)
  (complexRatioNearestEven scaled.1 scaled.2, exponent - 63)

/-- COMPLEX's coefficient scaling model for exact binary input `mantissa * 2^exponent`.
The caller supplies `k = 14-floor(log10(x))`. Positive k multiplies by RN64(10^k);
negative k divides by RN64(10^-k). The scaled operation rounds to RN64 before the
half-away integer step. W111 retained midpoint probes distinguish reciprocal multiplication
from division and distinguish this binary scaling from exact decimal rounding. -/
def complexRoundedCoefficient (mantissa : Nat) (exponent k : Int) : Nat :=
  let power := complexRatioRound64 (10 ^ k.natAbs) 1
  let operand := if k ≥ 0 then
    complexRatioBinaryScale (mantissa * power.1) 1 (exponent + power.2)
    else complexRatioBinaryScale mantissa power.1 (exponent - power.2)
  let scaled := complexRatioRound64 operand.1 operand.2
  let ratio := complexRatioBinaryScale scaled.1 1 scaled.2
  ratio.1 / ratio.2 + if 2 * (ratio.1 % ratio.2) ≥ ratio.2 then 1 else 0

theorem complex_ratio_rounding_ties :
    complexRatioNearestEven 5 2 = 2 ∧ complexRatioNearestEven 7 2 = 4 := by
  native_decide

/-- Exact binary inputs from the retained discriminator capture, not decimal substitutes.
They exercise multiplication, division, and both sides of the proposed midpoint behavior. -/
theorem complex_scaling_midpoint_witnesses :
    complexRoundedCoefficient 6371481190497722 (-332) 99 = 728252266453108
    ∧ complexRoundedCoefficient 5942363653846859 163 (-50) = 694781936777026
    ∧ complexRoundedCoefficient 7019420237467265 (-465) 139 = 736806179722723
    ∧ complexRoundedCoefficient 6132505895080754 (-624) 187 = 880887972081640
    ∧ complexRoundedCoefficient 8866857924100786 (-167) 49 = 473980498978759 := by
  native_decide

/-- Remove fractional trailing zeroes and then a trailing point. The caller passes an
ASCII decimal representation; integer trailing zeroes remain significant. -/
private def complexTrimFraction (s : String) : String :=
  if s.contains '.' then
    let cs := s.toList.reverse.dropWhile (· == '0')
    String.ofList ((if cs.head? == some '.' then cs.drop 1 else cs).reverse)
  else s

/-- Decimal presentation substrate after the coefficient has been rounded to 15 significant
digits, with half-away tie handling. `digits` is a nonempty unsigned decimal significand and
`scale` its decimal exponent. Rounding and the normal-range boundary are separate from this
presentation layer; no binary64 reconstruction is needed to render the decimal pair.
W111, Excel 16.0 build 20430: fixed notation fits at most 21 unsigned characters. -/
def complexRenderRoundedMagnitude (digits : String) (scale : Int) : String :=
  let point := (digits.length : Int) + scale
  let fixed := complexTrimFraction <|
    if point ≤ 0 then
      "0." ++ String.ofList (List.replicate (-point).toNat '0') ++ digits
    else if point.toNat ≥ digits.length then
      digits ++ String.ofList (List.replicate (point.toNat - digits.length) '0')
    else
      String.ofList (digits.toList.take point.toNat) ++ "." ++
        String.ofList (digits.toList.drop point.toNat)
  if fixed.length ≤ 21 then fixed else
    let exponent := point - 1
    let mantissa := complexTrimFraction <|
      String.ofList (digits.toList.take 1) ++ "." ++ String.ofList (digits.toList.drop 1)
    mantissa ++ "E" ++ (if exponent < 0 then "-" else "+") ++
      (if exponent.natAbs < 10 then "0" else "") ++ toString exponent.natAbs

/-- Component presence is decided from the original coefficient, before decimal formatting.
A nonzero coefficient whose decimal rounds below the normal range still contributes text
zero; it does not erase the component. The suffix and unit coefficient rules are unchanged. -/
def complexAssembleText (realZero imagZero imagUnit imagNegative : Bool)
    (realText imagMagnitudeText suffix : String) : String :=
  if imagZero then realText else
    let coefficient := if imagUnit then "" else imagMagnitudeText
    if realZero then (if imagNegative then "-" else "") ++ coefficient ++ suffix
    else realText ++ (if imagNegative then "-" else "+") ++ coefficient ++ suffix

theorem complex_render_keeps_small_nonzero_coefficients :
    complexRenderRoundedMagnitude "100000000000000" (-30) = "0.0000000000000001" := by
  native_decide

theorem complex_render_notation_boundary :
    complexRenderRoundedMagnitude "100000000000000" 6 = "100000000000000000000"
    ∧ complexRenderRoundedMagnitude "100000000000000" 7 = "1E+21"
    ∧ complexRenderRoundedMagnitude "123456789012345" (-19) = "0.0000123456789012345"
    ∧ complexRenderRoundedMagnitude "123456789012345" (-20) = "1.23456789012345E-06" := by
  native_decide

theorem complex_render_preserves_decimal_outside_binary64_range :
    complexRenderRoundedMagnitude "179769313486232" 294 = "1.79769313486232E+308" := by
  native_decide

theorem complex_rounded_zero_preserves_original_component :
    complexAssembleText false false true false "-0" "1" "i" = "-0+i"
    ∧ complexAssembleText true false false false "0" "0" "i" = "0i"
    ∧ complexAssembleText true false true true "0" "1" "j" = "-j" := by
  native_decide

/-- Algebraic layer of the scaled division operation graph. This uses the existing rational
carrier; IEEE rounding, arithmetic underflow and formatting are bound separately by the Rust
live replays. In particular, this rational layer does not predict the observed nonzero
floating residual of a complex number divided by itself. -/
def complexScaledDivision (a b c d : Rat) : Except WorksheetErrorCode (Rat × Rat) :=
  if c = 0 ∧ d = 0 then .error .num else
    let absC := if c < 0 then -c else c
    let absD := if d < 0 then -d else d
    if absC ≥ absD then
      let ratio := d / c
      let denominator := c + d * ratio
      .ok ((a + b * ratio) / denominator, (b - a * ratio) / denominator)
    else
      let ratio := c / d
      let denominator := d + c * ratio
      .ok ((a * ratio + b) / denominator, (b * ratio - a) / denominator)

/-- A subnormal arithmetic quotient is a zero component before text assembly. This boundary
is distinct from decimal rendering of a normal coefficient whose rounded text becomes zero. -/
def complexDivisionPublishCoefficient (value : Rat) : Rat :=
  let magnitude := if value < 0 then -value else value
  if magnitude < (1 / (2^1022 : Nat) : Rat) then 0 else value

theorem complex_scaled_division_axis_and_branch_examples :
    complexScaledDivision 3 4 2 0 = .ok (3/2, 2)
    ∧ complexScaledDivision 3 4 0 2 = .ok (2, -3/2)
    ∧ complexScaledDivision 3 4 3 4 = .ok (1, 0)
    ∧ complexScaledDivision 3 4 0 0 = .error .num := by
  native_decide

theorem complex_division_underflow_before_assembly :
    complexDivisionPublishCoefficient (-(1 / (2^1023 : Nat) : Rat)) = 0
    ∧ complexDivisionPublishCoefficient (1 / (2^1022 : Nat) : Rat) =
      (1 / (2^1022 : Nat) : Rat) := by
  native_decide

/-- Binary64 binding for the admitted nonzero-denominator division graph. The rational
layer above covers the zero-denominator error. Publishing components precedes formatting. -/
def complexScaledDivisionFloat (a b c d : Float) : Float × Float :=
  let value := if c.abs ≥ d.abs then
    let ratio := d / c
    let denominator := c + d * ratio
    ((a + b * ratio) / denominator, (b - a * ratio) / denominator)
  else
    let ratio := c / d
    let denominator := d + c * ratio
    ((a * ratio + b) / denominator, (b * ratio - a) / denominator)
  let publish := fun x : Float => if x.abs < Float.ofBits 0x0010000000000000 then 0 else x
  (publish value.1, publish value.2)

def complexDivisionFloatBits (a b c d : Float) : UInt64 × UInt64 :=
  let value := complexScaledDivisionFloat a b c d
  (value.1.toBits, value.2.toBits)

theorem complex_division_floating_self_residual :
    complexDivisionFloatBits (Float.ofBits 0x5f2d53cfc578e207)
      (Float.ofBits 0x5f338d352e5096af) (Float.ofBits 0x5f2d53cfc578e207)
      (Float.ofBits 0x5f338d352e5096af) = (0x3ff0000000000000, 0xbc90c27fa028b0ee) := by
  native_decide

/-- Executable binary64 squared-term boundary for IMSQRT. The comparison occurs on the
rounded multiplication result, before addition. This must not be replaced by hypot. -/
def complexSqrtSquaredTerm (value : Float) : Float :=
  let squared := value * value
  if squared < Float.ofBits 0x0010000000000000 then 0 else squared

/-- The observed radius graph preserves individual-square overflow, but publishes zero when
two finite squared terms overflow during addition. The two square roots are separate steps. -/
def complexSqrtRadius (re im : Float) : Float :=
  let a := complexSqrtSquaredTerm re
  let b := complexSqrtSquaredTerm im
  let sum := a + b
  if a.isFinite && b.isFinite && sum.isInf then 0 else sum.sqrt.sqrt

/-- Principal argument reconstruction used by the IMSQRT operation graph. The explicit ratio
and quadrant adjustment are observably distinct from a single atan2 call. Trig reconstruction
is bound to the separately characterized SIN/COS backend in Rust. -/
def complexRawArgumentWithAtan (atan : Float → Float) (real imaginary : Float) : Float :=
  let re := if real == 0 then (0 : Float) else real
  let im := if imaginary == 0 then (0 : Float) else imaginary
  let pi := Float.ofBits 0x400921fb54442d18
  let halfPi := Float.ofBits 0x3ff921fb54442d18
  if re == 0 then
    if im == 0 then 0 else if im < 0 then -halfPi else halfPi
  else if (im / re).isInf then 0
  else
    let base := atan (im / re)
    if re < 0 then if im < 0 then base - pi else base + pi else base

def complexSqrtArgumentWithAtan (atan : Float → Float) (re im : Float) : Float :=
  let angle := complexRawArgumentWithAtan atan re im
  if angle.abs < Float.ofBits 0x0010000000000000 then 0 else angle

-- Float.atan supplies a reference instantiation for the exact boundary examples;
-- production binds the parameter to the characterized reciprocal-reduced FPATAN substrate.
def complexSqrtArgument := complexSqrtArgumentWithAtan Float.atan

def complexSqrtWithTrig (atan cos sin : Float → Float) (re im : Float) : Float × Float :=
  let radius := complexSqrtRadius re im
  let angle := complexSqrtArgumentWithAtan atan re im / 2
  (radius * cos angle, radius * sin angle)

/-- IMARGUMENT publishes errors at the boundaries where IMSQRT uses angle zero. -/
def complexArgumentWithAtan (atan : Float → Float) (re im : Float) : Except WorksheetErrorCode Float :=
  if (re == 0 && im == 0) || (re != 0 && (im / re).isInf) then .error .div0
  else
    let angle := complexSqrtArgumentWithAtan atan re im
    .ok (if angle.abs < Float.ofBits 0x0010000000000000 then 0 else angle)

def complexArgument := complexArgumentWithAtan Float.atan

theorem complex_argument_ratio_overflow_is_div0 :
    (match complexArgument (Float.ofBits 0x01a56e1fc2f8f359) 1000000000 with
      | .error .div0 => true | _ => false) = true := by native_decide

theorem complex_argument_zero_is_div0 :
    (match complexArgument 0 0 with | .error .div0 => true | _ => false) = true := by native_decide

theorem complex_sqrt_term_and_sum_boundary_bindings :
    (complexSqrtSquaredTerm (Float.ofBits 0x1eb67e9c127b6e74)).toBits = 0
    ∧ (complexSqrtRadius (Float.ofBits 0x5fe7dddf6b095ff1)
        (Float.ofBits 0x5fe7dddf6b095ff1)).toBits = 0
    ∧ (complexSqrtRadius (Float.ofBits 0x5ff01c2a01d98729) 0).isInf = true := by
  native_decide

theorem complex_sqrt_signed_zero_principal_argument :
    (complexSqrtArgument (-1) (Float.ofBits 0x8000000000000000)).toBits =
      0x400921fb54442d18 := by
  native_decide

theorem complex_sqrt_ratio_overflow_argument :
    (complexSqrtArgument (Float.ofBits 0x01a56e1fc2f8f359) 1000000000).toBits = 0
    ∧ (complexSqrtArgument (-Float.ofBits 0x01a56e1fc2f8f359) (-1000000000)).toBits = 0 := by
  native_decide

/-- COMPLEX's numeric coefficient arguments use shared numeric text coercion;
complex-number strings in the IM family have a separate grammar. -/
def complexRealArgument : CoercionInput → Except CoercionError Rat
  | .number n => .ok n
  | .text text => coerceToNumber (.text text)
  | .logical _ => .error (.worksheetError .value)
  | .error code => .error (.worksheetError code)
  | .missingArg | .emptyCell => .ok 0

theorem complex_numeric_text_shares_decimal_coercion (text : String) :
    complexRealArgument (.text text) = coerceToNumber (.text text) := by rfl

/-- The sine/cosine graph retains cancellation in the exponential half-difference.
The scalar kernels and rounded reciprocal are explicit binding parameters, not a claim that Lean's
platform transcendentals duplicate the characterized Rust backends. -/
def complexSinWithKernels (sin cos exp recip : Float → Float) (re im : Float) : Float × Float :=
  let positive := exp im
  let negative := recip positive
  (sin re * ((positive + negative) / 2), cos re * ((positive - negative) / 2))

def complexCosWithKernels (sin cos exp recip : Float → Float) (re im : Float) : Float × Float :=
  let positive := exp im
  let negative := recip positive
  (cos re * ((positive + negative) / 2), -sin re * ((positive - negative) / 2))

theorem complex_trig_exponential_cancellation_binding :
    (complexSinWithKernels (fun _ => 0.5) (fun _ => 0.75) (fun _ => 1) (fun _ => 1) 1 0).2.toBits = 0
    ∧ (complexCosWithKernels (fun _ => 0.5) (fun _ => 0.75) (fun _ => 1) (fun _ => 1) 1 0).1.toBits =
      0x3fe8000000000000 := by native_decide

end OxFunc.Functions
