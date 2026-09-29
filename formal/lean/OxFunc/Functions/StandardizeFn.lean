import OxFunc.FunctionCore
import OxFunc.NumericPublication
namespace OxFunc.Functions
open OxFunc
abbrev standardizeExecutable := NumericPublication.standardize
def standardizeMeta : FunctionMeta := {
  functionId := "FUNC.STANDARDIZE", arity := { min := 3, max := 3 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .custom, fecDependencyProfile := .none,
  surfaceFecDependencyProfile := .refOnly }
theorem standardizeMeta_profiles :
    standardizeMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ standardizeMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ standardizeMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [standardizeMeta]
end OxFunc.Functions
