import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def covarianceSMeta : FunctionMeta := {
  functionId := "FUNC.COVARIANCE.S", arity := { min := 2, max := 2 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem covarianceSMeta_profiles :
    covarianceSMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ covarianceSMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ covarianceSMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [covarianceSMeta]
end OxFunc.Functions
