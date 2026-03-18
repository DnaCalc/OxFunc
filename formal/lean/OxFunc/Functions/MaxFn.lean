import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def maxMeta : FunctionMeta := {
  functionId := "FUNC.MAX"
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

theorem maxMeta_profiles :
    maxMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ maxMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ maxMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [maxMeta]

end OxFunc.Functions
