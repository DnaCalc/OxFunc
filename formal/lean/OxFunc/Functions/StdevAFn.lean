import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def stdevAMeta : FunctionMeta := {
  functionId := "FUNC.STDEVA", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem stdevAMeta_profiles :
    stdevAMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ stdevAMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ stdevAMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [stdevAMeta]
end OxFunc.Functions
