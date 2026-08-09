import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptAcoth [DecidableEq ε] [DecidableEq α] : DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def acothMeta : FunctionMeta := {
  functionId := "FUNC.ACOTH"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalAcothSurfaceClass (input : CoercionInput) : Except WorksheetErrorCode String :=
  match coerceToNumber input with
  | .ok n => if n ≤ 1 ∧ (-1 : Rat) ≤ n then .error .num else .ok "number"
  | .error (.worksheetError code) => .error code
  | .error _ => .error .value

/--
W109 executable publication-route tag for the identified current-reference
ACOTH graph. The tag records the load-bearing route order without duplicating
the Rust/x87 numeric backend: a subnormal reciprocal publishes positive zero;
otherwise the exact binary64 threshold `0x400d92b14ec204f3` selects either the
stored x87 ratio-log graph or the stored x87 direct inverse odd-power series.
-/
inductive AcothPublicationRoute where
  | reciprocalFlushPositiveZero
  | storedX87RatioLog
  | storedX87InverseOddPowerSeries
  deriving DecidableEq, Repr

def acothPublicationRoute (reciprocalFlush belowSeriesThreshold : Bool) : AcothPublicationRoute :=
  if reciprocalFlush then
    .reciprocalFlushPositiveZero
  else if belowSeriesThreshold then
    .storedX87RatioLog
  else
    .storedX87InverseOddPowerSeries

theorem evalAcoth_abs_one_is_num :
    evalAcothSurfaceClass (.number 1) = .error .num := by
  native_decide

theorem acothMeta_profiles :
    acothMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ acothMeta.coercionLiftProfile = CoercionLiftProfile.unaryNumericScalarOrArrayElementwise
    ∧ acothMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [acothMeta]

theorem acoth_publication_route_order_and_split :
    acothPublicationRoute true false = .reciprocalFlushPositiveZero
    ∧ acothPublicationRoute false true = .storedX87RatioLog
    ∧ acothPublicationRoute false false = .storedX87InverseOddPowerSeries := by
  native_decide

end OxFunc.Functions
