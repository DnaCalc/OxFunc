import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def devSqMeta : FunctionMeta := {
  functionId := "FUNC.DEVSQ", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .aggregateDirectAndRangeDualPolicy, kernelSignatureClass := .numsToNum,
  fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem devSqMeta_profiles :
    devSqMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ devSqMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ devSqMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [devSqMeta]
end OxFunc.Functions
