import OxFunc.FunctionCore
import OxFunc.Functions.Date

namespace OxFunc.Functions

open OxFunc

/-- Binary64 extraction binding: add half a second in day units before splitting
the serial. Integer calendar interpretation is bound by the DATE substrate and
the retained Rust surface replay. -/
def datePartsSerialBinding (serial : Float) : Option (Nat × Nat × Nat × Nat) :=
  let rounded := serial + 0.5 / 86400.0
  if !(serial ≥ 0.0 && rounded < 2958466.0) then none else
  let day := rounded.floor
  let fraction := rounded - day
  let hours := fraction * 24.0
  let hour := hours.floor
  let minutes := (hours - hour) * 60.0
  let minute := minutes.floor
  let second := ((minutes - minute) * 60.0).floor.toUInt64.toNat % 60
  some (day.toUInt64.toNat, hour.toUInt64.toNat, minute.toUInt64.toNat, second)

/-- DAYS truncates both serials without the half-second date-part conversion. -/
def daysSerialBinding (ending starting : Rat) : Except WorksheetErrorCode Int :=
  if ending < 0 ∨ ending ≥ 2958466 ∨ starting < 0 ∨ starting ≥ 2958466 then .error .num
  else .ok (Int.ediv ending.num ending.den - Int.ediv starting.num starting.den)

def daysPreparedBinding (ending starting : CoercionInput) : Except WorksheetErrorCode Int :=
  match ending, starting with
  | .error code, _ => .error code
  | _, .error code => .error code
  | _, _ => do daysSerialBinding (← dateCoerceNumber ending) (← dateCoerceNumber starting)

theorem daysPreparedBinding_error_precedence :
    daysPreparedBinding (.text "x") (.error .div0) = .error .div0 ∧
    daysPreparedBinding (.error .na) (.error .div0) = .error .na ∧
    daysPreparedBinding .missingArg (.logical true) = .ok (-1) := by native_decide

theorem datePartsSerialBinding_observed_edges :
    datePartsSerialBinding 0.0 = some (0, 0, 0, 0) ∧
    datePartsSerialBinding 0.5 = some (0, 12, 0, 0) ∧
    datePartsSerialBinding 0.3361053240740741 = some (0, 8, 3, 59) ∧
    datePartsSerialBinding 0.9999999999999999 = some (1, 0, 0, 0) ∧
    datePartsSerialBinding (-0.1) = none ∧
    datePartsSerialBinding 2958465.99999999 = none := by native_decide

theorem daysSerialBinding_no_rounding :
    daysSerialBinding (9999999999999999 / 10000000000000000) 0 = .ok 0 ∧
    daysSerialBinding (-1 / 10) 0 = .error .num ∧
    daysSerialBinding 2958466 0 = .error .num := by native_decide

def datePartBaseMeta : FunctionMeta := {
  functionId := "FUNC.DATE_PART_BASE"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def dayMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.DAY" }
def monthMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.MONTH" }
def yearMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.YEAR" }
def daysMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.DAYS", arity := Arity.exact 2 }
def hourMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.HOUR" }
def minuteMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.MINUTE" }
def secondMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.SECOND" }
def timeMeta : FunctionMeta := { datePartBaseMeta with functionId := "FUNC.TIME", arity := Arity.exact 3 }

theorem datePartsMeta_profiles :
    dayMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ monthMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ yearMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ daysMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ hourMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ minuteMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ secondMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ timeMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [datePartBaseMeta, dayMeta, monthMeta, yearMeta, daysMeta, hourMeta, minuteMeta, secondMeta, timeMeta]

end OxFunc.Functions
