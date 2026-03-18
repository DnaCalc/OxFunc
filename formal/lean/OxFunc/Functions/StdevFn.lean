import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def stdevMeta : FunctionMeta := {
  functionId := "FUNC.STDEV", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem stdevMeta_profiles :
    stdevMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ stdevMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ stdevMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [stdevMeta]
end OxFunc.Functions
