import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def permutMeta : FunctionMeta := {
  functionId := "FUNC.PERMUT", arity := Arity.exact 2, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .unaryNumericScalarOnly, kernelSignatureClass := .numsToNum, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem permutMeta_profiles :
    permutMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ permutMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOnly
    ∧ permutMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [permutMeta]
end OxFunc.Functions
