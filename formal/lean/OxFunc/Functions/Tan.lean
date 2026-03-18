import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def tanMeta : FunctionMeta := {
  functionId := "FUNC.TAN"
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

def evalTanSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalTan_numeric_text_admitted :
    evalTanSurfaceClass (.text "1") = .ok "number" := by
  simp [evalTanSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem tanMeta_profiles :
    tanMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ tanMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [tanMeta]

end OxFunc.Functions
