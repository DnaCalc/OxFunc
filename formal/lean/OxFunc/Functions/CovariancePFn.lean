import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def covariancePMeta : FunctionMeta := {
  functionId := "FUNC.COVARIANCE.P", arity := { min := 2, max := 2 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem covariancePMeta_profiles :
    covariancePMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ covariancePMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ covariancePMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [covariancePMeta]
end OxFunc.Functions
