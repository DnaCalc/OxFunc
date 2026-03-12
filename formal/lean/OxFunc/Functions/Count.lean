import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptCount [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def countMeta : FunctionMeta := {
  functionId := "FUNC.COUNT"
  arity := { min := 1, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
  kernelSignatureClass := KernelSignatureClass.numsToNum
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

inductive CountArgOrigin where
  | directScalar
  | arrayLike
  deriving DecidableEq, Repr

structure CountPreparedArg where
  origin : CountArgOrigin
  value : CoercionInput
  deriving DecidableEq, Repr

def countArgumentIncluded : CountPreparedArg → Except WorksheetErrorCode Bool
  | ⟨_, .number _⟩ => .ok true
  | ⟨_, .error code⟩ => .error code
  | ⟨.directScalar, .text s⟩ =>
      match coerceToNumber (.text s) with
      | .ok _ => .ok true
      | .error (.nonNumericText _) => .ok false
      | .error (.worksheetError code) => .error code
      | .error _ => .error .value
  | ⟨.directScalar, .logical _⟩ => .ok true
  | ⟨.arrayLike, .text _⟩
  | ⟨.arrayLike, .logical _⟩
  | ⟨_, .missingArg⟩
  | ⟨_, .emptyCell⟩ => .ok false

def evalCountPrepared : List CountPreparedArg → Except WorksheetErrorCode Nat
  | [] => .ok 0
  | x :: xs =>
      match countArgumentIncluded x, evalCountPrepared xs with
      | .ok true, .ok rest => .ok (rest + 1)
      | .ok false, .ok rest => .ok rest
      | .error e, _ => .error e
      | _, .error e => .error e

theorem evalCountPrepared_direct_numeric_text_and_logical :
    evalCountPrepared [
      ⟨.directScalar, .text "2"⟩,
      ⟨.directScalar, .logical true⟩,
      ⟨.directScalar, .number 1⟩
    ] = .ok 3 := by
  native_decide

theorem evalCountPrepared_array_like_text_and_logical_ignored :
    evalCountPrepared [
      ⟨.arrayLike, .text "2"⟩,
      ⟨.arrayLike, .logical true⟩,
      ⟨.arrayLike, .number 1⟩
    ] = .ok 1 := by
  native_decide

theorem evalCountPrepared_direct_nonnumeric_text_is_ignored :
    evalCountPrepared [⟨.directScalar, .text "bad"⟩] = .ok 0 := by
  native_decide

theorem countMeta_profiles :
    countMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ countMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy := by
  simp [countMeta]

end OxFunc.Functions
