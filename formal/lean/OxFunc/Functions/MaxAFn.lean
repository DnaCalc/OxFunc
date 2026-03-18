import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def maxAMeta : FunctionMeta := {
  functionId := "FUNC.MAXA"
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

theorem maxAMeta_profiles :
    maxAMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ maxAMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ maxAMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [maxAMeta]

end OxFunc.Functions
