import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def aveDevMeta : FunctionMeta := {
  functionId := "FUNC.AVEDEV", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem aveDevMeta_profiles :
    aveDevMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ aveDevMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ aveDevMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [aveDevMeta]
end OxFunc.Functions
