import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.NumericPublication

/-- Executable publication rule observed for the W111 numerical boundary slice.
It is applied at the documented operation, rather than asserted for every
worksheet function or intermediate arithmetic node. -/
def flushTiny (x : Float) : Float :=
  if x.abs < Float.ofBits 0x0010000000000000 then 0 else x

def finiteOrNum (x : Float) : Except WorksheetErrorCode Float :=
  if x.isFinite then .ok (flushTiny x) else .error .num

/-- The square-root primitive is explicitly supplied: the Rust x86_64 binding
is FSQRT at PC64 followed by a binary64 store. Float.sqrt is not substituted for
that two-rounding primitive. Primitive identity remains empirical. -/
def sqrtWithPrimitive (stagedSqrt : Float → Float) (x : Float) :
    Except WorksheetErrorCode Float :=
  if x < 0 then .error .num else .ok (stagedSqrt x)

def radians (stagedMul : Float → Float → Float) (x : Float) : Float :=
  flushTiny (stagedMul x (Float.ofBits 0x400921fb54442d18 / 180))

def sechWithCosh (cosh stagedReciprocal : Float → Float) (x : Float) : Float :=
  flushTiny (stagedReciprocal (cosh x))

def standardize (x mean stdev : Float) : Except WorksheetErrorCode Float :=
  if stdev ≤ 0 then .error .num
  else
    let difference := x - mean
    if !difference.isFinite then .error .num
    else finiteOrNum (difference / stdev)

def densityWithPrimitives (stagedMul : Float → Float → Float)
    (density : Float → Float) (x : Float) : Except WorksheetErrorCode Float :=
  if (stagedMul x x).isFinite then .ok (density x) else .error .num

def exponentialWithPrimitives (stagedMul : Float → Float → Float)
    (exp expm1 : Float → Float) (x lambda : Float) (cumulative : Bool) :
    Except WorksheetErrorCode Float :=
  if !x.isFinite || !lambda.isFinite || x < 0 || lambda ≤ 0 then .error .num
  else
    let product := stagedMul lambda x
    if !product.isFinite then .error .num
    else .ok (flushTiny (if cumulative then -(expm1 (-product))
      else stagedMul lambda (exp (-product))))

theorem flush_positive_subnormal : (flushTiny (Float.ofBits 1)).toBits = 0 := by
  native_decide
theorem flush_negative_subnormal :
    (flushTiny (Float.ofBits 0x8000000000000001)).toBits = 0 := by
  native_decide
theorem preserve_minimum_normal :
    (flushTiny (Float.ofBits 0x0010000000000000)).toBits = 0x0010000000000000 := by
  native_decide
theorem standardize_tiny_output :
    (standardize (Float.ofBits 0x0010000000000000) 0 2).toOption.map Float.toBits = some 0 := by
  native_decide
theorem standardize_overflow_before_division :
    (standardize 1.0e308 (-1.0e308) 1.0e308).toOption = none := by
  native_decide
theorem sqrt_admission_binding (primitive : Float → Float) (x : Float) (h : ¬ x < 0) :
    sqrtWithPrimitive primitive x = .ok (primitive x) := by
  simp [sqrtWithPrimitive, h]
theorem density_overflow_rejected :
    (densityWithPrimitives (fun _ _ => Float.ofBits 0x7ff0000000000000)
      (fun _ => 0) 1).toOption = none := by
  native_decide
theorem exponential_overflow_rejected :
    (exponentialWithPrimitives (fun _ _ => Float.ofBits 0x7ff0000000000000)
      (fun _ => 0) (fun _ => 0) 1 1 false).toOption = none := by
  native_decide

end OxFunc.NumericPublication
