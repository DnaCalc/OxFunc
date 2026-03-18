import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def sechMeta : FunctionMeta := {
  functionId := "FUNC.SECH"
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

def evalSechSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalSech_numeric_text_admitted :
    evalSechSurfaceClass (.text "1") = .ok "number" := by
  simp [evalSechSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem sechMeta_profiles :
    sechMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ sechMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [sechMeta]

end OxFunc.Functions
