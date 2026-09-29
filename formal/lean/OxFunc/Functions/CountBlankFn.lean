import OxFunc.FunctionCore
import OxFunc.CoercionPrimitives

namespace OxFunc.Functions

open OxFunc

def countBlankMeta : FunctionMeta := {
  functionId := "FUNC.COUNTBLANK"
  arity := { min := 1, max := 255 }
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.aggregateDirectAndRangeDualPolicy
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def countBlankReferencedValue : CoercionInput → Bool
  | .emptyCell => true
  | .text value => value.isEmpty
  | _ => false

def countBlankComputedArrayCell : CoercionInput → WorksheetErrorCode
  | .error code => code
  | _ => .value

-- Admitted computed scalar Number/Text/Logical/Error values have the same
-- result rule. Empty/missing carriers and the evaluator's reference-returning
-- expressions are separate admission lanes, not claimed by this binding.
def countBlankComputedScalarResult : CoercionInput → WorksheetErrorCode :=
  countBlankComputedArrayCell

theorem countBlank_computed_scalar_rejects_values_and_preserves_errors :
    countBlankComputedScalarResult (.number 2) = .value
    ∧ countBlankComputedScalarResult (.text "") = .value
    ∧ countBlankComputedScalarResult (.logical true) = .value
    ∧ countBlankComputedScalarResult (.error .div0) = .div0 := by
  native_decide

theorem countBlank_reference_and_computed_array_error_policy :
    countBlankReferencedValue (.error .na) = false
    ∧ countBlankReferencedValue .emptyCell = true
    ∧ countBlankReferencedValue (.text "") = true
    ∧ countBlankComputedArrayCell (.error .na) = .na
    ∧ countBlankComputedArrayCell (.number 1) = .value := by
  native_decide

theorem countBlankMeta_profiles :
    countBlankMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ countBlankMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [countBlankMeta]

end OxFunc.Functions
