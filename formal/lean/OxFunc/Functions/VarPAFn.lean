import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def varPAMeta : FunctionMeta := {
  functionId := "FUNC.VARPA", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem varPAMeta_profiles :
    varPAMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ varPAMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ varPAMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [varPAMeta]
end OxFunc.Functions
