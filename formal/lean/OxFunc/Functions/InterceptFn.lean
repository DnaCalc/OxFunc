import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def interceptMeta : FunctionMeta := {
  functionId := "FUNC.INTERCEPT", arity := { min := 2, max := 2 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem interceptMeta_profiles :
    interceptMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ interceptMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ interceptMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [interceptMeta]
end OxFunc.Functions
