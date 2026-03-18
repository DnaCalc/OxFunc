import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def cosMeta : FunctionMeta := {
  functionId := "FUNC.COS"
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

def evalCosSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalCos_numeric_text_admitted :
    evalCosSurfaceClass (.text "1") = .ok "number" := by
  simp [evalCosSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem cosMeta_profiles :
    cosMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ cosMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [cosMeta]

end OxFunc.Functions
