import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def fisherInvMeta : FunctionMeta := {
  functionId := "FUNC.FISHERINV", arity := { min := 1, max := 1 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .unaryNumericScalarOrArrayElementwise, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem fisherInvMeta_profiles :
    fisherInvMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ fisherInvMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
    ∧ fisherInvMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [fisherInvMeta]
end OxFunc.Functions
