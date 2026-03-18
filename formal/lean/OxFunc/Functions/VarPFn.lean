import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def legacyVarPopMeta : FunctionMeta := {
  functionId := "FUNC.VARP", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem legacyVarPopMeta_profiles :
    legacyVarPopMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ legacyVarPopMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ legacyVarPopMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [legacyVarPopMeta]
end OxFunc.Functions
