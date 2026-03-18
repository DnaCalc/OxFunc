import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

def chooseMeta : FunctionMeta := {
  functionId := "FUNC.CHOOSE"
  arity := { min := 2, max := 255 }
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

def ifsMeta : FunctionMeta := {
  functionId := "FUNC.IFS"
  arity := { min := 2, max := 254 }
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

inductive DeferredChoice where
  | ready (value : CoercionInput)
  | poison
  deriving DecidableEq, Repr

inductive ChooseIfsValue where
  | number (n : Rat)
  | text (s : String)
  | logical (b : Bool)
  | error (code : WorksheetErrorCode)
  deriving DecidableEq, Repr

def materializeDeferred : DeferredChoice → Except String ChooseIfsValue
  | .ready (.number n) => .ok (.number n)
  | .ready (.text s) => .ok (.text s)
  | .ready (.logical b) => .ok (.logical b)
  | .ready (.error code) => .ok (.error code)
  | .ready .emptyCell => .ok (.number 0)
  | .ready .missingArg => .ok (.error .value)
  | .poison => .error "forced_branch"

/-- `CHOOSE` range checking happens after adapter-side numeric coercion and truncation. This Lean
    slice models the already-truncated 1-based index selection rule over deferred branches. -/
def evalChoosePrepared : Nat → List DeferredChoice → Except String ChooseIfsValue
  | 0, _ => .ok (.error .value)
  | _, [] => .ok (.error .value)
  | 1, branch :: _ => materializeDeferred branch
  | index + 1, _ :: rest => evalChoosePrepared index rest

/-- `IFS` scans left-to-right, forces only the first matching branch, and returns `#N/A` when no
    condition is true. -/
def evalIfsPrepared : List (Bool × DeferredChoice) → Except String ChooseIfsValue
  | [] => .ok (.error .na)
  | (false, _) :: rest => evalIfsPrepared rest
  | (true, branch) :: _ => materializeDeferred branch

theorem chooseMeta_profiles :
    chooseMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ chooseMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ chooseMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [chooseMeta]

theorem ifsMeta_profiles :
    ifsMeta.argPreparationProfile = ArgPreparationProfile.refsVisibleInAdapter
    ∧ ifsMeta.coercionLiftProfile = CoercionLiftProfile.custom
    ∧ ifsMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [ifsMeta]

theorem evalChoosePrepared_selects_second_branch :
    evalChoosePrepared 2 [
      .ready (.number 10),
      .ready (.text "picked"),
      .poison
    ] = .ok (.text "picked") := by
  rfl

theorem evalChoosePrepared_out_of_range_is_value :
    evalChoosePrepared 4 [
      .ready (.number 10),
      .ready (.number 20),
      .ready (.number 30)
    ] = .ok (.error .value) := by
  rfl

theorem evalIfsPrepared_first_true_short_circuits :
    evalIfsPrepared [
      (false, .poison),
      (true, .ready (.text "hit")),
      (true, .poison)
    ] = .ok (.text "hit") := by
  rfl

theorem evalIfsPrepared_no_match_is_na :
    evalIfsPrepared [
      (false, .ready (.number 1)),
      (false, .ready (.number 2))
    ] = .ok (.error .na) := by
  rfl

end OxFunc.Functions
