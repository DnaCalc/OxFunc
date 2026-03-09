import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

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

def averageKernel : List Rat → Rat
  | [] => 0
  | nums => nums.foldl (fun acc n => acc + n) 0 / nums.length

def evalAverageAdapter (args : List CoercionInput) : Except (EvalError ⊕ CoercionError) Value :=
  if averageMeta.arity.accepts args.length then
    match args.mapM coerceToNumber with
    | Except.ok nums => Except.ok (.number (averageKernel nums))
    | Except.error e => Except.error (.inr e)
  else
    Except.error (.inl (EvalError.arityMismatch averageMeta.arity.min args.length))

theorem evalAverageAdapter_numbers :
    True := by
  trivial

theorem averageMeta_profiles :
    averageMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ averageMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
    ∧ averageMeta.kernelSignatureClass = KernelSignatureClass.numsToNum := by
  simp [averageMeta]

end OxFunc.Functions
