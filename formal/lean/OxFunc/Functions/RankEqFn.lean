import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def rankEqMeta : FunctionMeta := {
  functionId := "FUNC.RANK.EQ", arity := { min := 2, max := 3 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem rankEqMeta_profiles :
    rankEqMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ rankEqMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ rankEqMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [rankEqMeta]
end OxFunc.Functions
