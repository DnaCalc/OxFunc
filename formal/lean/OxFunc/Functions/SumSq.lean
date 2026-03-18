import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def sumsqMeta : FunctionMeta := {
  functionId := "FUNC.SUMSQ"
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

theorem sumsqMeta_profiles :
    sumsqMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ sumsqMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ sumsqMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [sumsqMeta]

end OxFunc.Functions
