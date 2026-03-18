import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def varPCompatMeta : FunctionMeta := {
  functionId := "FUNC.VARP", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem varPCompatMeta_profiles :
    varPCompatMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ varPCompatMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ varPCompatMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [varPCompatMeta]
end OxFunc.Functions
