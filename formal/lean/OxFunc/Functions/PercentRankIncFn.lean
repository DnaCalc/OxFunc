import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def percentRankIncMeta : FunctionMeta := {
  functionId := "FUNC.PERCENTRANK.INC", arity := { min := 2, max := 3 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem percentRankIncMeta_profiles :
    percentRankIncMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ percentRankIncMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ percentRankIncMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [percentRankIncMeta]
end OxFunc.Functions
