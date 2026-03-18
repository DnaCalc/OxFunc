import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def permutationAMeta : FunctionMeta := {
  functionId := "FUNC.PERMUTATIONA", arity := Arity.exact 2, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .unaryNumericScalarOnly, kernelSignatureClass := .numsToNum, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem permutationAMeta_profiles :
    permutationAMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ permutationAMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOnly
    ∧ permutationAMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [permutationAMeta]
end OxFunc.Functions
