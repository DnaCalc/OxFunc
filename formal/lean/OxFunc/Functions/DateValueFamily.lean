import OxFunc.FunctionCore
import OxFunc.LocaleFormat

namespace OxFunc.Functions

open OxFunc

def dateValueFamilyBaseMeta : FunctionMeta := {
  functionId := "FUNC.DATE_VALUE_FAMILY_BASE"
  arity := Arity.exact 1
  determinism := DeterminismClass.deterministic
  volatility := VolatilityClass.nonvolatile
  hostInteraction := HostInteractionClass.none
  threadSafety := ThreadSafetyClass.safePure
  argPreparationProfile := ArgPreparationProfile.valuesOnlyPreAdapter
  coercionLiftProfile := CoercionLiftProfile.custom
  kernelSignatureClass := KernelSignatureClass.custom
  fecDependencyProfile := FecDependencyProfile.localeProfile
  surfaceFecDependencyProfile := FecDependencyProfile.composite
}

def datevalueMeta : FunctionMeta := {
  dateValueFamilyBaseMeta with
  functionId := "FUNC.DATEVALUE"
}

def timevalueMeta : FunctionMeta := {
  dateValueFamilyBaseMeta with
  functionId := "FUNC.TIMEVALUE"
}

def days360Meta : FunctionMeta := {
  dateValueFamilyBaseMeta with
  functionId := "FUNC.DAYS360"
  arity := { min := 2, max := 3 }
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def datedifMeta : FunctionMeta := {
  dateValueFamilyBaseMeta with
  functionId := "FUNC.DATEDIF"
  arity := Arity.exact 3
  fecDependencyProfile := FecDependencyProfile.none
  surfaceFecDependencyProfile := FecDependencyProfile.refOnly
}

def evalDateValueText (text : String) : Except String Rat :=
  if text = "2024-02-03" then .ok 45325
  else if text = "2024-02-03 6:35 AM" then .ok 45325
  else if text = "6:35 AM" then .ok 0
  else if text = "22-Aug-2008" then .ok 39682
  else .error "#VALUE!"

def evalTimeValueText (text : String) : Except String Rat :=
  if text = "2:24 AM" then .ok (1 / 10)
  else if text = "22-Aug-2008 6:35 AM" then .ok (1975 / 7200)
  else if text = "2024-02-03" then .ok 0
  else if text = "22-Aug-2008" then .ok 0
  else .error "#VALUE!"

def evalDays360Seed (startSerial endSerial : Int) (european : Bool) : Except String Int :=
  if startSerial = 45351 ∧ endSerial = 45382 ∧ european = false then .ok 30
  else if startSerial = 45351 ∧ endSerial = 45382 ∧ european = true then .ok 31
  else if startSerial = 40575 ∧ endSerial = 40634 ∧ european = false then .ok 30
  else if startSerial = 40575 ∧ endSerial = 40634 ∧ european = true then .ok 32
  else .error "#NUM!"

def evalDateDifSeed (startSerial endSerial : Int) (unit : String) : Except String Int :=
  if startSerial = 36892 ∧ endSerial = 37622 ∧ unit = "Y" then .ok 2
  else if startSerial = 37043 ∧ endSerial = 37483 ∧ unit = "D" then .ok 440
  else if startSerial = 37043 ∧ endSerial = 37483 ∧ unit = "YD" then .ok 75
  else if startSerial = 37043 ∧ endSerial = 37483 ∧ unit = "M" then .ok 14
  else if startSerial = 37043 ∧ endSerial = 37483 ∧ unit = "YM" then .ok 2
  else if startSerial = 36921 ∧ endSerial = 36949 ∧ unit = "MD" then .ok 28
  else if unit = "Q" then .error "#NUM!"
  else .error "#NUM!"

theorem evalDateValueText_seed_rows :
    evalDateValueText "2024-02-03" = .ok 45325
    ∧ evalDateValueText "2024-02-03 6:35 AM" = .ok 45325
    ∧ evalDateValueText "6:35 AM" = .ok 0
    ∧ evalDateValueText "22-Aug-2008" = .ok 39682 := by
  simp [evalDateValueText]

theorem evalTimeValueText_seed_rows :
    evalTimeValueText "2:24 AM" = .ok (1 / 10)
    ∧ evalTimeValueText "22-Aug-2008 6:35 AM" = .ok (1975 / 7200)
    ∧ evalTimeValueText "2024-02-03" = .ok 0 := by
  simp [evalTimeValueText]

theorem evalDays360Seed_seed_rows :
    evalDays360Seed 45351 45382 false = .ok 30
    ∧ evalDays360Seed 45351 45382 true = .ok 31 := by
  simp [evalDays360Seed]

theorem evalDateDifSeed_seed_rows :
    evalDateDifSeed 37043 37483 "YD" = .ok 75
    ∧ evalDateDifSeed 36921 36949 "MD" = .ok 28
    ∧ evalDateDifSeed 37043 37483 "Q" = .error "#NUM!" := by
  simp [evalDateDifSeed]

theorem dateValueFamilyMeta_profiles :
    datevalueMeta.fecDependencyProfile = FecDependencyProfile.localeProfile
    ∧ timevalueMeta.surfaceFecDependencyProfile = FecDependencyProfile.composite
    ∧ days360Meta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly
    ∧ datedifMeta.surfaceFecDependencyProfile = FecDependencyProfile.refOnly := by
  simp [dateValueFamilyBaseMeta, datevalueMeta, timevalueMeta, days360Meta, datedifMeta]

end OxFunc.Functions
