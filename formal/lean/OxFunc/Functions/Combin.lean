import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptCombin [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def combinMeta : FunctionMeta := {
  functionId := "FUNC.COMBIN"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.unaryNumericScalarOnly
  kernelSignatureClass := KernelSignatureClass.numsToNum
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalCombinSurfaceClass (x y : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber x, coerceToNumber y with
  | .ok n, .ok k =>
      if n < 0 ∨ k < 0 ∨ n < k then .error .num else .ok "number"
  | .error (.worksheetError code), _ => .error code
  | _, .error (.worksheetError code) => .error code
  | .error _, _ => .error .value
  | _, .error _ => .error .value

theorem evalCombin_overflow_count_is_num :
    evalCombinSurfaceClass (.number 5) (.number 6) = .error .num := by
  native_decide

theorem combinMeta_profiles :
    combinMeta.kernelSignatureClass = KernelSignatureClass.numsToNum
    ∧ combinMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [combinMeta]

end OxFunc.Functions
