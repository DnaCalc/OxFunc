import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptAnd [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def andMeta : FunctionMeta := {
  functionId := "FUNC.AND"
  arity := { min := 1, max := 255 }
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

inductive AndArgOrigin where
  | directScalar
  | arrayLike
  deriving DecidableEq, Repr

structure AndPreparedArg where
  origin : AndArgOrigin
  value : CoercionInput
  deriving DecidableEq, Repr

def andArgumentTruth : AndPreparedArg → Except WorksheetErrorCode (Option Bool)
  | ⟨_, .logical b⟩ => .ok (some b)
  | ⟨_, .number n⟩ => .ok (some (n ≠ 0))
  | ⟨_, .error code⟩ => .error code
  | ⟨.directScalar, .text _⟩ => .error .value
  | ⟨.arrayLike, .text _⟩
  | ⟨_, .missingArg⟩
  | ⟨_, .emptyCell⟩ => .ok none

/-- Mirrors Rust `eval_and_surface` (`crates/oxfunc_core/src/functions/and_fn.rs`). Excel
evaluates every argument of AND: once an item has decided the result (`decided`, a FALSE was
seen) the remaining items are still scanned, and an error VALUE among them surfaces — the first
error in argument order winning over any later one (live Excel 16.0 build 20326, COM probe
2026-09-15, bead `oxf-xvt5.14`: `=AND(FALSE,1/0)` -> `#DIV/0!`, `=AND(FALSE,NA(),1/0)` ->
`#N/A`). Before that bead the loop returned `.ok false` on the first FALSE. -/
def evalAndPrepared : List AndPreparedArg → Except WorksheetErrorCode Bool
  | args =>
      let rec loop : List AndPreparedArg → Bool → Bool → Except WorksheetErrorCode Bool
        | [], sawValue, decided =>
            if decided then
              .ok false
            else if sawValue then
              .ok true
            else
              .error .value
        | x :: xs, sawValue, decided =>
            if decided then
              match x.value with
              | .error e => .error e
              | _ => loop xs sawValue true
            else
              match andArgumentTruth x with
              | .error e => .error e
              | .ok (some false) => loop xs sawValue true
              | .ok (some true) => loop xs true decided
              | .ok none => loop xs sawValue decided
      loop args false false

theorem evalAndPrepared_direct_text_is_value_error :
    evalAndPrepared [⟨.directScalar, .text "1"⟩] = .error .value := by
  native_decide

theorem evalAndPrepared_reference_text_and_blank_are_ignored :
    evalAndPrepared [
      ⟨.arrayLike, .text "x"⟩,
      ⟨.arrayLike, .emptyCell⟩,
      ⟨.arrayLike, .logical true⟩
    ] = .ok true := by
  native_decide

theorem evalAndPrepared_all_ignored_is_value_error :
    evalAndPrepared [
      ⟨.arrayLike, .text "x"⟩,
      ⟨.arrayLike, .emptyCell⟩
    ] = .error .value := by
  native_decide

/-- `=AND(FALSE,1/0)` -> `#DIV/0!`: an error after the deciding FALSE surfaces (`oxf-xvt5.14`). -/
theorem evalAndPrepared_error_after_deciding_false_surfaces :
    evalAndPrepared [
      ⟨.directScalar, .logical false⟩,
      ⟨.directScalar, .error .div0⟩
    ] = .error .div0 := by
  native_decide

/-- `=AND(1/0,FALSE)` -> `#DIV/0!`: an error before the deciding value surfaces. -/
theorem evalAndPrepared_error_before_deciding_false_surfaces :
    evalAndPrepared [
      ⟨.directScalar, .error .div0⟩,
      ⟨.directScalar, .logical false⟩
    ] = .error .div0 := by
  native_decide

/-- `=AND(FALSE,NA(),1/0)` -> `#N/A` and `=AND(FALSE,1/0,NA())` -> `#DIV/0!`: with several error
arguments the FIRST in argument order wins — a positional rule, not an error-code ranking. -/
theorem evalAndPrepared_first_error_in_argument_order_wins :
    evalAndPrepared [
      ⟨.directScalar, .logical false⟩,
      ⟨.directScalar, .error .na⟩,
      ⟨.directScalar, .error .div0⟩
    ] = .error .na
    ∧ evalAndPrepared [
      ⟨.directScalar, .logical false⟩,
      ⟨.directScalar, .error .div0⟩,
      ⟨.directScalar, .error .na⟩
    ] = .error .div0 := by
  constructor <;> native_decide

/-- `=AND(FALSE,TRUE,1)` is still FALSE: scanning past the deciding value changes nothing when
no later item is an error. -/
theorem evalAndPrepared_still_false_when_later_items_are_not_errors :
    evalAndPrepared [
      ⟨.directScalar, .logical false⟩,
      ⟨.directScalar, .logical true⟩,
      ⟨.directScalar, .number 1⟩
    ] = .ok false := by
  native_decide

theorem andMeta_profiles :
    andMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ andMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [andMeta]

end OxFunc.Functions
