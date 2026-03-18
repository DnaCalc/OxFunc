import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def percentileIncMeta : FunctionMeta := {
  functionId := "FUNC.PERCENTILE.INC", arity := Arity.exact 2, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem percentileIncMeta_profiles :
    percentileIncMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ percentileIncMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ percentileIncMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [percentileIncMeta]
end OxFunc.Functions
