import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def notMeta : FunctionMeta := {
  functionId := "FUNC.NOT"
  arity := Arity.exact 1
  determinism := .deterministic
  volatility := .nonvolatile
  hostInteraction := .none
  threadSafety := .safePure
  argPreparationProfile := .valuesOnlyPreAdapter
  coercionLiftProfile := .custom
  kernelSignatureClass := .custom
  fecDependencyProfile := .none
  surfaceFecDependencyProfile := .refOnly
}

theorem notMeta_profiles :
    notMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ notMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ notMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [notMeta]

end OxFunc.Functions
