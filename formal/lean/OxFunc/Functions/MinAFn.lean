import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def minAMeta : FunctionMeta := {
  functionId := "FUNC.MINA"
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

theorem minAMeta_profiles :
    minAMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ minAMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ minAMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [minAMeta]

end OxFunc.Functions
