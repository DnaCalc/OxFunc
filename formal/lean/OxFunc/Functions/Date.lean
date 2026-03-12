import OxFunc.CoercionPrimitives
import OxFunc.FunctionCore
import OxFunc.ValueUniverse

namespace OxFunc.Functions

open OxFunc

private instance instDecidableEqExceptDate [DecidableEq ε] [DecidableEq α] :
    DecidableEq (Except ε α)
  | .error a, .error b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .ok a, .ok b =>
      if h : a = b then isTrue (by cases h; rfl) else isFalse (by intro h'; cases h'; exact h rfl)
  | .error _, .ok _ => isFalse (by intro h; cases h)
  | .ok _, .error _ => isFalse (by intro h; cases h)

def dateMeta : FunctionMeta := {
  functionId := "FUNC.DATE"
  arity := Arity.exact 3
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

def truncateRatToInt (q : Rat) : Int :=
  if q.num < 0 then
    -Int.ediv (-q.num) q.den
  else
    Int.ediv q.num q.den

def daysFromCivil (year month day : Int) : Int :=
  let adjustedYear := year - if month ≤ 2 then 1 else 0
  let era :=
    if adjustedYear ≥ 0 then
      Int.ediv adjustedYear 400
    else
      Int.ediv (adjustedYear - 399) 400
  let yoe := adjustedYear - era * 400
  let mp := month + if month > 2 then (-3 : Int) else 9
  let doy := Int.ediv (153 * mp + 2) 5 + day - 1
  let doe := yoe * 365 + Int.ediv yoe 4 - Int.ediv yoe 100 + doy
  era * 146097 + doe - 719468

def excelSerialFromYmd (year month day : Int) : Except WorksheetErrorCode Int :=
  if year = 1900 ∧ month = 2 ∧ day = 29 then
    .ok 60
  else
    let base := daysFromCivil 1899 12 31
    let days := daysFromCivil year month 1 - base + (day - 1)
    if days < 0 then
      .error .num
    else if days ≥ 60 then
      .ok (days + 1)
    else
      .ok days

def normalizeDateYear (year : Int) : Int :=
  if 0 ≤ year ∧ year ≤ 1899 then
    year + 1900
  else
    year

def normalizeDateYearMonth (year month : Int) : Int × Int :=
  let monthIndex := year * 12 + (month - 1)
  let normalizedYear := Int.ediv monthIndex 12
  let normalizedMonth := Int.emod monthIndex 12 + 1
  (normalizedYear, normalizedMonth)

def evalDatePrepared
    (year month day : CoercionInput) : Except WorksheetErrorCode Int :=
  match coerceToNumber year, coerceToNumber month, coerceToNumber day with
  | .ok y, .ok m, .ok d =>
      let yearValue := normalizeDateYear (truncateRatToInt y)
      let monthValue := truncateRatToInt m
      let dayValue := truncateRatToInt d
      let (normalizedYear, normalizedMonth) := normalizeDateYearMonth yearValue monthValue
      if normalizedYear < 0 ∨ normalizedYear > 9999 then
        .error .num
      else
        excelSerialFromYmd normalizedYear normalizedMonth dayValue
  | .error (.worksheetError code), _, _ => .error code
  | _, .error (.worksheetError code), _ => .error code
  | _, _, .error (.worksheetError code) => .error code
  | _, _, _ => .error .value

theorem evalDatePrepared_serial_zero_boundary :
    evalDatePrepared (.number 1900) (.number 1) (.number 0) = .ok 0 := by
  native_decide

theorem evalDatePrepared_month_zero_boundary_is_num :
    evalDatePrepared (.number 1900) (.number 0) (.number 1) = .error .num := by
  native_decide

theorem evalDatePrepared_preserves_1900_leap_bug :
    evalDatePrepared (.number 1900) (.number 2) (.number 29) = .ok 60 := by
  native_decide

theorem evalDatePrepared_short_year_and_truncated_day :
    evalDatePrepared (.number 0) (.number 1) (.number 1) = .ok 1
    ∧ evalDatePrepared (.number 2008) (.number 1) (.number (29 / 10 : Rat)) = .ok 39449 := by
  native_decide

theorem dateMeta_profiles :
    dateMeta.argPreparationProfile = ArgPreparationProfile.valuesOnlyPreAdapter
    ∧ dateMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [dateMeta]

end OxFunc.Functions
