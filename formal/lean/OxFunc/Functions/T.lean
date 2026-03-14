import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def tMeta : FunctionMeta := {
  functionId := "FUNC.T"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalTScalar : CoercionInput → Except WorksheetErrorCode String
  | .text s => .ok s
  | .error code => .error code
  | .number _ | .logical _ | .missingArg | .emptyCell => .ok ""

def evalTArray : List CoercionInput → List (Except WorksheetErrorCode String) :=
  List.map evalTScalar

theorem evalT_number_empty_string :
    evalTScalar (.number 42) = .ok "" := by
  rfl

theorem evalT_blank_scalar_empty_string :
    evalTScalar .emptyCell = .ok "" := by
  rfl

theorem evalT_array_preserves_text_and_errors :
    evalTArray [.text "x", .error .na, .logical true] = [.ok "x", .error .na, .ok ""] := by
  rfl

theorem tMeta_profiles :
    tMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ tMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ tMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [tMeta]

end OxFunc.Functions
