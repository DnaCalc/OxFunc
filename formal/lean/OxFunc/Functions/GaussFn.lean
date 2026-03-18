import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def gaussMeta : FunctionMeta := {
  functionId := "FUNC.GAUSS", arity := { min := 1, max := 1 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .unaryNumericScalarOrArrayElementwise, kernelSignatureClass := .numToNum, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem gaussMeta_profiles :
    gaussMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ gaussMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
    ∧ gaussMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [gaussMeta]
end OxFunc.Functions
