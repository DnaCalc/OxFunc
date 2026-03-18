import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def pearsonMeta : FunctionMeta := {
  functionId := "FUNC.PEARSON", arity := { min := 2, max := 2 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem pearsonMeta_profiles :
    pearsonMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ pearsonMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ pearsonMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [pearsonMeta]
end OxFunc.Functions
