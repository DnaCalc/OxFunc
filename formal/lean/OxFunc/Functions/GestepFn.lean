import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def gestepMeta : FunctionMeta := {
  functionId := "FUNC.GESTEP", arity := { min := 1, max := 2 }, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .custom, kernelSignatureClass := .numsToNum, fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
end OxFunc.Functions
