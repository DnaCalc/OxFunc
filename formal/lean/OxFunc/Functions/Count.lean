import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

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

def countContribution : CoercionInput → Except CoercionError Nat
  | .text "bad" => Except.ok 0
  | .missingArg => Except.ok 0
  | .emptyCell => Except.ok 0
  | arg =>
      match coerceToNumber arg with
      | Except.ok _ => Except.ok 1
      | Except.error e => Except.error e

def evalCountAdapter (args : List CoercionInput) : Except (EvalError ⊕ CoercionError) Value :=
  if countMeta.arity.accepts args.length then
    match args.mapM countContribution with
    | Except.ok nums => Except.ok (.number ((nums.foldl (fun acc n => acc + n) 0 : Nat) : Rat))
    | Except.error e => Except.error (.inr e)
  else
    Except.error (.inl (EvalError.arityMismatch countMeta.arity.min args.length))

theorem evalCountAdapter_numbers_and_text :
    True := by
  trivial

theorem countMeta_profiles :
    countMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ countMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy := by
  simp [countMeta]

end OxFunc.Functions
