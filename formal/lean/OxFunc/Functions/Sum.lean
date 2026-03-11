import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptSum [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def sumMeta : FunctionMeta := {
  functionId := "FUNC.SUM"
  arity := { min := 1, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
  kernelSignatureClass := KernelSignatureClass.numsToNum
  fecDependencyProfile := FecDependencyProfile.refOnly
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def sumKernel (nums : List Rat) : Rat := nums.foldl (fun acc n => acc + n) 0

inductive AggregateArrayProvenance where
  | directArrayLiteral
  | opaqueArrayValue
  | referenceDerived
  deriving DecidableEq, Repr, Inhabited

inductive SumArgOrigin where
  | directScalar
  | arrayLike (provenance : AggregateArrayProvenance)
  deriving DecidableEq, Repr, Inhabited

structure SumPreparedArg where
  origin : SumArgOrigin
  value : CoercionInput
  deriving DecidableEq, Repr

def evalSumPreparedValue : SumPreparedArg → Except CoercionError (Option Rat)
  | ⟨.directScalar, .missingArg⟩ => Except.ok (some 0)
  | ⟨.directScalar, .emptyCell⟩ => Except.ok (some 0)
  | ⟨.directScalar, value⟩ =>
      match coerceToNumber value with
      | Except.ok n => Except.ok (some n)
      | Except.error e => Except.error e
  | ⟨.arrayLike _, .number n⟩ => Except.ok (some n)
  | ⟨.arrayLike _, .error code⟩ => Except.error (.worksheetError code)
  | ⟨.arrayLike _, .text _⟩ => Except.ok none
  | ⟨.arrayLike _, .logical _⟩ => Except.ok none
  | ⟨.arrayLike _, .missingArg⟩ => Except.ok none
  | ⟨.arrayLike _, .emptyCell⟩ => Except.ok none

def collectSumNumbers : List SumPreparedArg → Except CoercionError (List Rat)
  | [] => Except.ok []
  | x :: xs =>
      match evalSumPreparedValue x, collectSumNumbers xs with
      | Except.ok (some n), Except.ok rest => Except.ok (n :: rest)
      | Except.ok none, Except.ok rest => Except.ok rest
      | Except.error e, _ => Except.error e
      | _, Except.error e => Except.error e

def evalSumPrepared (args : List SumPreparedArg) : Except CoercionError Value := do
  let nums ← collectSumNumbers args
  Except.ok (Value.number (sumKernel nums))

theorem sumKernel_nil_zero :
    sumKernel [] = 0 := by
  simp [sumKernel]

theorem evalSumPrepared_numbers :
    evalSumPrepared [
      ⟨.directScalar, CoercionInput.number 1⟩,
      ⟨.directScalar, CoercionInput.number 2⟩,
      ⟨.directScalar, CoercionInput.number 3⟩
    ] = Except.ok (Value.number (sumKernel [1, 2, 3])) := by
  native_decide

theorem evalSumPrepared_direct_scalar_text_and_logical :
    evalSumPrepared [
      ⟨.directScalar, CoercionInput.text "2"⟩,
      ⟨.directScalar, CoercionInput.logical true⟩
    ] = Except.ok (Value.number 3) := by
  native_decide

theorem evalSumPrepared_direct_array_literal_scan_policy :
    evalSumPrepared [
      ⟨.arrayLike .directArrayLiteral, CoercionInput.text "2"⟩,
      ⟨.arrayLike .directArrayLiteral, CoercionInput.logical true⟩
    ] = Except.ok (Value.number 0) := by
  native_decide

theorem evalSumPrepared_reference_derived_scan_policy :
    evalSumPrepared [
      ⟨.arrayLike .referenceDerived, CoercionInput.text "2"⟩,
      ⟨.arrayLike .referenceDerived, CoercionInput.logical true⟩
    ] = Except.ok (Value.number 0) := by
  native_decide

theorem evalSumPrepared_opaque_array_fallback_scan_policy :
    evalSumPrepared [
      ⟨.arrayLike .opaqueArrayValue, CoercionInput.text "2"⟩,
      ⟨.arrayLike .opaqueArrayValue, CoercionInput.logical true⟩
    ] = Except.ok (Value.number 0) := by
  native_decide

theorem evalSumPrepared_rejects_non_numeric_direct_text :
    evalSumPrepared [
      ⟨.directScalar, CoercionInput.number 1⟩,
      ⟨.directScalar, CoercionInput.text "bad"⟩
    ] = Except.error (CoercionError.nonNumericText "bad") := by
  native_decide

theorem sumMeta_profiles :
    sumMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ sumMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ sumMeta.kernelSignatureClass = KernelSignatureClass.numsToNum
    ∧ sumMeta.fecDependencyProfile = FecDependencyProfile.refOnly
    ∧ sumMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [sumMeta]

end OxFunc.Functions
