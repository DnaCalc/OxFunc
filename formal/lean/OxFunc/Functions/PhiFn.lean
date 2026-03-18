import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def phiMeta : FunctionMeta := {
  functionId := "FUNC.PHI", arity := { min := 1, max := 1 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .unaryNumericScalarOrArrayElementwise, kernelSignatureClass := .numToNum, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem phiMeta_profiles :
    phiMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ phiMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
    ∧ phiMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [phiMeta]
end OxFunc.Functions
