import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptFactDouble [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def factDoubleMeta : FunctionMeta := {
  functionId := "FUNC.FACTDOUBLE"
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

def evalFactDoubleSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok n => if n < (-1 : Rat) then .error .num else .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalFactDouble_minus_two_is_num :
    evalFactDoubleSurfaceClass (.number (-2)) = .error .num := by
  native_decide

theorem factDoubleMeta_profiles :
    factDoubleMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ factDoubleMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
    ∧ factDoubleMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [factDoubleMeta]

end OxFunc.Functions
