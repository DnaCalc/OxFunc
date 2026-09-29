import OxFunc.IntegerPreparation
import OxFunc.DecimalTextConversion
import OxFunc.CoercionPrimitives

namespace OxFunc.DecimalRounding

inductive Mode where
  | nearest | down | up
  deriving BEq, DecidableEq, Repr

-- Function-local scalar preparation. Absent coordinates are represented by
-- none and become NA at their own position, before left-to-right coercion.
def coercePreparedNumber : CoercionInput → Except CoercionError Rat
  | .missingArg | .emptyCell => .ok 0
  | other => coerceToNumber other

def preparedPair (left right : Option CoercionInput) : Except CoercionError (Rat × Rat) := do
  let lhs ← coercePreparedNumber (left.getD (.error .na))
  let rhs ← coercePreparedNumber (right.getD (.error .na))
  pure (lhs, rhs)

theorem prepared_missing_zero :
    coercePreparedNumber .missingArg = .ok 0 ∧ coercePreparedNumber .emptyCell = .ok 0 := by
  constructor <;> rfl

theorem prepared_error_before_later_padding :
    preparedPair (some (.error .div0)) none = .error (.worksheetError .div0) := by rfl

theorem prepared_missing_before_later_error :
    preparedPair (some .missingArg) (some (.error .num)) = .error (.worksheetError .num) := by rfl

theorem prepared_earlier_padding_before_present_error :
    preparedPair none (some (.error .div0)) = .error (.worksheetError .na) := by rfl

def signedCount (value : Float) (limit : Nat) (mask : Nat) : Int :=
  let q := IntegerPreparation.binary64Magnitude value
  let magnitude := min (q.num.natAbs / q.den) limit % mask
  if value < 0 then -(magnitude : Int) else magnitude

def roundCount (value : Float) : Int := signedCount value 2147483647 4294967296
def directedCount (value : Float) : Int := signedCount value 4294967295 65536

def wrap32 (value : Int) : Int :=
  let bits := value % 4294967296
  if bits ≥ 2147483648 then bits - 4294967296 else bits

def decimalValue (significand : Nat) (scale : Int) : Float :=
  if significand = 0 || scale < -323 then 0
  else if scale > 308 then Float.ofBits 0x7ff0000000000000
  else Float.ofBits ((decimalTextBinary64Bits significand scale).getD 0x7ff0000000000000)

-- Function-local initial normalization policy. The shared decimal text parser
-- continues to publish zero for this exponent; its semantics are unchanged.
def initialDirectedSubnormal (significand : Nat) (scale : Int) : Option Float :=
  if significand = 0 then none else
  let value := decimalNarrowDyadic (decimalScaleStages significand scale) 53
  let shift := 53 - (value.significand.log2 + 1)
  let mantissa := value.significand * 2 ^ shift
  let exponent := value.exponent - Int.ofNat shift + 52
  if exponent = -1023 then some (Float.ofBits (UInt64.ofNat (mantissa % 2 ^ 52))) else none

-- Executable binding of the finite normal/zero runtime candidate. The retained
-- exceptional initial15 precision and endpoint observations remain open lanes.
def kernel (mode : Mode) (number : Float) (digits : Int) : Float :=
  if number == 0 || number.abs < Float.ofBits 0x0010000000000000 then 0 else
  let magnitude := IntegerPreparation.binary64Magnitude number
  let exponent := numericTextExponent 650 magnitude 0
  let pair := if mode == .nearest then numericTextRuntimeInitialDecimal magnitude exponent
    else IntegerPreparation.initial15Away magnitude exponent
  let initialSubnormal := if mode == .nearest then none else initialDirectedSubnormal pair.digits pair.scale
  if let some initial := initialSubnormal then
    if number < 0 then -initial else initial
  else
  let decimalDigits := if digits.natAbs > 32767 then (if digits < 0 then -100 else 100) else digits
  let dropped := if mode == .nearest then 15 - wrap32 (digits + pair.scale + 15)
    else -(pair.scale + decimalDigits)
  let result := if dropped ≤ 0 then decimalValue pair.digits pair.scale else
    let divisor := 10 ^ (min dropped.toNat 32)
    let quotient := pair.digits / divisor
    let remainder := pair.digits % divisor
    let scale := pair.scale + dropped
    match mode with
    | .nearest => decimalValue (quotient + if 2 * remainder ≥ divisor then 1 else 0) scale
    | .down => decimalValue quotient scale
    | .up =>
      let down := decimalValue quotient scale
      let initial := decimalValue pair.digits pair.scale
      let difference := if number < 0 then -initial - (-down) else initial - down
      down + if remainder = 0 || difference.toBits >>> 48 == 0 then 0 else decimalValue 1 (-digits)
  if result == 0 then 0 else if number < 0 then -result else result

def eval (mode : Mode) (number count : Float) : Option Float :=
  if !number.isFinite || !count.isFinite then none else
  let result := kernel mode number (if mode == .nearest then roundCount count else directedCount count)
  if result.isFinite then some result else none

def evalBits (mode : Mode) (number count : Float) : Option UInt64 :=
  (eval mode number count).map Float.toBits

theorem count_widths_and_fallback :
    directedCount 65537 = 1 ∧ directedCount (-65537) = -1
    ∧ directedCount 1.0e100 = 65535 ∧ roundCount 1.0e100 = 2147483647
    ∧ roundCount (-1.0e100) = -2147483647 := by native_decide

theorem directed_large_count_boundaries :
    evalBits .down 1.234 65537 = some (1.2 : Float).toBits
    ∧ evalBits .down 1.125e-101 32768 = some 0
    ∧ evalBits .down 1.125e100 (-32768) = some (1.0e100 : Float).toBits
    ∧ evalBits .up 1.125e100 (-32768) = none := by native_decide

theorem round_decimal_point_wrap :
    evalBits .nearest 0.5 1.0e100 = some (0.5 : Float).toBits
    ∧ evalBits .nearest 1.5 1.0e100 = some 0
    ∧ evalBits .nearest 0.05 (-1.0e100) = some 0
    ∧ evalBits .nearest 0.005 (-1.0e100) = some (0.005 : Float).toBits := by native_decide

theorem directed_decimal_assembly :
    evalBits .up 1.35 1 = some 0x3ff6666666666667
    ∧ evalBits .up 1.0e-200 (-126) = some 0x5a17a2ecc414a040 := by native_decide

theorem directed_initial_subnormal_publication :
    evalBits .down (Float.ofBits 0x0010000000000000) 0 = some 0x000ffffffffffffa
    ∧ evalBits .up (Float.ofBits 0x8010000000000007) (-308) = some 0x800ffffffffffffa
    ∧ evalBits .down (Float.ofBits 0x0010000000000008) 0 = some 0
    ∧ evalBits .nearest (Float.ofBits 0x0010000000000000) 0 = some 0
    ∧ evalBits .nearest (Float.ofBits 0x8010000000000000) 323 = some 0
    ∧ decimalTextBinary64Bits 222507385850720 (-322) = some 0 := by native_decide

theorem roundup_signed_tiny_residual :
    evalBits .up (Float.ofBits 0x0210be08d0527f0a) 305 = some 0x0210be08d0527e1d
    ∧ evalBits .up (Float.ofBits 0x8210be08d0527f0a) 305 = some 0x8210be08ec6943e2 := by native_decide

end OxFunc.DecimalRounding
