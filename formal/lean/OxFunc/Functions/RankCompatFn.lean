import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def rankCompatMeta : FunctionMeta := {
  functionId := "FUNC.RANK", arity := { min := 2, max := 3 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem rankCompatMeta_profiles :
    rankCompatMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ rankCompatMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ rankCompatMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [rankCompatMeta]
end OxFunc.Functions
