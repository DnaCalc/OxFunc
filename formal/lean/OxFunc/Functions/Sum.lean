import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def sumMeta : FunctionMeta := {
  functionId := "FUNC.SUM"
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

def sumKernel (nums : List Rat) : Rat := nums.foldl (fun acc n => acc + n) 0

def coerceAllNumbers : List CoercionInput → Except CoercionError (List Rat)
  | [] => Except.ok []
  | x :: xs =>
      match coerceToNumber x, coerceAllNumbers xs with
      | Except.ok n, Except.ok rest => Except.ok (n :: rest)
      | Except.error e, _ => Except.error e
      | _, Except.error e => Except.error e

def evalSumAdapter (args : List CoercionInput) : Except (EvalError ⊕ CoercionError) Value :=
  if sumMeta.arity.accepts args.length then
    match coerceAllNumbers args with
    | Except.ok nums => Except.ok (Value.number (sumKernel nums))
    | Except.error e => Except.error (Sum.inr e)
  else
    Except.error (Sum.inl (EvalError.arityMismatch sumMeta.arity.min args.length))

theorem sumKernel_nil_zero :
    sumKernel [] = 0 := by
  simp [sumKernel]

theorem evalSumAdapter_numbers :
    evalSumAdapter [
      CoercionInput.number 1,
      CoercionInput.number 2,
      CoercionInput.number 3
    ] = Except.ok (Value.number (sumKernel [1, 2, 3])) := by
  simp [evalSumAdapter, sumMeta, Arity.accepts, coerceAllNumbers, sumKernel, coerceToNumber]

theorem evalSumAdapter_rejects_non_numeric_text :
    evalSumAdapter [CoercionInput.number 1, CoercionInput.text "bad"] =
      Except.error (Sum.inr (CoercionError.nonNumericText "bad")) := by
  simp [evalSumAdapter, sumMeta, Arity.accepts, coerceAllNumbers, coerceToNumber, parseSimpleNumber]

theorem sumMeta_profiles :
    sumMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ sumMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ sumMeta.kernelSignatureClass = KernelSignatureClass.numsToNum := by
  simp [sumMeta]

end OxFunc.Functions
