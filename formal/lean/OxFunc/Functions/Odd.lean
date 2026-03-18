import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def oddMeta : FunctionMeta := {
  functionId := "FUNC.ODD"
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

def evalOddSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalOdd_numeric_text_admitted :
    evalOddSurfaceClass (.text "1") = .ok "number" := by
  simp [evalOddSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem oddMeta_profiles :
    oddMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ oddMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [oddMeta]

end OxFunc.Functions
