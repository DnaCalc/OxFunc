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
  let base := daysFromCivil 1899 12 31
  let start := daysFromCivil year month 1 - base
  let start := if start ≥ 60 then start + 1 else start
  let serial := start + day - 1
  if serial < 0 ∨ serial > 2958465 then .error .num else .ok serial

/-- Finite date-argument binding. Exact rational gaps encode the observed
inclusive RN64-to-RN53 boundary; the downstream date arithmetic is integral. -/
def dateIntegerFloor (q : Rat) : Int :=
  let upper := -Int.ediv (-q.num) q.den
  let threshold : Rat := if upper > 0 then 2049 / 8589934592 else 2049 / 17179869184
  if (upper : Rat) - q ≤ threshold then upper else upper - 1

def normalizeDateYear (year : Int) : Int :=
  let year := min year 10000
  let bits := if -2147483648 ≤ year ∧ year < 2147483648 then year else 0
  let bits := Int.emod bits 65536
  let signed := if bits ≥ 32768 then bits - 65536 else bits
  signed + if signed < 1900 then 1900 else 0

def normalizeDateDay (day : Int) : Int :=
  if -32768 ≤ day ∧ day ≤ 32767 then day else 32767

def normalizeDateYearMonth (year month : Int) : Int × Int :=
  let monthIndex := year * 12 + (month - 1)
  let normalizedYear := Int.ediv monthIndex 12
  let normalizedMonth := Int.emod monthIndex 12 + 1
  (normalizedYear, normalizedMonth)

def dateCoerceNumber : CoercionInput → Except WorksheetErrorCode Rat
  | .missingArg | .emptyCell => .ok 0
  | arg => match coerceToNumber arg with
    | .ok n => .ok n
    | .error (.worksheetError code) => .error code
    | .error _ => .error .value

def evalDatePrepared
    (year month day : CoercionInput) : Except WorksheetErrorCode Int := do
      let y ← dateCoerceNumber year
      let m ← dateCoerceNumber month
      let d ← dateCoerceNumber day
      let yearValue := normalizeDateYear (dateIntegerFloor y)
      let monthValue := dateIntegerFloor m
      let dayValue := normalizeDateDay (dateIntegerFloor d)
      let (normalizedYear, normalizedMonth) := normalizeDateYearMonth yearValue monthValue
      if monthValue < -32767 ∨ monthValue ≥ 32767 ∨ normalizedYear < 1900 ∨ normalizedYear > 9999 then
        .error .num
      else
        excelSerialFromYmd normalizedYear normalizedMonth dayValue

theorem dateCoerceNumber_observed_preparation :
    dateCoerceNumber .missingArg = .ok 0 ∧
    dateCoerceNumber .emptyCell = .ok 0 ∧
    dateCoerceNumber (.logical true) = .ok 1 ∧
    evalDatePrepared (.text "x") (.error .div0) (.number 1) = .error .value := by native_decide

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

theorem evalDatePrepared_rollover_and_width_witnesses :
    evalDatePrepared (.number 1900) (.number 2) (.number 30) = .ok 61 ∧
    evalDatePrepared (.number (-1)) (.number 13) (.number 1) = .ok 1 ∧
    evalDatePrepared (.number (-65536)) (.number 1) (.number 1) = .ok 1 ∧
    evalDatePrepared (.number 65536) (.number 1) (.number 1) = .error .num ∧
    evalDatePrepared (.number 1900) (.number 1) (.number (-32769)) = .ok 32767 ∧
    evalDatePrepared (.number 9999) (.number 12) (.number 32) = .error .num := by
  native_decide

theorem dateIntegerFloor_boundary_witnesses :
    dateIntegerFloor (1 - 2049 / 8589934592) = 1 ∧
    dateIntegerFloor (1 - 2050 / 8589934592) = 0 ∧
    dateIntegerFloor (-2049 / 17179869184) = 0 ∧
    dateIntegerFloor (-2050 / 17179869184) = -1 := by
  native_decide

theorem normalizeDateYear_signed_width_and_cap :
    normalizeDateYear 10001 = 10000 ∧
    normalizeDateYear 65536 = 10000 ∧
    normalizeDateYear (-65537) = 1899 ∧
    normalizeDateYear (-67438) = -2 ∧
    normalizeDateYear (-848223069) = 9379 := by native_decide

end OxFunc.Functions
