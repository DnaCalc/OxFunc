import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def secMeta : FunctionMeta := {
  functionId := "FUNC.SEC"
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

def evalSecSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalSec_numeric_text_admitted :
    evalSecSurfaceClass (.text "1") = .ok "number" := by
  simp [evalSecSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem secMeta_profiles :
    secMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ secMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [secMeta]

end OxFunc.Functions
