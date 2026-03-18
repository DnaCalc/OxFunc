import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptAtanh [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def atanhMeta : FunctionMeta := {
  functionId := "FUNC.ATANH"
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

def evalAtanhSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok n => if n ≤ (-1 : Rat) ∨ 1 ≤ n then .error .num else .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalAtanh_abs_one_is_num :
    evalAtanhSurfaceClass (.number 1) = .error .num := by
  native_decide

theorem atanhMeta_profiles :
    atanhMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ atanhMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
    ∧ atanhMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [atanhMeta]

end OxFunc.Functions
