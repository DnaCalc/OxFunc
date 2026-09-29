import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptQuotient [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def quotientMeta : FunctionMeta := {
  functionId := "FUNC.QUOTIENT"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.numsToNum
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def coerceQuotientNumber : CoercionInput → Except CoercionError Rat
  | .logical _ => .error (.worksheetError .value)
  | .missingArg => .error (.worksheetError .na)
  | .emptyCell => .ok 0
  | other => coerceToNumber other

/-- Original reference origin and resolved extent remain visible to the local
adapter. Array values and unit references retain their ordinary value policy. -/
def quotientRejectReference (isReference : Bool) (cellCount : Nat) :
    Except WorksheetErrorCode Unit :=
  if isReference && cellCount > 1 then .error .value else .ok ()

theorem quotient_reference_origin_binding :
    quotientRejectReference true 2 = .error .value ∧
    quotientRejectReference true 1 = .ok () ∧
    quotientRejectReference false 6 = .ok () := by native_decide

def evalQuotientSurfaceClass (x y : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceQuotientNumber x with
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value
  | .ok _ => match coerceQuotientNumber y with
    | .error (.worksheetError code) => .error code
    | .error _ => .error .value
    | .ok 0 => .error .div0
    | .ok _ => .ok "number"

-- Binding: raw is the binary64 quotient after truncation toward zero.
-- Nonfinite division is #NUM!; integer zero has the positive-zero encoding.
def publishQuotientBits (raw : UInt64) : Except WorksheetErrorCode UInt64 :=
  if raw &&& 0x7FF0000000000000 = 0x7FF0000000000000 then .error .num
  else if raw &&& 0x7FFFFFFFFFFFFFFF = 0 then .ok 0
  else .ok raw

theorem quotientPublication_signed_zero_and_overflow :
    publishQuotientBits 0x8000000000000000 = .ok 0
    ∧ publishQuotientBits 0 = .ok 0
    ∧ publishQuotientBits 0x3FF0000000000000 = .ok 0x3FF0000000000000
    ∧ publishQuotientBits 0x7FF0000000000000 = .error .num
    ∧ publishQuotientBits 0xFFF0000000000000 = .error .num := by
  native_decide

theorem evalQuotient_zero_divisor_is_div0 :
    evalQuotientSurfaceClass (.number 1) (.number 0) = .error .div0 := by
  native_decide

theorem quotientMeta_profiles :
    quotientMeta.kernelSignatureClass = KernelSignatureClass.numsToNum
    ∧ quotientMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ quotientMeta.coercionLiftProfile = CoercionLiftProfile.custom := by
  simp [quotientMeta]

end OxFunc.Functions
