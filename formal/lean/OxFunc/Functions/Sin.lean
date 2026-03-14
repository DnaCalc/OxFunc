import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptSin [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def sinMeta : FunctionMeta := {
  functionId := "FUNC.SIN"
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

def evalSinSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok _ => .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

def evalSinLiftClass (inputs : List CoercionInput) : List (Except WorksheetErrorCode String) :=
  inputs.map evalSinSurfaceClass

theorem evalSin_numeric_text_admitted :
    evalSinSurfaceClass (.text "1") = .ok "number" := by
  simp [evalSinSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem evalSin_bad_text_value :
    evalSinSurfaceClass (.text "asd") = .error .value := by
  simp [evalSinSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem evalSin_array_bad_text_element_errors :
    evalSinLiftClass [.number 1, .text "asd"] = [.ok "number", .error .value] := by
  simp [evalSinLiftClass, evalSinSurfaceClass, coerceToNumber, parseSimpleNumber]

theorem sinMeta_profiles :
    sinMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ sinMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
    ∧ sinMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ sinMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [sinMeta]

end OxFunc.Functions
