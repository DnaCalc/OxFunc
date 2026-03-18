import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def minMeta : FunctionMeta := {
  functionId := "FUNC.MIN"
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

theorem minMeta_profiles :
    minMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ minMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ minMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [minMeta]

end OxFunc.Functions
