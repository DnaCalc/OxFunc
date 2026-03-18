import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def logMeta : FunctionMeta := {
  functionId := "FUNC.LOG", arity := { min := 1, max := 2 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem logMeta_profiles :
    logMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ logMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ logMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [logMeta]
end OxFunc.Functions
