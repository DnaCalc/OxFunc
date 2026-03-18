import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def orMeta : FunctionMeta := {
  functionId := "FUNC.OR"
  arity := { min := 1, max := 255 }
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

theorem orMeta_profiles :
    orMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ orMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ orMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [orMeta]

end OxFunc.Functions
