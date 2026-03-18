import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def largeMeta : FunctionMeta := {
  functionId := "FUNC.LARGE", arity := Arity.exact 2, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem largeMeta_profiles :
    largeMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ largeMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ largeMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [largeMeta]
end OxFunc.Functions
