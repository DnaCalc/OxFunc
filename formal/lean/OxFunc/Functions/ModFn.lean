import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.RemainderPublication
import OxFunc.ElementaryPrepared

namespace OxFunc.Functions

open OxFunc

abbrev modNumericBinding := RemainderPublication.modWithPrimitive
abbrev modPreparedPairBinding := ElementaryPrepared.orderedPair

private instance instDecidableEqExceptMod [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def modMeta : FunctionMeta := {
  functionId := "FUNC.MOD"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.numsToNum
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

/-- Bounded scalar coercion/zero-divisor class. Numeric-domain publication is
bound separately by modNumericBinding. Empty and explicit Missing become zero;
coercion errors are left-first, as observed in the retained typed packet. -/
private def modPreparedScalar (x : CoercionInput) : Except WorksheetErrorCode Rat :=
  match x with
  | .emptyCell | .missingArg => .ok 0
  | _ => match coerceToNumber x with
    | .ok value => .ok value
    | .error (.worksheetError code) => .error code
    | .error _ => .error .value

def evalModSurfaceClass (x y : CoercionInput) : Except WorksheetErrorCode String := do
  let _ ← modPreparedScalar x
  let divisor ← modPreparedScalar y
  if divisor == 0 then .error .div0 else .ok "number"

theorem evalMod_zero_divisor_is_div0 :
    evalModSurfaceClass (.number 3) (.number 0) = .error .div0 := by
  native_decide

theorem evalMod_left_coercion_error_precedes_right_explicit_error :
    evalModSurfaceClass (.text "x") (.error .ref) = .error .value ∧
    evalModSurfaceClass (.error .ref) (.text "x") = .error .ref := by native_decide

theorem evalMod_empty_prepared_as_zero :
    evalModSurfaceClass .emptyCell (.number 2) = .ok "number" ∧
    evalModSurfaceClass (.number 2) .emptyCell = .error .div0 := by native_decide

theorem evalMod_missing_prepared_as_zero :
    evalModSurfaceClass .missingArg (.number 2) = .ok "number" ∧
    evalModSurfaceClass (.number 2) .missingArg = .error .div0 ∧
    evalModSurfaceClass .missingArg (.error .ref) = .error .ref := by native_decide

theorem modPreparedPair_padding_preserves_argument_order :
    modPreparedPairBinding (some (.error .value)) none = .error (.worksheetError .value) ∧
    modPreparedPairBinding none (some (.error .value)) = .error (.worksheetError .na) := by
  constructor <;> rfl

theorem modMeta_profiles :
    modMeta.kernelSignatureClass = KernelSignatureClass.numsToNum
    ∧ modMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ modMeta.coercionLiftProfile = CoercionLiftProfile.custom := by
  simp [modMeta]

end OxFunc.Functions
