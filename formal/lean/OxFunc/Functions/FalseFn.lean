import OxFunc.FunctionCore
namespace OxFunc.Functions
open OxFunc
def falseMeta : FunctionMeta := {
  functionId := "FUNC.FALSE", arity := Arity.exact 0, determinism := .deterministic, volatility := .nonvolatile,
  hostInteraction := .none, threadSafety := .safePure, argPreparationProfile := .valuesOnlyPreAdapter,
  coercionLiftProfile := .none, kernelSignatureClass := .custom, fecDependencyProfile := .none, surfaceFecDependencyProfile := .none }
end OxFunc.Functions
