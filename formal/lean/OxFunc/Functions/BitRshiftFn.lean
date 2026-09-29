import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.Functions.Bitwise
namespace OxFunc.Functions
open OxFunc
def bitRshiftKernelModel (number shift : Rat) := bitShiftKernelModel number shift false
def bitRshiftMeta : FunctionMeta := {
  functionId := "FUNC.BITRSHIFT", arity := Arity.exact 2, determinism := .deterministic,
  volatility := .nonvolatile, hostInteraction := .none, threadSafety := .safePure,
  argPreparationProfile := .valuesOnlyPreAdapter, coercionLiftProfile := .unaryNumericScalarOnly,
  kernelSignatureClass := .numsToNum, fecDependencyProfile := .none, surfaceFecDependencyProfile := .refOnly }
theorem bitRshiftMeta_profiles :
    bitRshiftMeta.kernelSignatureClass = KernelSignatureClass.numsToNum := by simp [bitRshiftMeta]
end OxFunc.Functions
