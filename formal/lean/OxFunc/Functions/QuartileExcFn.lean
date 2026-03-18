import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def quartileExcMeta : FunctionMeta := {
  functionId := "FUNC.QUARTILE.EXC", arity := Arity.exact 2, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem quartileExcMeta_profiles :
    quartileExcMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ quartileExcMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ quartileExcMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [quartileExcMeta]
end OxFunc.Functions
