import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def varMeta : FunctionMeta := {
  functionId := "FUNC.VAR", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem varMeta_profiles :
    varMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ varMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ varMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [varMeta]
end OxFunc.Functions
