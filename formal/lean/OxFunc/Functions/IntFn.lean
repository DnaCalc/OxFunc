import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.IntegerPreparation

namespace OxFunc.Functions

open OxFunc
abbrev intExecutable := IntegerPreparation.intKernel

def intMeta : FunctionMeta := {
  functionId := "FUNC.INT"
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

def evalIntSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalInt_numeric_text_admitted :
    evalIntSurfaceClass (.text "1") = .ok "number" := by
  have parsed : parseSimpleNumber "1" = some 1 := by native_decide
  simp [evalIntSurfaceClass, coerceToNumber, parsed]

theorem intMeta_profiles :
    intMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ intMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [intMeta]

end OxFunc.Functions
