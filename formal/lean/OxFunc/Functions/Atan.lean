import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def atanMeta : FunctionMeta := {
  functionId := "FUNC.ATAN"
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

def evalAtanSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalAtan_numeric_text_admitted :
    evalAtanSurfaceClass (.text "1") = .ok "number" := by
  simp [evalAtanSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem atanMeta_profiles :
    atanMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ atanMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [atanMeta]

end OxFunc.Functions
