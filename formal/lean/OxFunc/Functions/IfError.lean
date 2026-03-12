import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

def ifErrorMeta : FunctionMeta := {
  functionId := "FUNC.IFERROR"
  arity := Arity.exact 2
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

inductive IfErrorValue where
  | number (n : Rat)
  | text (s : String)
  | logical (b : Bool)
  | error (code : WorksheetErrorCode)
  deriving DecidableEq, Repr

inductive DeferredFallback where
  | ready (value : CoercionInput)
  | poison
  deriving DecidableEq, Repr

def materializeIfErrorInput : CoercionInput → IfErrorValue
  | .number n => .number n
  | .text s => .text s
  | .logical b => .logical b
  | .error code => .error code
  | .emptyCell => .number 0
  | .missingArg => .error .value

def forceDeferredFallback : DeferredFallback → Except String CoercionInput
  | .ready value => .ok value
  | .poison => .error "forced_fallback"

def evalIfErrorPrepared
    (primary : CoercionInput)
    (fallback : DeferredFallback) : Except String IfErrorValue :=
  match primary with
  | .error _ => do
      let forced ← forceDeferredFallback fallback
      .ok (materializeIfErrorInput forced)
  | other => .ok (materializeIfErrorInput other)

theorem evalIfErrorPrepared_passes_non_error_through_without_forcing_fallback :
    evalIfErrorPrepared (.number 1) .poison = .ok (.number 1) := by
  rfl

theorem evalIfErrorPrepared_blank_primary_becomes_zero :
    evalIfErrorPrepared .emptyCell .poison = .ok (.number 0) := by
  rfl

theorem evalIfErrorPrepared_blank_fallback_becomes_zero :
    evalIfErrorPrepared (.error .na) (.ready .emptyCell) = .ok (.number 0) := by
  rfl

theorem evalIfErrorPrepared_missing_fallback_becomes_value_error :
    evalIfErrorPrepared (.error .na) (.ready .missingArg) = .ok (.error .value) := by
  rfl

theorem ifErrorMeta_profiles :
    ifErrorMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ ifErrorMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [ifErrorMeta]

end OxFunc.Functions
