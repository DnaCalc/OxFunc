import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def varCompatMeta : FunctionMeta := {
  functionId := "FUNC.VAR", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem varCompatMeta_profiles :
    varCompatMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ varCompatMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ varCompatMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [varCompatMeta]
end OxFunc.Functions
