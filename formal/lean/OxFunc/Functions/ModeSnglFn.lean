import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def modeSnglMeta : FunctionMeta := {
  functionId := "FUNC.MODE.SNGL", arity := { min := 1, max := 255 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .numsToNum, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem modeSnglMeta_profiles :
    modeSnglMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ modeSnglMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ modeSnglMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [modeSnglMeta]
end OxFunc.Functions
