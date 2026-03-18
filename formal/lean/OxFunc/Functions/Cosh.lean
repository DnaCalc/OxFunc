import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def coshMeta : FunctionMeta := {
  functionId := "FUNC.COSH"
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

def evalCoshSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalCosh_numeric_text_admitted :
    evalCoshSurfaceClass (.text "1") = .ok "number" := by
  simp [evalCoshSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem coshMeta_profiles :
    coshMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ coshMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [coshMeta]

end OxFunc.Functions
