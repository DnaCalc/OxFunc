import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def percentRankExcMeta : FunctionMeta := {
  functionId := "FUNC.PERCENTRANK.EXC", arity := { min := 2, max := 3 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem percentRankExcMeta_profiles :
    percentRankExcMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ percentRankExcMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ percentRankExcMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [percentRankExcMeta]
end OxFunc.Functions
