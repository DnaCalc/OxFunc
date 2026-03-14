namespace OxFunc

inductive LocaleProfileId where
  | enUs
  | currentExcelHost
  deriving DecidableEq, Repr

inductive WorkbookDateSystem where
  | system1900
  | system1904
  deriving DecidableEq, Repr

def parseValueText (profile : LocaleProfileId) (dateSystem : WorkbookDateSystem) (text : String) : Option Rat :=
  if profile = .currentExcelHost ∧ dateSystem = .system1900 ∧ text = "1234.5" then some (2469 / 2)
  else if profile = .currentExcelHost ∧ dateSystem = .system1900 ∧ text = "1 234.5" then some (2469 / 2)
  else if profile = .currentExcelHost ∧ dateSystem = .system1900 ∧ text = "123" then some 123
  else if profile = .currentExcelHost ∧ dateSystem = .system1900 ∧ text = "R1 234.57" then some (123457 / 100)
  else if profile = .currentExcelHost ∧ dateSystem = .system1900 ∧ text = "12%" then some (12 / 100)
  else if profile = .currentExcelHost ∧ dateSystem = .system1900 ∧ text = "2024-02-03" then some 45325
  else if profile = .currentExcelHost ∧ dateSystem = .system1900 ∧ text = "1/2/2024" then none
  else if profile = .enUs ∧ dateSystem = .system1900 ∧ text = "1/2/2024" then some 45293
  else none

/-- Admitted render subset for the local W13 shim. -/
def renderWithCode (profile : LocaleProfileId) (dateSystem : WorkbookDateSystem) (value : Rat) (code : String) : Option String :=
  if profile = .currentExcelHost ∧ dateSystem = .system1900 ∧ value = (1234567 / 1000 : Rat) ∧ code = "0" then some "1235"
  else if profile = .currentExcelHost ∧ dateSystem = .system1900 ∧ value = (1234567 / 1000 : Rat) ∧ code = "0.00" then some "1234.57"
  else if profile = .currentExcelHost ∧ dateSystem = .system1900 ∧ value = (123 : Rat) ∧ code = "0.00" then some "123.00"
  else if profile = .currentExcelHost ∧ dateSystem = .system1900 ∧ value = (1 / 8 : Rat) ∧ code = "0%" then some "13%"
  else if profile = .currentExcelHost ∧ dateSystem = .system1900 ∧ value = (45325 : Rat) ∧ code = "yyyy-mm-dd" then some "2024-02-03"
  else none

def renderCurrency (profile : LocaleProfileId) (value : Rat) (decimals : Int) : Option String :=
  if profile = .currentExcelHost ∧ value = (1234567 / 1000 : Rat) ∧ decimals = 2 then some "R1 234.57"
  else if profile = .currentExcelHost ∧ value = (-1234567 / 1000 : Rat) ∧ decimals = 2 then some "-R1 234.57"
  else if profile = .currentExcelHost ∧ value = (1 : Rat) ∧ decimals = 2 then some "R1.00"
  else if profile = .currentExcelHost ∧ value = (0 : Rat) ∧ decimals = 2 then some "R0.00"
  else if profile = .currentExcelHost ∧ value = (123 : Rat) ∧ decimals = 2 then some "R123.00"
  else none

def renderFixed (profile : LocaleProfileId) (value : Rat) (decimals : Int) (noCommas : Bool) : Option String :=
  if profile = .currentExcelHost ∧ value = (1234567 / 1000 : Rat) ∧ decimals = 2 ∧ noCommas = false then some "1 234.57"
  else if profile = .currentExcelHost ∧ value = (1234567 / 1000 : Rat) ∧ decimals = 2 ∧ noCommas = true then some "1234.57"
  else if profile = .currentExcelHost ∧ value = (-1234567 / 1000 : Rat) ∧ decimals = 2 ∧ noCommas = false then some "-1 234.57"
  else if profile = .currentExcelHost ∧ value = (1 : Rat) ∧ decimals = 2 then some "1.00"
  else if profile = .currentExcelHost ∧ value = (0 : Rat) ∧ decimals = 2 then some "0.00"
  else if profile = .currentExcelHost ∧ value = (123 : Rat) ∧ decimals = 2 then some "123.00"
  else none

theorem parseValueText_currentHost_grouped_numeric :
    parseValueText .currentExcelHost .system1900 "1 234.5" = some (2469 / 2) := by
  native_decide

theorem parseValueText_currentHost_currency :
    parseValueText .currentExcelHost .system1900 "R1 234.57" = some (123457 / 100) := by
  native_decide

theorem parseValueText_currentHost_percent :
    parseValueText .currentExcelHost .system1900 "12%" = some (12 / 100) := by
  native_decide

theorem parseValueText_currentHost_iso_date :
    parseValueText .currentExcelHost .system1900 "2024-02-03" = some 45325 := by
  native_decide

theorem parseValueText_currentHost_slash_date_rejected :
    parseValueText .currentExcelHost .system1900 "1/2/2024" = none := by
  native_decide

theorem parseValueText_enUs_slash_date :
    parseValueText .enUs .system1900 "1/2/2024" = some 45293 := by
  native_decide

theorem renderWithCode_currentHost_percent_seed :
    renderWithCode .currentExcelHost .system1900 (1 / 8) "0%" = some "13%" := by
  native_decide

theorem renderCurrency_currentHost_positive_seed :
    renderCurrency .currentExcelHost (1234567 / 1000) 2 = some "R1 234.57" := by
  native_decide

theorem renderFixed_currentHost_nocommas_seed :
    renderFixed .currentExcelHost (1234567 / 1000) 2 true = some "1234.57" := by
  native_decide

end OxFunc
