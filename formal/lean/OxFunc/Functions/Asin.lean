import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptAsin [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def asinMeta : FunctionMeta := {
  functionId := "FUNC.ASIN"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def asinInDomain (n : Rat) : Bool := decide ((-1 : Rat) ≤ n ∧ n ≤ (1 : Rat))

def evalAsinSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok n => if asinInDomain n then .ok "number" else .error .num
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

def evalAsinLiftClass (inputs : List CoercionInput) : List (Except WorksheetErrorCode String) :=
  inputs.map evalAsinSurfaceClass

theorem evalAsin_numeric_text_admitted :
    evalAsinSurfaceClass (.text "1") = .ok "number" := by
  native_decide

theorem evalAsin_domain_error_num :
    evalAsinSurfaceClass (.number 2) = .error .num := by
  native_decide

theorem evalAsin_array_domain_element_errors :
    evalAsinLiftClass [.number 0, .number 2] = [.ok "number", .error .num] := by
  native_decide

theorem asinMeta_profiles :
    asinMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ asinMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
    ∧ asinMeta.fecDependencyProfile = FecDependencyProfile.none
    ∧ asinMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [asinMeta]

end OxFunc.Functions
