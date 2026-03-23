import OxFunc.FunctionCore
import OxFunc.ValueUniverse
import OxFunc.RefResolverSeam

namespace OxFunc.Functions

open OxFunc

def opImplicitIntersectionMeta : FunctionMeta := {
  functionId := "FUNC.OP_IMPLICIT_INTERSECTION"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.workbookState
  threadSafety := ThreadSafetyClass.hostSerialized
  argPreparationProfile := ArgPreparationProfile.refsVisibleInAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.composite
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

inductive ImplicitIntersectionSource where
  | scalar (v : ValueTag)
  | singleColumnRef (row1 row2 : Nat)
  | singleRowRef (col1 col2 : Nat)
  | twoDimRef
  | arrayTopLeft (v : ValueTag)
  deriving DecidableEq, Repr

inductive ImplicitIntersectionResult where
  | passthrough (v : ValueTag)
  | selected (v : ValueTag)
  | wsError (code : WorksheetErrorCode)
  deriving DecidableEq, Repr

def evalImplicitIntersection (callerRow callerCol : Nat) :
    ImplicitIntersectionSource → ImplicitIntersectionResult
  | .scalar v => .passthrough v
  | .arrayTopLeft v => .selected v
  | .singleColumnRef row1 row2 =>
      if row1 ≤ callerRow && callerRow ≤ row2 then
        .selected .number
      else
        .wsError .value
  | .singleRowRef col1 col2 =>
      if col1 ≤ callerCol && callerCol ≤ col2 then
        .selected .number
      else
        .wsError .value
  | .twoDimRef => .wsError .value

theorem implicitIntersection_meta_profile :
    opImplicitIntersectionMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ opImplicitIntersectionMeta.hostInteraction = HostInteractionClass.workbookState
    ∧ opImplicitIntersectionMeta.surfaceFecDependencyProfile = FecDependencyProfile.composite := by
  simp [opImplicitIntersectionMeta]

theorem implicitIntersection_scalar_passthrough :
    evalImplicitIntersection 5 3 (.scalar .text) = .passthrough .text := by
  rfl

theorem implicitIntersection_single_column_selects_when_row_aligned :
    evalImplicitIntersection 2 2 (.singleColumnRef 1 3) = .selected .number := by
  native_decide

theorem implicitIntersection_single_row_selects_when_col_aligned :
    evalImplicitIntersection 2 2 (.singleRowRef 1 3) = .selected .number := by
  native_decide

theorem implicitIntersection_array_payload_uses_top_left :
    evalImplicitIntersection 9 9 (.arrayTopLeft .number) = .selected .number := by
  rfl

theorem implicitIntersection_two_dimensional_current_baseline_is_value_error :
    evalImplicitIntersection 3 3 .twoDimRef = .wsError .value := by
  rfl

end OxFunc.Functions
