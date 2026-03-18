import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def xorMeta : FunctionMeta := {
  functionId := "FUNC.XOR"
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

theorem xorMeta_profiles :
    xorMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ xorMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ xorMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [xorMeta]

end OxFunc.Functions
