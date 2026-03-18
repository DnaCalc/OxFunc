import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def roundUpMeta : FunctionMeta := {
  functionId := "FUNC.ROUNDUP", arity := { min := 2, max := 2 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .numsToNum, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem roundUpMeta_profiles :
    roundUpMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ roundUpMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ roundUpMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [roundUpMeta]
end OxFunc.Functions
