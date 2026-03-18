import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def stdevPCompatMeta : FunctionMeta := {
  functionId := "FUNC.STDEVP", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem stdevPCompatMeta_profiles :
    stdevPCompatMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ stdevPCompatMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ stdevPCompatMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [stdevPCompatMeta]
end OxFunc.Functions
