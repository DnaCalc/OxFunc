import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def roundMeta : FunctionMeta := {
  functionId := "FUNC.ROUND"
  arity := Arity.exact 2
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.unaryNumericScalarOnly
  kernelSignatureClass := KernelSignatureClass.numsToNum
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def roundKernelSeed (n digits : Rat) : Rat :=
  if digits = 0 then Int.ediv (n.num + 5) 10 else n

def evalRoundAdapter (n digits : CoercionInput) : Except (EvalError ⊕ CoercionError) Value :=
  match coerceToNumber n, coerceToNumber digits with
  | Except.ok lhs, Except.ok rhs => Except.ok (.number (roundKernelSeed lhs rhs))
  | Except.error e, _ => Except.error (.inr e)
  | _, Except.error e => Except.error (.inr e)

theorem evalRoundAdapter_seed_zero_digits :
    evalRoundAdapter (.number 12) (.number 0) = Except.ok (.number (roundKernelSeed 12 0)) := by
  simp [evalRoundAdapter, coerceToNumber]

theorem roundMeta_profiles :
    roundMeta.kernelSignatureClass = KernelSignatureClass.numsToNum
    ∧ roundMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter := by
  simp [roundMeta]

end OxFunc.Functions
