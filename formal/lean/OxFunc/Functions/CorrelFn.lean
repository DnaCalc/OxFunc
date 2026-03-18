import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def correlMeta : FunctionMeta := {
  functionId := "FUNC.CORREL", arity := { min := 2, max := 2 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem correlMeta_profiles :
    correlMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ correlMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ correlMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [correlMeta]
end OxFunc.Functions
