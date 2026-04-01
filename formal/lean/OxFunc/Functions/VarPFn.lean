import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def legacyVarPopMeta : FunctionMeta := {
  functionId := "FUNC.VARP", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }

def varPopMeta : FunctionMeta := { legacyVarPopMeta with functionId := "FUNC.VAR.P" }

theorem legacyVarPopMeta_profiles :
    legacyVarPopMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ legacyVarPopMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ legacyVarPopMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [legacyVarPopMeta]

theorem varPopMeta_profiles :
    varPopMeta.functionId = "FUNC.VAR.P"
    ∧ varPopMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ varPopMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ varPopMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [varPopMeta, legacyVarPopMeta]
end OxFunc.Functions
