import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def smallMeta : FunctionMeta := {
  functionId := "FUNC.SMALL", arity := Arity.exact 2, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem smallMeta_profiles :
    smallMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ smallMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ smallMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [smallMeta]
end OxFunc.Functions
