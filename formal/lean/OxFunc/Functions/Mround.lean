import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptMround [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def mroundMeta : FunctionMeta := {
  functionId := "FUNC.MROUND"
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

def evalMroundSurfaceClass (x y : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber x, coerceToNumber y with
  | .ok _, .ok 0 => .ok "number"
  | .ok n, .ok m =>
      if (0 ≤ n ∧ 0 ≤ m) ∨ (n ≤ 0 ∧ m ≤ 0) then .ok "number" else .error .num
  | .error (.worksheetError code), _ => .error code
  | _, .error (.worksheetError code) => .error code
  | .error _, _ => .error .value
  | _, .error _ => .error .value

theorem evalMround_sign_mismatch_is_num :
    evalMroundSurfaceClass (.number 10) (.number (-3)) = .error .num := by
  native_decide

theorem mroundMeta_profiles :
    mroundMeta.kernelSignatureClass = KernelSignatureClass.numsToNum
    ∧ mroundMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [mroundMeta]

end OxFunc.Functions
