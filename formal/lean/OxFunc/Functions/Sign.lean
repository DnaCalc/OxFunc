import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def signMeta : FunctionMeta := {
  functionId := "FUNC.SIGN"
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

def evalSignSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalSign_numeric_text_admitted :
    evalSignSurfaceClass (.text "1") = .ok "number" := by
  simp [evalSignSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem signMeta_profiles :
    signMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ signMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [signMeta]

end OxFunc.Functions
