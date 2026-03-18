import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def fisherMeta : FunctionMeta := {
  functionId := "FUNC.FISHER", arity := { min := 1, max := 1 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .unaryNumericScalarOrArrayElementwise, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem fisherMeta_profiles :
    fisherMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ fisherMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
    ∧ fisherMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [fisherMeta]
end OxFunc.Functions
