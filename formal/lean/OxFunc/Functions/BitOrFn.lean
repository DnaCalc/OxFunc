import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.Functions.Bitwise
namespace OxFunc.Functions
open OxFunc
def bitOrKernelModel := bitBinaryKernelModel .orOp
def bitOrMeta : FunctionMeta := {
  functionId := "FUNC.BITOR", arity := Arity.exact 2, determinism := .deterministic,
  volatility := .nonvolatile, hostInteraction := .none, threadSafety := .safePure,
  argPreparationProfile := .valuesOnlyPreAdapter, coercionLiftProfile := .unaryNumericScalarOnly,
  kernelSignatureClass := .numsToNum, fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem bitOrMeta_profiles :
    bitOrMeta.kernelSignatureClass = KernelSignatureClass.numsToNum := by simp [bitOrMeta]
end OxFunc.Functions
