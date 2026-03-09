import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def andMeta : FunctionMeta := {
  functionId := "FUNC.AND"
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

def evalAndAdapter : List CoercionInput → Except CoercionError Bool
  | [] => Except.ok true
  | x :: xs =>
      match coerceToNumber x, evalAndAdapter xs with
      | Except.ok n, Except.ok rest => Except.ok (n ≠ 0 ∧ rest)
      | Except.error e, _ => Except.error e
      | _, Except.error e => Except.error e

theorem evalAndAdapter_false_branch :
    evalAndAdapter [.logical true, .number 0] = Except.ok false := by
  simp [evalAndAdapter, coerceToNumber]

theorem andMeta_profiles :
    andMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ andMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [andMeta]

end OxFunc.Functions
