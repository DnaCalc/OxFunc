import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def roundDownMeta : FunctionMeta := {
  functionId := "FUNC.ROUNDDOWN", arity := { min := 2, max := 2 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .numsToNum, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem roundDownMeta_profiles :
    roundDownMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ roundDownMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ roundDownMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [roundDownMeta]
end OxFunc.Functions
