import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def quartileIncMeta : FunctionMeta := {
  functionId := "FUNC.QUARTILE.INC", arity := Arity.exact 2, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem quartileIncMeta_profiles :
    quartileIncMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ quartileIncMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ quartileIncMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [quartileIncMeta]
end OxFunc.Functions
