import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def baseMeta : FunctionMeta := {
  functionId := "FUNC.BASE", arity := { min := 2, max := 3 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem baseMeta_profiles :
    baseMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ baseMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ baseMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [baseMeta]
end OxFunc.Functions
