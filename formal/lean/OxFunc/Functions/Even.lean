import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def evenMeta : FunctionMeta := {
  functionId := "FUNC.EVEN"
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

def evalEvenSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalEven_numeric_text_admitted :
    evalEvenSurfaceClass (.text "1") = .ok "number" := by
  simp [evalEvenSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem evenMeta_profiles :
    evenMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ evenMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [evenMeta]

end OxFunc.Functions
