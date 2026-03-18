import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def stdevSMeta : FunctionMeta := {
  functionId := "FUNC.STDEV.S", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem stdevSMeta_profiles :
    stdevSMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ stdevSMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ stdevSMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [stdevSMeta]
end OxFunc.Functions
