import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def naMeta : FunctionMeta := {
  functionId := "FUNC.NA", arity := Arity.exact 0, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .none, kernelSignatureClass := .custom, fecDependencyProfile := .none, surfaceFecDependencyProfile := .none }
end OxFunc.Functions
