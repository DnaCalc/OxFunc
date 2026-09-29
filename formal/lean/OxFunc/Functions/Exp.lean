import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def expMeta : FunctionMeta := {
  functionId := "FUNC.EXP"
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

def evalExpSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalExp_numeric_text_admitted :
    evalExpSurfaceClass (.text "1") = .ok "number" := by
  have parsed : parseSimpleNumber "1" = some 1 := by native_decide
  simp [evalExpSurfaceClass, coerceToNumber, parsed]

theorem expMeta_profiles :
    expMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ expMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [expMeta]

end OxFunc.Functions
