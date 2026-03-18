import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def rankAvgMeta : FunctionMeta := {
  functionId := "FUNC.RANK.AVG", arity := { min := 2, max := 3 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem rankAvgMeta_profiles :
    rankAvgMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ rankAvgMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ rankAvgMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [rankAvgMeta]
end OxFunc.Functions
