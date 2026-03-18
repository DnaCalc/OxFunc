import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def varAMeta : FunctionMeta := {
  functionId := "FUNC.VARA", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem varAMeta_profiles :
    varAMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ varAMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ varAMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [varAMeta]
end OxFunc.Functions
