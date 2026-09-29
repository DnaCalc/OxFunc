import OxFunc.FunctionCore
import OxFunc.AggregatePublication
namespace OxFunc.Functions
open OxFunc

abbrev harMeanNumericBinding := AggregatePublication.harmean
abbrev harMeanPreparedBinding := AggregatePublication.harmeanPrepared
def harMeanMeta : FunctionMeta := {
  functionId := "FUNC.HARMEAN", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem harMeanMeta_profiles :
    harMeanMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ harMeanMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ harMeanMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [harMeanMeta]
end OxFunc.Functions
