import OxFunc.PermutationPower
import OxFunc.FunctionCore
import OxFunc.Functions.DistributionArguments
namespace OxFunc.Functions
open OxFunc
def permutationAMeta : FunctionMeta := {
  functionId := "FUNC.PERMUTATIONA", arity := Arity.exact 2, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .numsToNum, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem permutationAMeta_profiles :
    permutationAMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ permutationAMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ permutationAMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [permutationAMeta]
abbrev permutationAPowerBinding := OxFunc.PermutationPower.kernel
abbrev permutationAPrepared := distributionNumericPrepared 2

theorem permutationA_missing_and_padding_binding :
    permutationAPrepared [.missingArg, .number 3] = .ok [0, 3] ∧
    permutationAPrepared [.number 3, .missingArg] = .ok [3, 0] ∧
    permutationAPrepared [.error .ref, .error .na] = .error .ref ∧
    permutationAPrepared [.error .na, .error .ref] = .error .na := by
  exact ⟨rfl, rfl, rfl, rfl⟩

end OxFunc.Functions
