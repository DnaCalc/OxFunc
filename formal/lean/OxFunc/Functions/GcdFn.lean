import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptGcd [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def gcdMeta : FunctionMeta := {
  functionId := "FUNC.GCD"
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

def evalGcdSurfaceClass (inputs : List CoercionInput) : Except WorksheetErrorCode String :=
  match inputs with
  | [] => .error .value
  | _ =>
      match inputs.map coerceToNumber |>.find? (fun r =>
        match r with
        | .ok n => n < 0
        | .error _ => False) with
      | some _ => .error .num
      | none => .ok "number"

theorem evalGcd_negative_item_is_num :
    evalGcdSurfaceClass [.number (-1), .number 5] = .error .num := by
  native_decide

theorem gcdMeta_profiles :
    gcdMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ gcdMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [gcdMeta]

end OxFunc.Functions
