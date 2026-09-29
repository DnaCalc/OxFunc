import OxFunc.FunctionCore
import OxFunc.Functions.Date

namespace OxFunc.Functions

open OxFunc

/-- EDATE/EOMONTH and WEEKNUM reject logical arguments and missing required
arguments. WEEKDAY uses DATE-style zero preparation for explicit missing slots.
WEEKNUM's absent or explicitly omitted selector defaults before this binding. -/
def dateWeekPreparedNumber (strict : Bool) : CoercionInput → Except WorksheetErrorCode Rat
  | .logical b => if strict then .error .value else .ok (if b then 1 else 0)
  | .missingArg => if strict then .error .na else .ok 0
  | arg => dateCoerceNumber arg

theorem dateWeekPreparedNumber_observed_kinds :
    dateWeekPreparedNumber true (.logical true) = .error .value ∧
    dateWeekPreparedNumber true .missingArg = .error .na ∧
    dateWeekPreparedNumber true .emptyCell = .ok 0 ∧
    dateWeekPreparedNumber false .missingArg = .ok 0 ∧
    dateWeekPreparedNumber false (.logical true) = .ok 1 := by native_decide

/-- Raw serial admission for EDATE/EOMONTH/WEEKNUM/ISOWEEKNUM. WEEKDAY instead
uses the rounded date-part binding before this integer weekday substrate. -/
def dateWeekSerialAdmission (serial : Rat) : Option Int :=
  if serial < 0 ∨ serial ≥ 2958466 then none
  else some (Int.ediv serial.num serial.den)

def dateMonthTargetAdmission (year month offset : Int) : Option (Int × Int) :=
  let target := normalizeDateYearMonth year (month + offset)
  if target.1 < 1900 ∨ target.1 > 9999 then none else some target

def weekdayIntegerBinding (serial selector : Int) : Option Int :=
  let sunday := Int.emod (serial - 1) 7 + 1
  if selector = 1 then some sunday
  else if selector = 2 ∨ selector = 11 then some (Int.emod (sunday + 5) 7 + 1)
  else if selector = 3 then some (Int.emod (sunday + 5) 7)
  else if 12 ≤ selector ∧ selector ≤ 17 then
    let start := Int.emod (selector - 10) 7 + 1
    some (Int.emod (sunday - start) 7 + 1)
  else none

theorem dateWeekAdmission_bounds :
    dateWeekSerialAdmission (-1 / 10) = none ∧
    dateWeekSerialAdmission (29584659 / 10) = some 2958465 ∧
    dateWeekSerialAdmission 2958466 = none ∧
    dateMonthTargetAdmission 1900 1 (-1) = none ∧
    dateMonthTargetAdmission 9999 12 1 = none := by native_decide

theorem weekdayIntegerBinding_observed_selectors :
    weekdayIntegerBinding 43831 1 = some 4 ∧
    weekdayIntegerBinding 43831 2 = some 3 ∧
    weekdayIntegerBinding 43831 3 = some 2 ∧
    weekdayIntegerBinding 43831 13 = some 1 ∧
    weekdayIntegerBinding 43831 21 = none := by native_decide

def dateWeekBaseMeta : FunctionMeta := {
  functionId := "FUNC.DATE_WEEK_BASE"
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

def edateMeta : FunctionMeta := {
  dateWeekBaseMeta with
  functionId := "FUNC.EDATE"
  arity := Arity.exact 2
}

def eomonthMeta : FunctionMeta := {
  dateWeekBaseMeta with
  functionId := "FUNC.EOMONTH"
  arity := Arity.exact 2
}

def weekdayMeta : FunctionMeta := {
  dateWeekBaseMeta with
  functionId := "FUNC.WEEKDAY"
  arity := { min := 1, max := 2 }
}

def weeknumMeta : FunctionMeta := {
  dateWeekBaseMeta with
  functionId := "FUNC.WEEKNUM"
  arity := { min := 1, max := 2 }
}

def isoweeknumMeta : FunctionMeta := {
  dateWeekBaseMeta with
  functionId := "FUNC.ISOWEEKNUM"
}

theorem dateWeekMeta_profiles :
    edateMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ eomonthMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ weekdayMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ weeknumMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ isoweeknumMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [dateWeekBaseMeta, edateMeta, eomonthMeta, weekdayMeta, weeknumMeta, isoweeknumMeta]

end OxFunc.Functions
