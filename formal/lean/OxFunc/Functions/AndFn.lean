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

def evalAndPrepared : List AndPreparedArg → Except WorksheetErrorCode Bool
  | args =>
      let rec loop : List AndPreparedArg → Bool → Except WorksheetErrorCode Bool
        | [], sawValue =>
            if sawValue then
              .ok true
            else
              .error .value
        | x :: xs, sawValue =>
            match andArgumentTruth x with
            | .error e => .error e
            | .ok (some false) => .ok false
            | .ok (some true) => loop xs true
            | .ok none => loop xs sawValue
      loop args false

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

theorem andMeta_profiles :
    andMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ andMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [andMeta]

end OxFunc.Functions
