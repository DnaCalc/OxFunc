import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def rsqMeta : FunctionMeta := {
  functionId := "FUNC.RSQ", arity := { min := 2, max := 2 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem rsqMeta_profiles :
    rsqMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ rsqMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ rsqMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [rsqMeta]
end OxFunc.Functions
