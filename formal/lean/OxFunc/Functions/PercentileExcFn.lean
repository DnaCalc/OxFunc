import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def percentileExcMeta : FunctionMeta := {
  functionId := "FUNC.PERCENTILE.EXC", arity := Arity.exact 2, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem percentileExcMeta_profiles :
    percentileExcMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ percentileExcMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ percentileExcMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [percentileExcMeta]
end OxFunc.Functions
