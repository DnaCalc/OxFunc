import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptOr [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def orMeta : FunctionMeta := {
  functionId := "FUNC.OR"
  arity := { min := 1, max := 255 }
  determinism := .deterministic
  volatility := .nonvolatile
  hostInteraction := .none
  threadSafety := .safePure
  argPreparationProfile := .valuesOnlyPreAdapter
  coercionLiftProfile := .custom
  kernelSignatureClass := .custom
  fecDependencyProfile := .none
  surfaceFecDependencyProfile := .refOnly
}

inductive OrArgOrigin where
  | directScalar
  | arrayLike
  deriving DecidableEq, Repr

structure OrPreparedArg where
  origin : OrArgOrigin
  value : CoercionInput
  deriving DecidableEq, Repr

/-- The same per-item truth rule AND uses (Rust `and_argument_truth` serves AND, OR and XOR). -/
def orArgumentTruth : OrPreparedArg → Except WorksheetErrorCode (Option Bool)
  | ⟨_, .logical b⟩ => .ok (some b)
  | ⟨_, .number n⟩ => .ok (some (n ≠ 0))
  | ⟨_, .error code⟩ => .error code
  | ⟨.directScalar, .text _⟩ => .error .value
  | ⟨.arrayLike, .text _⟩
  | ⟨_, .missingArg⟩
  | ⟨_, .emptyCell⟩ => .ok none

/-- Mirrors Rust `eval_or_surface` (`crates/oxfunc_core/src/functions/or_fn.rs`). Excel
evaluates every argument of OR: once an item has decided the result (`decided`, a TRUE was
seen) the remaining items are still scanned, and an error VALUE among them surfaces — the first
error in argument order winning over any later one (live Excel 16.0 build 20326, COM probe
2026-09-15, bead `oxf-xvt5.14`: `=OR(TRUE,1/0)` -> `#DIV/0!`, `=OR(TRUE,NA(),1/0)` -> `#N/A`).
Before that bead the Rust loop returned TRUE on the first TRUE; this file carried no kernel
model. -/
def evalOrPrepared : List OrPreparedArg → Except WorksheetErrorCode Bool
  | args =>
      let rec loop : List OrPreparedArg → Bool → Bool → Except WorksheetErrorCode Bool
        | [], sawValue, decided =>
            if decided then
              .ok true
            else if sawValue then
              .ok false
            else
              .error .value
        | x :: xs, sawValue, decided =>
            if decided then
              match x.value with
              | .error e => .error e
              | _ => loop xs sawValue true
            else
              match orArgumentTruth x with
              | .error e => .error e
              | .ok (some true) => loop xs sawValue true
              | .ok (some false) => loop xs true decided
              | .ok none => loop xs sawValue decided
      loop args false false

/-- `=OR(TRUE,1/0)` -> `#DIV/0!`: an error after the deciding TRUE surfaces (`oxf-xvt5.14`). -/
theorem evalOrPrepared_error_after_deciding_true_surfaces :
    evalOrPrepared [
      ⟨.directScalar, .logical true⟩,
      ⟨.directScalar, .error .div0⟩
    ] = .error .div0 := by
  native_decide

/-- `=OR(TRUE,NA())` -> `#N/A`. -/
theorem evalOrPrepared_na_after_deciding_true_surfaces :
    evalOrPrepared [
      ⟨.directScalar, .logical true⟩,
      ⟨.directScalar, .error .na⟩
    ] = .error .na := by
  native_decide

/-- `=OR(1/0,TRUE)` -> `#DIV/0!`: an error before the deciding value surfaces. -/
theorem evalOrPrepared_error_before_deciding_true_surfaces :
    evalOrPrepared [
      ⟨.directScalar, .error .div0⟩,
      ⟨.directScalar, .logical true⟩
    ] = .error .div0 := by
  native_decide

/-- `=OR(TRUE,NA(),1/0)` -> `#N/A` and `=OR(TRUE,1/0,NA())` -> `#DIV/0!`: with several error
arguments the FIRST in argument order wins — a positional rule, not an error-code ranking. -/
theorem evalOrPrepared_first_error_in_argument_order_wins :
    evalOrPrepared [
      ⟨.directScalar, .logical true⟩,
      ⟨.directScalar, .error .na⟩,
      ⟨.directScalar, .error .div0⟩
    ] = .error .na
    ∧ evalOrPrepared [
      ⟨.directScalar, .logical true⟩,
      ⟨.directScalar, .error .div0⟩,
      ⟨.directScalar, .error .na⟩
    ] = .error .div0 := by
  constructor <;> native_decide

/-- `=OR(TRUE,FALSE,0)` is still TRUE: scanning past the deciding value changes nothing when
no later item is an error. -/
theorem evalOrPrepared_still_true_when_later_items_are_not_errors :
    evalOrPrepared [
      ⟨.directScalar, .logical true⟩,
      ⟨.directScalar, .logical false⟩,
      ⟨.directScalar, .number 0⟩
    ] = .ok true := by
  native_decide

/-- Reference text and blank cells are ignored; nothing but ignored items is `#VALUE!` (the two
existing Rust unit tests, mirrored). -/
theorem evalOrPrepared_reference_text_and_blank_are_ignored :
    evalOrPrepared [
      ⟨.arrayLike, .text "x"⟩,
      ⟨.arrayLike, .emptyCell⟩,
      ⟨.arrayLike, .number 0⟩
    ] = .ok false
    ∧ evalOrPrepared [
      ⟨.arrayLike, .text "x"⟩,
      ⟨.arrayLike, .emptyCell⟩
    ] = .error .value := by
  constructor <;> native_decide

theorem orMeta_profiles :
    orMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ orMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ orMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [orMeta]

end OxFunc.Functions
