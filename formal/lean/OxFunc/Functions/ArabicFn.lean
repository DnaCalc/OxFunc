import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def arabicMeta : FunctionMeta := {
  functionId := "FUNC.ARABIC", arity := Arity.exact 1, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem arabicMeta_profiles :
    arabicMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ arabicMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ arabicMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [arabicMeta]
end OxFunc.Functions
