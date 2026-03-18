import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def averageAMeta : FunctionMeta := {
  functionId := "FUNC.AVERAGEA", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem averageAMeta_profiles :
    averageAMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ averageAMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ averageAMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [averageAMeta]
end OxFunc.Functions
