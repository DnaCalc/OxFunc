import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.Functions.Bitwise
namespace OxFunc.Functions
open OxFunc
def bitXorKernelModel := bitBinaryKernelModel .xorOp
def bitXorMeta : FunctionMeta := {
  functionId := "FUNC.BITXOR", arity := Arity.exact 2, determinism := .deterministic,
  volatility := .nonvolatile, hostInteraction := .none, threadSafety := .safePure,
  argPreparationProfile := .valuesOnlyPreAdapter, coercionLiftProfile := .unaryNumericScalarOnly,
  kernelSignatureClass := .numsToNum, fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem bitXorMeta_profiles :
    bitXorMeta.kernelSignatureClass = KernelSignatureClass.numsToNum := by simp [bitXorMeta]
end OxFunc.Functions
