import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.NumericPublication

namespace OxFunc.Functions

open OxFunc
abbrev radiansExecutable := NumericPublication.radians

def radiansMeta : FunctionMeta := {
  functionId := "FUNC.RADIANS"
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

def evalRadiansSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalRadians_numeric_text_admitted :
    evalRadiansSurfaceClass (.text "1") = .ok "number" := by
  have parsed : parseSimpleNumber "1" = some 1 := by native_decide
  simp [evalRadiansSurfaceClass, coerceToNumber, parsed]

theorem radiansMeta_profiles :
    radiansMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ radiansMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [radiansMeta]

end OxFunc.Functions
