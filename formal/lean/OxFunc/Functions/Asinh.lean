import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def asinhMeta : FunctionMeta := {
  functionId := "FUNC.ASINH"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
  kernelSignatureClass := KernelSignatureClass.numToNum
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalAsinhSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalAsinh_numeric_text_admitted :
    evalAsinhSurfaceClass (.text "1") = .ok "number" := by
  simp [evalAsinhSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem asinhMeta_profiles :
    asinhMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ asinhMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [asinhMeta]

end OxFunc.Functions
