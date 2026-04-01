import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def legacyStdevPopMeta : FunctionMeta := {
  functionId := "FUNC.STDEVP", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }

def stdevPopMeta : FunctionMeta := { legacyStdevPopMeta with functionId := "FUNC.STDEV.P" }

theorem legacyStdevPopMeta_profiles :
    legacyStdevPopMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ legacyStdevPopMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ legacyStdevPopMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [legacyStdevPopMeta]

theorem stdevPopMeta_profiles :
    stdevPopMeta.functionId = "FUNC.STDEV.P"
    ∧ stdevPopMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ stdevPopMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ stdevPopMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [stdevPopMeta, legacyStdevPopMeta]
end OxFunc.Functions
