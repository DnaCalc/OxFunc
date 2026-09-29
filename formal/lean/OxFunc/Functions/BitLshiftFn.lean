import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.Functions.Bitwise
namespace OxFunc.Functions
open OxFunc
def bitLshiftKernelModel (number shift : Rat) := bitShiftKernelModel number shift true
def bitLshiftMeta : FunctionMeta := {
  functionId := "FUNC.BITLSHIFT", arity := Arity.exact 2, determinism := .deterministic,
  volatility := .nonvolatile, hostInteraction := .none, threadSafety := .safePure,
  argPreparationProfile := .valuesOnlyPreAdapter, coercionLiftProfile := .unaryNumericScalarOnly,
  kernelSignatureClass := .numsToNum, fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem bitLshiftMeta_profiles :
    bitLshiftMeta.kernelSignatureClass = KernelSignatureClass.numsToNum := by simp [bitLshiftMeta]
end OxFunc.Functions
