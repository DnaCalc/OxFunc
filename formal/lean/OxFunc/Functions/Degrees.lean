import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def degreesMeta : FunctionMeta := {
  functionId := "FUNC.DEGREES"
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

def evalDegreesSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalDegrees_numeric_text_admitted :
    evalDegreesSurfaceClass (.text "1") = .ok "number" := by
  simp [evalDegreesSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem degreesMeta_profiles :
    degreesMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ degreesMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [degreesMeta]

end OxFunc.Functions
