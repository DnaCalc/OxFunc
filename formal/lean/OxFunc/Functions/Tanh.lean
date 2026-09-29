import OxFunc.HyperbolicComposition
import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def tanhMeta : FunctionMeta := {
  functionId := "FUNC.TANH"
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

def evalTanhSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

theorem evalTanh_numeric_text_admitted :
    evalTanhSurfaceClass (.text "1") = .ok "number" := by
  have parsed : parseSimpleNumber "1" = some 1 := by native_decide
  simp [evalTanhSurfaceClass, coerceToNumber, parsed]

theorem tanhMeta_profiles :
    tanhMeta.kernelSignatureClass = KernelSignatureClass.numToNum
    ∧ tanhMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [tanhMeta]

abbrev tanhCompositionBinding := OxFunc.HyperbolicComposition.tanh

end OxFunc.Functions
