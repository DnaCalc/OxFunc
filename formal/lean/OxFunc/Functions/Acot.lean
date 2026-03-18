import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def acotMeta : FunctionMeta := {
  functionId := "FUNC.ACOT"
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

def evalAcotSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalAcot_numeric_text_admitted :
    evalAcotSurfaceClass (.text "1") = .ok "number" := by
  simp [evalAcotSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem acotMeta_profiles :
    acotMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ acotMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [acotMeta]

end OxFunc.Functions
