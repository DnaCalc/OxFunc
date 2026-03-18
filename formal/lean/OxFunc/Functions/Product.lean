import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def productMeta : FunctionMeta := {
  functionId := "FUNC.PRODUCT"
  arity := { min := 1, max := 255 }
  determinism := .deterministic
  volatility := .nonvolatile
  hostInteraction := .none
  threadSafety := .safePure
  argPreparationProfile := .valuesOnlyPreAdapter
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy
  kernelSignatureClass := .numsToNum
  fecDependencyProfile := .none
  surfaceFecDependencyProfile := .refOnly
}

theorem productMeta_profiles :
    productMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ productMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ productMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [productMeta]

end OxFunc.Functions
