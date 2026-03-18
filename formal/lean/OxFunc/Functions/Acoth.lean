import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptAcoth [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def acothMeta : FunctionMeta := {
  functionId := "FUNC.ACOTH"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalAcothSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok n => if n ≤ 1 ∧ (-1 : Rat) ≤ n then .error .num else .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalAcoth_abs_one_is_num :
    evalAcothSurfaceClass (.number 1) = .error .num := by
  native_decide

theorem acothMeta_profiles :
    acothMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ acothMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
    ∧ acothMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [acothMeta]

end OxFunc.Functions
