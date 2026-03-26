import OxFunc.FunctionCore
import OxFunc.Functions.PowerFn

namespace OxFunc.Functions

open OxFunc

def financialTimeValuePureMeta (functionId : String) : FunctionMeta := {
  functionId := functionId
  arity := { min := 3, max := 5 }
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

def rateMeta : FunctionMeta :=
  { (financialTimeValuePureMeta "FUNC.RATE") with arity := { min := 3, max := 6 } }

def irregularFinancialMeta (functionId : String) : FunctionMeta :=
  { (financialTimeValuePureMeta functionId) with arity := { min := 2, max := 3 } }

inductive PaymentTiming where
  | endOfPeriod
  | beginningOfPeriod
  deriving DecidableEq, Repr

def timingFactor (periodicRate : Rat) : PaymentTiming → Rat
  | .endOfPeriod => 1
  | .beginningOfPeriod => 1 + periodicRate

def growthIntegerPublication (periodicRate : Rat) (periods : Nat) : Rat :=
  powerNatPublication (1 + periodicRate) periods

def annuityTermIntegerPublication
    (periodicRate : Rat)
    (periods : Nat)
    (timing : PaymentTiming) : Rat :=
  if periodicRate = 0 then
    periods
  else
    timingFactor periodicRate timing * (growthIntegerPublication periodicRate periods - 1) / periodicRate

def pvIntegerPublication
    (periodicRate : Rat)
    (periods : Nat)
    (paymentValue : Rat)
    (futureValue : Rat)
    (timing : PaymentTiming) : Rat :=
  if periodicRate = 0 then
    -(futureValue + paymentValue * periods)
  else
    let factor := growthIntegerPublication periodicRate periods;
    let term := annuityTermIntegerPublication periodicRate periods timing;
    -(futureValue + paymentValue * term) / factor

def fvIntegerPublication
    (periodicRate : Rat)
    (periods : Nat)
    (paymentValue : Rat)
    (presentValue : Rat)
    (timing : PaymentTiming) : Rat :=
  let factor := growthIntegerPublication periodicRate periods;
  let term := annuityTermIntegerPublication periodicRate periods timing;
  -(presentValue * factor + paymentValue * term)

def pmtIntegerPublication
    (periodicRate : Rat)
    (periods : Nat)
    (presentValue : Rat)
    (futureValue : Rat)
    (timing : PaymentTiming) : Rat :=
  if periodicRate = 0 then
    -(futureValue + presentValue) / periods
  else
    let factor := growthIntegerPublication periodicRate periods;
    let term := annuityTermIntegerPublication periodicRate periods timing;
    -(futureValue + presentValue * factor) / term

/--
W24 Batch 11 packet-evidences the admitted scalar/sequence financial family.
W53 adds a narrow executable alignment layer for the repaired integer-period
publication lanes by routing growth through the shared `POWER` integer-publication
helper. The full floating-point solver/publication family still remains outside
the current Lean scope.
-/
theorem financial_time_value_family_profiles :
    (financialTimeValuePureMeta "FUNC.PV").hostInteraction = HostInteractionClass.none
    ∧ (financialTimeValuePureMeta "FUNC.PMT").threadSafety = ThreadSafetyClass.safePure
    ∧ rateMeta.arity = { min := 3, max := 6 }
    ∧ (irregularFinancialMeta "FUNC.MIRR").surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [financialTimeValuePureMeta, rateMeta, irregularFinancialMeta]

theorem growth_integer_publication_seed_05_10 :
    growthIntegerPublication (1 / 20 : Rat) 10 = (16679880978201 / 10240000000000 : Rat) := by
  native_decide

theorem pv_integer_publication_seed_05_10 :
    pvIntegerPublication (1 / 20 : Rat) 10 (-100) 0 .endOfPeriod =
      (12879761956402000 / 16679880978201 : Rat) := by
  native_decide

theorem fv_integer_publication_seed_05_10 :
    fvIntegerPublication (1 / 20 : Rat) 10 (-100) 0 .endOfPeriod =
      (6439880978201 / 5120000000 : Rat) := by
  native_decide

theorem pmt_integer_publication_seed_05_10 :
    pmtIntegerPublication (1 / 20 : Rat) 10 1000 0 .endOfPeriod =
      (-833994048910050 / 6439880978201 : Rat) := by
  native_decide

theorem financial_growth_uses_shared_power_publication :
    growthIntegerPublication (1 / 20 : Rat) 10 = powerNatPublication (21 / 20 : Rat) 10 := by
  native_decide

end OxFunc.Functions
