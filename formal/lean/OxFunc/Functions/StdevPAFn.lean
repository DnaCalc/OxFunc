import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def stdevPAMeta : FunctionMeta := {
  functionId := "FUNC.STDEVPA", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem stdevPAMeta_profiles :
    stdevPAMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ stdevPAMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ stdevPAMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [stdevPAMeta]
end OxFunc.Functions
