import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def ifMeta : FunctionMeta := {
  functionId := "FUNC.IF"
  arity := { min := 2, max := 3 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def ifConditionTruthy : CoercionInput → Except CoercionError Bool
  | .logical b => Except.ok b
  | .number n => Except.ok (n ≠ 0)
  | .missingArg => Except.ok false
  | .emptyCell => Except.ok false
  | other =>
      match coerceToNumber other with
      | Except.ok n => Except.ok (n ≠ 0)
      | Except.error e => Except.error e

def evalIfAdapter (cond : CoercionInput) (thenVal elseVal : Rat) : Except CoercionError Rat :=
  match ifConditionTruthy cond with
  | Except.ok true => Except.ok thenVal
  | Except.ok false => Except.ok elseVal
  | Except.error e => Except.error e

theorem evalIfAdapter_true_branch :
    evalIfAdapter (.logical true) 10 20 = Except.ok 10 := by
  simp [evalIfAdapter, ifConditionTruthy]

theorem evalIfAdapter_false_branch :
    evalIfAdapter (.number 0) 10 20 = Except.ok 20 := by
  simp [evalIfAdapter, ifConditionTruthy]

theorem evalIfAdapter_text_bad_errors :
    evalIfAdapter (.text "bad") 10 20 =
      Except.error (CoercionError.nonNumericText "bad") := by
  simp [evalIfAdapter, ifConditionTruthy, coerceToNumber, parseSimpleNumber]

theorem ifMeta_profiles :
    ifMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ ifMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ ifMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [ifMeta]

end OxFunc.Functions
