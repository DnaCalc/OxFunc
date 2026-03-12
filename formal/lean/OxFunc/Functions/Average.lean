import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptAverage [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def averageMeta : FunctionMeta := {
  functionId := "FUNC.AVERAGE"
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

inductive AverageArgOrigin where
  | directScalar
  | arrayLike
  deriving DecidableEq, Repr

structure AveragePreparedArg where
  origin : AverageArgOrigin
  value : CoercionInput
  deriving DecidableEq, Repr

def averageInputErrorToWorksheet : CoercionError → WorksheetErrorCode
  | .worksheetError code => code
  | _ => .value

def averageArgumentValue : AveragePreparedArg → Except WorksheetErrorCode (Option Rat)
  | ⟨.directScalar, value⟩ =>
      match coerceToNumber value with
      | .ok n => .ok (some n)
      | .error e => .error (averageInputErrorToWorksheet e)
  | ⟨.arrayLike, .number n⟩ => .ok (some n)
  | ⟨.arrayLike, .error code⟩ => .error code
  | ⟨.arrayLike, .text _⟩
  | ⟨.arrayLike, .logical _⟩
  | ⟨.arrayLike, .missingArg⟩
  | ⟨.arrayLike, .emptyCell⟩ => .ok none

def collectAverageState : List AveragePreparedArg → Except WorksheetErrorCode (Rat × Nat)
  | [] => .ok (0, 0)
  | x :: xs =>
      match averageArgumentValue x, collectAverageState xs with
      | .ok (some n), .ok (acc, count) => .ok (n + acc, count + 1)
      | .ok none, .ok state => .ok state
      | .error e, _ => .error e
      | _, .error e => .error e

def evalAveragePrepared : List AveragePreparedArg → Except WorksheetErrorCode Rat
  | args =>
      match collectAverageState args with
      | .error e => .error e
      | .ok (_, 0) => .error .div0
      | .ok (acc, count + 1) => .ok (acc / (count + 1 : Rat))

theorem evalAveragePrepared_direct_numeric_text_and_logical :
    evalAveragePrepared [
      ⟨.directScalar, .text "2"⟩,
      ⟨.directScalar, .logical true⟩
    ] = .ok (3 / 2 : Rat) := by
  native_decide

theorem evalAveragePrepared_array_like_text_and_logical_div0 :
    evalAveragePrepared [
      ⟨.arrayLike, .text "2"⟩,
      ⟨.arrayLike, .logical true⟩
    ] = .error .div0 := by
  native_decide

theorem evalAveragePrepared_reference_style_numeric_survivor :
    evalAveragePrepared [
      ⟨.arrayLike, .number 6⟩,
      ⟨.arrayLike, .text "2"⟩,
      ⟨.arrayLike, .logical true⟩
    ] = .ok 6 := by
  native_decide

theorem averageMeta_profiles :
    averageMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ averageMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ averageMeta.kernelSignatureClass = KernelSignatureClass.numsToNum := by
  simp [averageMeta]

end OxFunc.Functions
