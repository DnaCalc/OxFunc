import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def slopeMeta : FunctionMeta := {
  functionId := "FUNC.SLOPE", arity := { min := 2, max := 2 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem slopeMeta_profiles :
    slopeMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ slopeMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ slopeMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [slopeMeta]
end OxFunc.Functions
