import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.Functions.Bitwise
namespace OxFunc.Functions
open OxFunc
def bitAndKernelModel := bitBinaryKernelModel .andOp
def bitAndMeta : FunctionMeta := {
  functionId := "FUNC.BITAND", arity := Arity.exact 2, determinism := .deterministic,
  volatility := .nonvolatile, hostInteraction := .none, threadSafety := .safePure,
  argPreparationProfile := .valuesOnlyPreAdapter, coercionLiftProfile := .unaryNumericScalarOnly,
  kernelSignatureClass := .numsToNum, fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem bitAndMeta_profiles :
    bitAndMeta.kernelSignatureClass = KernelSignatureClass.numsToNum := by simp [bitAndMeta]
end OxFunc.Functions
