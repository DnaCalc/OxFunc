import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

def countAMeta : FunctionMeta := {
  functionId := "FUNC.COUNTA"
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

def countAArgumentIncluded : CoercionInput → Bool
  | .missingArg => false
  | .emptyCell => false
  | _ => true

def evalCountAPrepared (args : List CoercionInput) : Nat :=
  args.foldl (fun acc arg => if countAArgumentIncluded arg then acc + 1 else acc) 0

theorem evalCountAPrepared_counts_empty_string_and_error :
    evalCountAPrepared [.text "", .error .na, .emptyCell] = 2 := by
  native_decide

theorem evalCountAPrepared_ignores_missing_and_empty :
    evalCountAPrepared [.missingArg, .emptyCell] = 0 := by
  native_decide

theorem countAMeta_profiles :
    countAMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ countAMeta.coercionLiftProfile = CoercionLiftProfile.aggregateDirectAndRangeDualPolicy := by
  simp [countAMeta]

end OxFunc.Functions
