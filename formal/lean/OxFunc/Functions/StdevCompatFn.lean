import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def stdevCompatMeta : FunctionMeta := {
  functionId := "FUNC.STDEV", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem stdevCompatMeta_profiles :
    stdevCompatMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ stdevCompatMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ stdevCompatMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [stdevCompatMeta]
end OxFunc.Functions
