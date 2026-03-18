import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptMultinomial [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def multinomialMeta : FunctionMeta := {
  functionId := "FUNC.MULTINOMIAL"
  arity := { min := 1, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalMultinomialSurfaceClass (inputs : List CoercionInput) : Except WorksheetErrorCode String :=
  if inputs.length = 0 then
    .error .value
  else
    match inputs.map coerceToNumber |>.find? (fun r =>
      match r with
      | .ok n => n < 0
      | .error _ => False) with
    | some _ => .error .num
    | none =>
        match inputs.map coerceToNumber |>.find? (fun r =>
          match r with
          | .error _ => True
          | .ok _ => False) with
        | some (.error (.worksheetError code)) => .error code
        | some _ => .error .value
        | none => .ok "number"

theorem evalMultinomial_negative_item_is_num :
    evalMultinomialSurfaceClass [.number 1, .number (-1), .number 2] = .error .num := by
  native_decide

theorem multinomialMeta_profiles :
    multinomialMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ multinomialMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [multinomialMeta]

end OxFunc.Functions
