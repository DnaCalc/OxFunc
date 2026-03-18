import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def varSMeta : FunctionMeta := {
  functionId := "FUNC.VAR.S", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem varSMeta_profiles :
    varSMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ varSMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ varSMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [varSMeta]
end OxFunc.Functions
