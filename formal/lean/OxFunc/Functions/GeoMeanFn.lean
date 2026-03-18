import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def geoMeanMeta : FunctionMeta := {
  functionId := "FUNC.GEOMEAN", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem geoMeanMeta_profiles :
    geoMeanMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ geoMeanMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ geoMeanMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [geoMeanMeta]
end OxFunc.Functions
